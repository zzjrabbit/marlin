use std::fs;

use libloading::{Library, Symbol};
use snafu::{ResultExt, Whatever};
use verilator::PortDirection;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    fs::create_dir_all("artifacts")
        .whatever_context("Failed to create artifacts directory")?;
    verilator::build(
        &["sv/main.sv"],
        "main",
        &[
            ("single_input", 0, 0, PortDirection::Input),
            ("small_input", 7, 0, PortDirection::Input),
            ("medium_input", 63, 0, PortDirection::Input),
            ("big_input", 127, 0, PortDirection::Input),
            ("single_output", 0, 0, PortDirection::Output),
            ("small_output", 7, 0, PortDirection::Output),
            ("medium_output", 63, 0, PortDirection::Output),
            ("big_output", 127, 0, PortDirection::Output),
        ],
        "artifacts".as_ref(),
    )
    .whatever_context("Failed to build verilator dynamic library")?;

    //let library = unsafe { Library::new("artifacts/obj_dir/libVmain.so") }
    //    .expect("failed to get lib");
    //let new_main: Symbol<extern "C" fn() -> *mut libc::c_void> =
    //    unsafe { library.get(b"new_main") }.expect("failed to get symbol");
    //let eval: Symbol<extern "C" fn(*mut libc::c_void)> =
    //    unsafe { library.get(b"eval") }.expect("failed to get symbol");
    //let set_single: Symbol<extern "C" fn(*mut libc::c_void, i32)> =
    //    unsafe { library.get(b"set_single") }.expect("failed to get symbol");
    //let get_single: Symbol<extern "C" fn(*mut libc::c_void) -> i32> =
    //    unsafe { library.get(b"get_single") }.expect("failed to get symbol");
    //
    //let new_main = *new_main;
    //let eval = *eval;
    //let set_single = *set_single;
    //let get_single = *get_single;
    //
    //let main = new_main();
    //set_single(main, 1);
    //println!("{}", get_single(main));
    //eval(main);
    //println!("{}", get_single(main));

    Ok(())
}
