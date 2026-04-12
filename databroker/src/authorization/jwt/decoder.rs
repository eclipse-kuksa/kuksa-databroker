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
enum KeyKind {
    Rsa,
    Ec,
    Ed,
}

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
    ///
    /// Supported formats:
    /// - RSA: SPKI (`BEGIN PUBLIC KEY`) or PKCS#1 (`BEGIN RSA PUBLIC KEY`) — RS256/RS384/RS512
    /// - EC: SPKI (`BEGIN PUBLIC KEY`) — ES256 (P-256) / ES384 (P-384)
    /// - Ed25519: SPKI (`BEGIN PUBLIC KEY`) — EdDSA
    ///
    /// The constructor tries each key family's parser in order; the OID embedded in the SPKI
    /// DER structure disambiguates RSA from EC automatically, so no PEM-label string matching
    /// is needed.
    pub fn new(public_key: impl Into<String>) -> Result<Decoder, Error> {
        let pem = public_key.into();
        let pem_bytes = pem.as_bytes();
        // Try each key family in order. jsonwebtoken's from_*_pem parsers accept both the
        // algorithm-agnostic SPKI format (BEGIN PUBLIC KEY, RFC 7468 §13) and the legacy
        // algorithm-specific formats (BEGIN RSA PUBLIC KEY / BEGIN EC PUBLIC KEY) where
        // applicable. The OID embedded in SPKI DER disambiguates key families automatically.
        let (decoding_key, key_kind) = if let Ok(dk) = DecodingKey::from_rsa_pem(pem_bytes) {
            (dk, KeyKind::Rsa)
        } else if let Ok(dk) = DecodingKey::from_ec_pem(pem_bytes) {
            (dk, KeyKind::Ec)
        } else if let Ok(dk) = DecodingKey::from_ed_pem(pem_bytes) {
            (dk, KeyKind::Ed)
        } else {
            return Err(Error::PublicKeyError(
                "Could not parse public key as RSA, EC, or Ed25519 PEM".into(),
            ));
        };
        Ok(Decoder {
            decoding_key,
            key_kind,
        })
    }

    /// Build a `Validation` for the token's declared algorithm, constrained to the loaded key
    /// family. Returns an error if the token header is malformed or if the declared algorithm
    /// is incompatible with the key type (algorithm-confusion guard).
    fn make_validator(&self, token: &str) -> Result<Validation, Error> {
        let alg = decode_header(token)
            .map_err(|e| Error::DecodeError(format!("Invalid token header: {e}")))?
            .alg;
        let algorithm = match (&self.key_kind, alg) {
            (KeyKind::Rsa, a @ (Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512)) => a,
            (KeyKind::Ec, a @ (Algorithm::ES256 | Algorithm::ES384)) => a,
            (KeyKind::Ed, a @ Algorithm::EdDSA) => a,
            (kind, alg) => {
                return Err(Error::DecodeError(format!(
                    "Token algorithm {alg:?} is not compatible with the loaded {kind:?} key"
                )));
            }
        };
        let mut validator = Validation::new(algorithm);
        // TODO: Make "aud" configurable.
        validator.set_audience(&["kuksa.val"]);
        Ok(validator)
    }

    pub fn decode(&self, token: impl AsRef<str>) -> Result<Claims, Error> {
        let token = token.as_ref();
        let validator = self.make_validator(token)?;
        decode::<Claims>(token, &self.decoding_key, &validator)
            .map(|t| t.claims)
            .map_err(|e| Error::DecodeError(e.to_string()))
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

    // EC P-256 public key (SPKI, from openssl ecparam -name prime256v1)
    const EC_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----\n\
        MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEu25UcRd2d2I7ADSPvHKqDOXOz3r6\n\
        MGG7aTQlnJuVDLwBEOCTxHldf0xvpHeJIXB3ijQCNT8biPI6yTHgJU7kRw==\n\
        -----END PUBLIC KEY-----\n";

    // Ed25519 key pair generated with openssl genpkey -algorithm ed25519
    const ED_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----\n\
        MCowBQYDK2VwAyEAJsQjain09Q5eK6QMjzQ1iyTJhXTdcP+xPQCMWT9XS9Q=\n\
        -----END PUBLIC KEY-----\n";

    // ES256 token signed with EC_PRIVATE_KEY above (exp: 9999999999)
    const EC_TOKEN: &str = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.\
        eyJzdWIiOiJ0ZXN0LWVjIiwiaXNzIjoidGVzdCIsImF1ZCI6WyJrdWtzYS52YWwiXSwi\
        aWF0IjoxMDAwMDAwMDAwLCJleHAiOjk5OTk5OTk5OTksInNjb3BlIjoicmVhZDpWZWhp\
        Y2xlLlNwZWVkIn0._x8jJjYR3V5c8jvK2T18AeRpDjl_-c64RFgXe_1wjtr-rwRKM6Zm\
        OMZddboSFqcSZWw-Ecs2NZn8pqsKxGuAYQ";

    // EdDSA token signed with the Ed25519 private key above (exp: 9999999999)
    const ED_TOKEN: &str = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.\
        eyJzdWIiOiJ0ZXN0LWVkIiwiaXNzIjoidGVzdCIsImF1ZCI6WyJrdWtzYS52YWwiXSwi\
        aWF0IjoxMDAwMDAwMDAwLCJleHAiOjk5OTk5OTk5OTksInNjb3BlIjoicmVhZDpWZWhp\
        Y2xlLlNwZWVkIn0.rc5TAOzQGd4cYFMY1YkzUtPs8ThEk7lR3n0ZfEAEi6nDdgDmS3DM\
        nc-5ANY5BlZ5aQJ7h_jH5WtcN35G3NorDQ";

    #[test]
    fn test_ec_key_decode() {
        let decoder =
            Decoder::new(EC_PUBLIC_KEY).expect("EC P-256 decoder creation should succeed");
        match decoder.decode(EC_TOKEN) {
            Ok(claims) => assert_eq!(claims.scope, "read:Vehicle.Speed"),
            Err(err) => panic!("EC token decode should succeed but failed with: {err}"),
        }
    }

    #[test]
    fn test_ed_key_decode() {
        let decoder = Decoder::new(ED_PUBLIC_KEY).expect("Ed25519 decoder creation should succeed");
        match decoder.decode(ED_TOKEN) {
            Ok(claims) => assert_eq!(claims.scope, "read:Vehicle.Speed"),
            Err(err) => panic!("EdDSA token decode should succeed but failed with: {err}"),
        }
    }

    #[test]
    fn test_invalid_key_rejected() {
        let result = Decoder::new("not a valid PEM key");
        assert!(
            result.is_err(),
            "Invalid PEM should return PublicKeyError, got Ok"
        );
    }

    // RSA-4096 public key in PKCS#1 format ("BEGIN RSA PUBLIC KEY") — same key as test_parse_token
    // but in the legacy PKCS#1 encoding (openssl rsa -pubin -RSAPublicKey_out).
    // Verifies the try-chain handles the legacy format without PEM-label string matching.
    const RSA_PKCS1_PUBLIC_KEY: &str = "-----BEGIN RSA PUBLIC KEY-----\n\
        MIICCgKCAgEA6ScE9EKXEWVyYhzfhfvg+LC8NseiuEjfrdFx3HKkb31bRw/SeS0R\n\
        ye0KDP7uzffwreKf6wWYGxVUPYmyKC7jPji5MpDBGM9r3pIZSvPUFdpTE5TiRHFB\n\
        xWbqPSYt954BTLq4rMu/W+oq5PdfnugbvoYpLf0dclBl1g9KyszkDnItz3TYbWhG\n\
        MbsUSfyeSPzH0IADzLoifxbc5mgiR73NCA/4yNSpfLoqWgQ2vdTM1182sMSmxfqS\n\
        gMzIMUX/tiaXGdkoKITF1sULlLyWfTo979XRZ0hmUwvfzr3OjMZNoClpYSVbKY+v\n\
        txHyux9KOOtv9lPMsgYIaPXvisrsneDZfCS0afOfjgR96uHIe2UPSGAXru3yGziq\n\
        EfpRZoxsgXaOe905ordLD5bSX14xkN7NCz7rxDLlxPQyxp4Vhog7p/QeUyydBpZj\n\
        q2bAE5GAJtiu+XGvG8RypzJFKFQwMNswg1BoZVD0mb0MtU8KQmHcZIfY0FVer/CR\n\
        0mUjfl1rHbtoJB+RY03lQvYNAD04ibAGNI1RhlTziu35Xo6NDEgs9hVs9k3WrtF+\n\
        ZUxhivWmP2VXhWruRakVkC1NzKGh54e5/KlluFbBNpWgvWZqzWo9Jr7/fzHtR0Q0\n\
        IZwkxh+Vd/bUZya1uLKqP+sTcc+aTHbnAEiqOjPq0D6X45wCzIwjILUCAwEAAQ==\n\
        -----END RSA PUBLIC KEY-----\n";

    /// PKCS#1 RSA key is the same mathematical key as the SPKI RSA key in test_parse_token,
    /// so the same RS256 token decodes correctly — proving backward compat without label matching.
    #[test]
    fn test_rsa_pkcs1_legacy_format() {
        let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJsb2NhbCBkZXYiLCJpc3MiOiJjcmVhdGVUb2tlbi5weSIsImF1ZCI6WyJrdWtzYS52YWwiXSwiaWF0IjoxNzY4NTY0ODAwLCJleHAiOjE4NjE5MTk5OTksInNjb3BlIjoicmVhZDpWZWhpY2xlLlNwZWVkIn0.CeOLvMfINCGLii3ycsYBU3WINXTuPYK0Z5BKnHxoZwE7lKXLQQKEmqh3GO1kx8rHJZxe4YQAK543tQCZ2GZQBVM3uJmShLRnWFkMd-DB_LEDw6codw11UoxUNcgld-d5pnYfBXlVc45TvoYUMoaezEx3jsZKlDYnXxybC0W7uepwvex7Zz0H7zv2WJJ73Qz6gRn5Mm6jQthq0GBO1POsxTTLC9xwaL_8MEdYmUOXxa3pWexo0qv_50OWgAYzg0djzHp8oByh2aFwg0NhjD6IkraMRvj1xmLsOaZPpzV9dKlozRPia3efbsf5pgLhYEAb6iVpnifmEFHGn548lrjqqcGVTOS_8CIpihjh7iMsnEkpU2wKnrlDU2jg4XhPsZ7eCJLnFe0rB7Gu8WVXxRC6P0DQDjJR5rShLK4IfAWcZAFQjh9ZSat6Ii5TezdH5nXCaEpu7DPEZ3_HyyA54uW3l397v1q13saJmBVEc3egiO8mmaHWcClCVwm47_UZIh4tdMTtREWoKELXjTlGmHp4R4hFx7H5inRScs8iHYEe2fjY2-wVQUEv2aCw8zT-HQ9U7rew1Em8DiAJUJIDCbZMBT2t-USIVZUFrOiQ5BcCHW36rx5w4NcS0Y_8VGajKbnWqH_8MP66CdzrnZrBIAjRIZSUtk-4iQYRlYm3Y8z-n0A";
        let decoder =
            Decoder::new(RSA_PKCS1_PUBLIC_KEY).expect("PKCS#1 RSA decoder creation should succeed");
        match decoder.decode(token) {
            Ok(claims) => assert_eq!(claims.scope, "read:Vehicle.Speed"),
            Err(err) => panic!("PKCS#1 RSA token decode should succeed but failed with: {err}"),
        }
    }

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
