use asky::prelude::*;

fn main() -> Result<(), Error> {
    let choice = Select::new("Choose number", 1..=30).prompt()?;
    println!("{choice}, interesting choice.");

    // ...

    Ok(())
}
