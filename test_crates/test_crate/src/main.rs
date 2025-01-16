use snafu::Whatever;
use verilog::{verilog, VerilatorRuntime};

#[verilog(src = "sv/main.sv", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let mut runtime =
        VerilatorRuntime::new("artifacts".into(), &["sv/main.sv".as_ref()])?;

    let mut main = runtime.create_model::<Main>()?;

    main.medium_input = u32::MAX;
    println!("{}", main.medium_output);
    main.eval();
    println!("{}", main.medium_output);

    Ok(())
}
