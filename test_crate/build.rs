use snafu::Whatever;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    verilator::build(&["sv/main.sv"], "main")
}
