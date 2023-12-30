use asky::prelude::*;

fn main() -> Result<(), Error> {
    let options = 1..=30;
    let choice = Select::new("Choose number", options.clone()).prompt()?;
    println!("{}, interesting choice.", options.into_iter().nth(choice).unwrap());

    // ...

    Ok(())
}
