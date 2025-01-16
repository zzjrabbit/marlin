use snafu::Whatever;
use spade::{spade, SpadeRuntime};

#[spade(src = "main.spade", name = "main")]
struct Main;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let mut runtime = SpadeRuntime::new("artifacts".into())?;

    let mut main = runtime.create_model::<Main>()?;

    main.eval();
    println!("{}", main.out);

    Ok(())
}
