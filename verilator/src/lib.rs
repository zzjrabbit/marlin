// Copyright (C) 2024 Ethan Uppal.
//
// This project is free software: you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, version 3 of the License only.
//
// This project is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public License for more
// details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this project. If not, see <https://www.gnu.org/licenses/>.

use std::{env, process::Command};

use camino::Utf8PathBuf;
use snafu::{prelude::*, Whatever};

// hardcoded knowledge:
// - output library is obj_dir/libV${top_module}.a

pub fn build(source_files: &[&str], top_module: &str) -> Result<(), Whatever> {
    let build_directory= Utf8PathBuf::from(env::var("OUT_DIR")
        .whatever_context("OUT_DIR environment variable not set: are you sure you're running this from a build script?")?)
        .join("obj_dir");

    println!("cargo::rustc-link-search={}", build_directory);
    println!("cargo::rustc-link-lib=V{}", top_module);

    for source_file in source_files {
        println!("cargo::rerun-if-changed={}", source_file);
    }

    let output = Command::new("verilator")
        .args(["--cc", "-sv", "--build", "-j", "0", "-Wall"])
        .args(["--Mdir", build_directory.as_str()])
        .args(["--top-module", top_module])
        .args(source_files)
        .output()
        .whatever_context("Invocation of verilator failed")?;

    if !output.status.success() {
        whatever!(
            "Invocation of verilator failed with nonzero exit code {}\n\n{}",
            output.status,
            String::from_utf8(output.stderr).unwrap_or_default()
        );
    }

    Ok(())
}
