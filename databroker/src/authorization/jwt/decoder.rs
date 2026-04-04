/********************************************************************************
* Copyright (c) 2023 Contributors to the Eclipse Foundation
*
* See the NOTICE file(s) distributed with this work for additional
* information regarding copyright ownership.
*
* This program and the accompanying materials are made available under the
* terms of the Apache License 2.0 which is available at
* http://www.apache.org/licenses/LICENSE-2.0
*
* SPDX-License-Identifier: Apache-2.0
********************************************************************************/

use std::convert::TryFrom;

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

use crate::permissions::{Permission, Permissions, PermissionsBuildError};

use super::scope;

#[derive(Debug)]
pub enum Error {
    PublicKeyError(String),
    DecodeError(String),
    ClaimsError,
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

#[derive(Clone, Debug)]
enum KeyKind { Rsa, Ec, Ed }

#[derive(Clone)]
pub struct Decoder {
    decoding_key: DecodingKey,
    key_kind: KeyKind,
}

#[derive(Debug, Deserialize)]
pub struct Claims {
    #[allow(dead_code)]
    pub sub: String, // Subject (whom token refers to)
    #[allow(dead_code)]
    pub iss: String, // Issuer
    #[allow(dead_code)]
    pub aud: Vec<String>, // Audience
    #[allow(dead_code)]
    pub iat: u64, // Issued at (as UTC timestamp)
    // nbf: usize, // Optional. Not Before (as UTC timestamp)
    #[allow(dead_code)]
    pub exp: u64, // Expiration time (as UTC timestamp)
    #[allow(dead_code)]
    pub scope: String,
}

impl Decoder {
    /// Create a Decoder from a PEM-encoded public key.
    /// Supports RSA (RS256/RS384/RS512), EC/P-256 (ES256/ES384), and Ed25519 (EdDSA).
    /// For PKCS#8 BEGIN PUBLIC KEY the constructor tries RSA, EC, then Ed in order.
    pub fn new(public_key: impl Into<String>) -> Result<Decoder, Error> {
        let pem = public_key.into();
        let pem_bytes = pem.as_bytes();
        let (decoding_key, key_kind) = if pem.contains("BEGIN EC PUBLIC KEY") {
            let dk = DecodingKey::from_ec_pem(pem_bytes)
                .map_err(|e| Error::PublicKeyError(format!("EC public key error: {e}")))?;
            (dk, KeyKind::Ec)
        } else if pem.contains("BEGIN RSA PUBLIC KEY") {
            let dk = DecodingKey::from_rsa_pem(pem_bytes)
                .map_err(|e| Error::PublicKeyError(format!("RSA public key error: {e}")))?;
            (dk, KeyKind::Rsa)
        } else {
            if let Ok(dk) = DecodingKey::from_rsa_pem(pem_bytes) {
                (dk, KeyKind::Rsa)
            } else if let Ok(dk) = DecodingKey::from_ec_pem(pem_bytes) {
                (dk, KeyKind::Ec)
            } else if let Ok(dk) = DecodingKey::from_ed_pem(pem_bytes) {
                (dk, KeyKind::Ed)
            } else {
                return Err(Error::PublicKeyError(
                    "Could not parse public key as RSA, EC, or Ed25519 PEM".into(),
                ));
            }
        };
        Ok(Decoder { decoding_key, key_kind })
    }

    /// Build Validation for the token's declared alg, constrained to the loaded key family.
    fn make_validator(&self, token: &str) -> Validation {
        let header_alg = decode_header(token).ok().map(|h| h.alg);
        let algorithm = match header_alg {
            Some(alg) => match (&self.key_kind, alg) {
                (KeyKind::Rsa, a @ (Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512)) => a,
                (KeyKind::Ec,  a @ (Algorithm::ES256 | Algorithm::ES384)) => a,
                (KeyKind::Ed,  a @ Algorithm::EdDSA) => a,
                (KeyKind::Rsa, _) => Algorithm::RS256,
                (KeyKind::Ec,  _) => Algorithm::ES256,
                (KeyKind::Ed,  _) => Algorithm::EdDSA,
            },
            None => match self.key_kind {
                KeyKind::Rsa => Algorithm::RS256,
                KeyKind::Ec  => Algorithm::ES256,
                KeyKind::Ed  => Algorithm::EdDSA,
            },
        };
        let mut validator = Validation::new(algorithm);
        // TODO: Make "aud" configurable.
        validator.set_audience(&["kuksa.val"]);
        validator
    }

    pub fn decode(&self, token: impl AsRef<str>) -> Result<Claims, Error> {
        let token = token.as_ref();
        let validator = self.make_validator(token);
        match decode::<Claims>(token, &self.decoding_key, &validator) {
            Ok(token) => Ok(token.claims),
            Err(err) => Err(Error::DecodeError(err.to_string())),
        }
    }
}

impl TryFrom<Claims> for Permissions {
    type Error = Error;

