use asky::prelude::*;

fn main() -> Result<(), Error> {
    let color = Text::new("What's your favorite color?").prompt()?;
    println!("{color} is a beautiful color");

    // ...

    Ok(())
}
