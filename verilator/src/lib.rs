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

use std::{fmt::Write, fs, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use snafu::{prelude::*, Whatever};

pub mod types {
    /// From the Verilator documentation: "Data representing 'bit' of 1-8 packed
    /// bits."
    pub type CData = u8;

    /// From the Verilator documentation: "Data representing 'bit' of 9-16
    /// packed bits"
    pub type SData = u16;

    /// From the Verilator documentation: "Data representing 'bit' of 17-32
    /// packed bits."
    pub type IData = u32;

    /// From the Verilator documentation: "Data representing 'bit' of 33-64
    /// packed bits."
    pub type QData = u64;

    /// From the Verilator documentation: "Data representing one element of
    /// WData array."
    pub type EData = u32;

    /// From the Verilator documentation: "Data representing >64 packed bits
    /// (used as pointer)."
    pub type WData = EData;
}

// hardcoded knowledge:
// - output library is obj_dir/libV${top_module}.a
// - location of verilated.h
// - verilator library is obj_dir/libverilated.a

pub enum PortDirection {
    Input,
    Output,
    Inout,
}

fn build_ffi(
    artifact_directory: &Utf8Path,
    top: &str,
    ports: &[(&str, usize, usize, PortDirection)],
) -> Result<Utf8PathBuf, Whatever> {
    let ffi_wrappers = artifact_directory.join("ffi.cpp");

    let mut buffer = String::new();
    writeln!(
        &mut buffer,
        r#"
#include "verilated.h"
#include "V{top}.h"

extern "C" {{
    void* ffi_new_V{top}() {{
        return new V{top}{{}};
    }}

    
    void ffi_V{top}_eval(V{top}* top) {{
        top->eval();
    }}

    void ffi_delete_V{top}(V{top}* top) {{
        delete top;
    }}
"#
    )
    .whatever_context("Failed to format utility FFI")?;

    for (port, msb, lsb, direction) in ports {
        let width = msb - lsb + 1;
        if width > 64 {
            let underlying = format!(
                "Port `{}` on top module `{}` was larger than 64 bits wide",
                port, top
            );
            whatever!(Err(underlying), "We don't support larger than 64-bit width on ports yet because weird C linkage things");
        }
        let macro_prefix = match direction {
            PortDirection::Input => "VL_IN",
            PortDirection::Output => "VL_OUT",
            PortDirection::Inout => "VL_INOUT",
        };
        let macro_suffix = if width <= 8 {
            "8"
        } else if width <= 16 {
            "16"
        } else if width <= 32 {
            ""
        } else if width <= 64 {
            "64"
        } else {
            "W"
        };
        let type_macro = |name: Option<&str>| {
            format!(
                "{}{}({}, {}, {}{})",
                macro_prefix,
                macro_suffix,
                name.unwrap_or("/* return value */"),
                msb,
                lsb,
                if width > 64 {
                    format!(", {}", (width + 31) / 32) // words are 32 bits
                                                       // according to header
                                                       // file
                } else {
                    "".into()
                }
            )
        };

        if matches!(direction, PortDirection::Input | PortDirection::Inout) {
            let input_type = type_macro(Some("new_value"));
            writeln!(
                &mut buffer,
                r#"
    void ffi_V{top}_pin_{port}(V{top}* top, {input_type}) {{
        top->{port} = new_value;
    }}
            "#
            )
            .whatever_context("Failed to format input port FFI")?;
        }

        if matches!(direction, PortDirection::Output | PortDirection::Inout) {
            let return_type = type_macro(None);
            writeln!(
                &mut buffer,
                r#"
    {return_type} ffi_V{top}_read_{port}(V{top}* top) {{
        return top->{port};
    }}
            "#
            )
            .whatever_context("Failed to format output port FFI")?;
        }
    }

    writeln!(&mut buffer, "}} // extern \"C\"")
        .whatever_context("Failed to format ending brace")?;

    fs::write(&ffi_wrappers, buffer)
        .whatever_context("Failed to write FFI wrappers file")?;

    Ok(ffi_wrappers)
}

pub fn build(
    source_files: &[&str],
    top_module: &str,
    ports: &[(&str, usize, usize, PortDirection)],
    artifact_directory: &Utf8Path,
) -> Result<Utf8PathBuf, Whatever> {
    let ffi_artifact_directory = artifact_directory.join("ffi");
    fs::create_dir_all(&ffi_artifact_directory).whatever_context(
        "Failed to create ffi subdirectory under artifacts directory",
    )?;
    let verilator_artifact_directory = artifact_directory.join("obj_dir");

    let ffi_wrappers = build_ffi(&ffi_artifact_directory, top_module, ports)
        .whatever_context("Failed to build FFI wrappers")?;

    // bug in verilator#5226 means the directory must be relative to -Mdir
    let ffi_wrappers = Utf8Path::new("../ffi/ffi.cpp");

    let library_name = format!("V{}_dyn", top_module);
    let verilator_output = Command::new("verilator")
        .args(["--cc", "-sv", "--build", "-j", "0", "-Wall"])
        .args(["-CFLAGS", "-shared -fpic"])
        .args(["--lib-create", &library_name])
        .args(["--Mdir", verilator_artifact_directory.as_str()])
        .args(["--top-module", top_module])
        //.arg("-O3")
        .args(source_files)
        .arg(ffi_wrappers)
        .output()
        .whatever_context("Invocation of verilator failed")?;

    if !verilator_output.status.success() {
        whatever!(
            "Invocation of verilator failed with nonzero exit code {}\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}",
            verilator_output.status,
            String::from_utf8(verilator_output.stdout).unwrap_or_default(),
            String::from_utf8(verilator_output.stderr).unwrap_or_default()
        );
    }

    Ok(verilator_artifact_directory.join(format!("lib{}.so", library_name)))
}
