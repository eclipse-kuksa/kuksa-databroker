/********************************************************************************
* Copyright (c) 2022-2023 Contributors to the Eclipse Foundation
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

use anyhow::Result;

use vergen_gix::Build;
use vergen_gix::Cargo;
use vergen_gix::Emitter;
use vergen_gix::Gix;

// Extract build info (at build time)
fn main() -> Result<()> {
    let gitcl = Gix::all_git();
    let cargocl = Cargo::all_cargo();
    let build = Build::all_build();
    Emitter::default()
        .add_instructions(&cargocl)?
        .add_instructions(&gitcl)?
        .add_instructions(&build)?
        .emit()?;
    Ok(())
}