    fn try_from(claims: Claims) -> Result<Self, Self::Error> {
        let scopes = scope::parse_whitespace_separated(&claims.scope).map_err(|err| match err {
            scope::Error::ParseError => Error::ClaimsError,
        })?;

        let mut permissions = Permissions::builder();
        for scope in scopes {
            match scope.path {
                Some(path) => {
                    permissions = match scope.action {
                        scope::Action::Read => {
                            permissions.add_read_permission(Permission::Glob(path))
                        }
                        scope::Action::Actuate => {
                            permissions.add_actuate_permission(Permission::Glob(path))
                        }
                        scope::Action::Provide => {
                            permissions.add_provide_permission(Permission::Glob(path))
                        }
                        scope::Action::Create => {
                            permissions.add_create_permission(Permission::Glob(path))
                        }
                    }
                }
                None => {
                    // Empty path => all paths
                    permissions = match scope.action {
                        scope::Action::Read => permissions.add_read_permission(Permission::All),
                        scope::Action::Actuate => {
                            permissions.add_actuate_permission(Permission::All)
                        }
                        scope::Action::Provide => {
                            permissions.add_provide_permission(Permission::All)
                        }
                        scope::Action::Create => permissions.add_create_permission(Permission::All),
                    };
                }
            }
        }

        permissions = permissions
            .expires_at(std::time::UNIX_EPOCH + std::time::Duration::from_secs(claims.exp));

        permissions.build().map_err(|err| match err {
            PermissionsBuildError::BuildError => Error::ClaimsError,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_token() {
        let pub_key = "-----BEGIN PUBLIC KEY-----
MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA6ScE9EKXEWVyYhzfhfvg
+LC8NseiuEjfrdFx3HKkb31bRw/SeS0Rye0KDP7uzffwreKf6wWYGxVUPYmyKC7j
Pji5MpDBGM9r3pIZSvPUFdpTE5TiRHFBxWbqPSYt954BTLq4rMu/W+oq5Pdfnugb
voYpLf0dclBl1g9KyszkDnItz3TYbWhGMbsUSfyeSPzH0IADzLoifxbc5mgiR73N
CA/4yNSpfLoqWgQ2vdTM1182sMSmxfqSgMzIMUX/tiaXGdkoKITF1sULlLyWfTo9
79XRZ0hmUwvfzr3OjMZNoClpYSVbKY+vtxHyux9KOOtv9lPMsgYIaPXvisrsneDZ
fCS0afOfjgR96uHIe2UPSGAXru3yGziqEfpRZoxsgXaOe905ordLD5bSX14xkN7N
Cz7rxDLlxPQyxp4Vhog7p/QeUyydBpZjq2bAE5GAJtiu+XGvG8RypzJFKFQwMNsw
g1BoZVD0mb0MtU8KQmHcZIfY0FVer/CR0mUjfl1rHbtoJB+RY03lQvYNAD04ibAG
NI1RhlTziu35Xo6NDEgs9hVs9k3WrtF+ZUxhivWmP2VXhWruRakVkC1NzKGh54e5
/KlluFbBNpWgvWZqzWo9Jr7/fzHtR0Q0IZwkxh+Vd/bUZya1uLKqP+sTcc+aTHbn
AEiqOjPq0D6X45wCzIwjILUCAwEAAQ==
-----END PUBLIC KEY-----
";
        let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJsb2NhbCBkZXYiLCJpc3MiOiJjcmVhdGVUb2tlbi5weSIsImF1ZCI6WyJrdWtzYS52YWwiXSwiaWF0IjoxNzY4NTY0ODAwLCJleHAiOjE4NjE5MTk5OTksInNjb3BlIjoicmVhZDpWZWhpY2xlLlNwZWVkIn0.CeOLvMfINCGLii3ycsYBU3WINXTuPYK0Z5BKnHxoZwE7lKXLQQKEmqh3GO1kx8rHJZxe4YQAK543tQCZ2GZQBVM3uJmShLRnWFkMd-DB_LEDw6codw11UoxUNcgld-d5pnYfBXlVc45TvoYUMoaezEx3jsZKlDYnXxybC0W7uepwvex7Zz0H7zv2WJJ73Qz6gRn5Mm6jQthq0GBO1POsxTTLC9xwaL_8MEdYmUOXxa3pWexo0qv_50OWgAYzg0djzHp8oByh2aFwg0NhjD6IkraMRvj1xmLsOaZPpzV9dKlozRPia3efbsf5pgLhYEAb6iVpnifmEFHGn548lrjqqcGVTOS_8CIpihjh7iMsnEkpU2wKnrlDU2jg4XhPsZ7eCJLnFe0rB7Gu8WVXxRC6P0DQDjJR5rShLK4IfAWcZAFQjh9ZSat6Ii5TezdH5nXCaEpu7DPEZ3_HyyA54uW3l397v1q13saJmBVEc3egiO8mmaHWcClCVwm47_UZIh4tdMTtREWoKELXjTlGmHp4R4hFx7H5inRScs8iHYEe2fjY2-wVQUEv2aCw8zT-HQ9U7rew1Em8DiAJUJIDCbZMBT2t-USIVZUFrOiQ5BcCHW36rx5w4NcS0Y_8VGajKbnWqH_8MP66CdzrnZrBIAjRIZSUtk-4iQYRlYm3Y8z-n0A";

        let decoder = Decoder::new(pub_key).expect("Creation of decoder should succeed");

        match decoder.decode(token) {
            Ok(claims) => {
                assert_eq!(claims.scope, "read:Vehicle.Speed");
            }
            Err(err) => panic!("decode should succeed but failed with:{err}"),
        }
    }
}
