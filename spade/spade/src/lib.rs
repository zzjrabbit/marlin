pub use spade_macro::spade;
pub use verilog::__reexports;

use std::env::current_dir;

use camino::{Utf8Path, Utf8PathBuf};
use snafu::{whatever, ResultExt, Whatever};
use verilog::{VerilatorRuntime, __reexports::verilator::VerilatedModel};

fn search_for_swim_toml(mut start: Utf8PathBuf) -> Option<Utf8PathBuf> {
    while !start.as_str().is_empty() {
        if start.join("swim.toml").is_file() {
            return Some(start.join("swim.toml"));
        }
        start.pop();
    }
    None
}

pub struct SpadeRuntime {
    verilator_runtime: VerilatorRuntime,
}

impl SpadeRuntime {
    pub fn new(artifact_directory: &Utf8Path) -> Result<Self, Whatever> {
        let Some(swim_toml_path) = search_for_swim_toml(
            current_dir()
                .whatever_context("Failed to get current directory")?
                .try_into()
                .whatever_context(
                    "Failed to convert current directory to UTF-8",
                )?,
        ) else {
            whatever!("Failed to find swim.toml");
        };

        let mut spade_sv_path = swim_toml_path;
        spade_sv_path.pop();
        spade_sv_path.push("build/spade.sv");

        Ok(Self {
            verilator_runtime: VerilatorRuntime::new(
                artifact_directory,
                &[&spade_sv_path],
            )?,
        })
    }

    pub fn create_model<M: VerilatedModel>(&mut self) -> Result<M, Whatever> {
        self.verilator_runtime.create_model()
    }
}
