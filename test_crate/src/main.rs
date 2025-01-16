use snafu::Whatever;
use verilator::{PortDirection, VerilatorRuntime};
use verilog::verilog;

#[verilog(src = "sv/main.sv", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let mut runtime =
        VerilatorRuntime::new("artifacts".into(), &["sv/main.sv".as_ref()]);

    let mut main = runtime.create_model::<Main>()?;

    main.single_input = 1;
    println!("{}", main.single_output);
    main.eval();
    println!("{}", main.single_output);

    Ok(())
}
