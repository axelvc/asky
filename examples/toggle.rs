use asky::prelude::*;

fn main() -> Result<(), Error> {
    let tabs = Toggle::new("Which is better?", "Tabs", "Spaces").prompt()?;
    println!("I also prefer {tabs}.");

    // ...

    Ok(())
}
