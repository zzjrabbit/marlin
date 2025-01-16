use std::fs;

use libloading::Library;
use snafu::{ResultExt, Whatever};
use verilator::PortDirection;
use verilog::verilog;

#[verilog(src = "sv/main.sv", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    fs::create_dir_all("artifacts")
        .whatever_context("Failed to create artifacts directory")?;
    let library_path = verilator::build(
        &["sv/main.sv"],
        "main",
        &[
            ("single_input", 0, 0, PortDirection::Input),
            ("single_output", 0, 0, PortDirection::Output),
        ],
        "artifacts".as_ref(),
    )
    .whatever_context("Failed to build verilator dynamic library")?;

    let library =
        unsafe { Library::new(library_path) }.expect("failed to get lib");
    let new_main: extern "C" fn() -> *mut libc::c_void =
        *unsafe { library.get(b"ffi_new_Vmain") }
            .expect("failed to get symbol");
    let delete_main: extern "C" fn(*mut libc::c_void) =
        *unsafe { library.get(b"ffi_delete_Vmain") }
            .expect("failed to get symbol");
    let pin_input: extern "C" fn(*mut libc::c_void, i32) =
        *unsafe { library.get(b"ffi_Vmain_pin_single_input") }
            .expect("failed to get symbol");
    let read_output: extern "C" fn(*mut libc::c_void) -> i32 =
        *unsafe { library.get(b"ffi_Vmain_read_single_output") }
            .expect("failed to get symbol");
    let eval: extern "C" fn(*mut libc::c_void) =
        *unsafe { library.get(b"ffi_Vmain_eval") }
            .expect("failed to get symbol");

    let main = new_main();
    pin_input(main, 1);
    println!("{}", read_output(main));
    eval(main);
    println!("{}", read_output(main));
    delete_main(main);

    Ok(())
}
