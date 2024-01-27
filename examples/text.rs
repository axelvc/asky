use asky::prelude::*;
use asky::style::NoStyle;

fn main() -> Result<(), Error> {
    let color = Text::new("What's your favorite color?")
        .with_style(NoStyle)
        .prompt()?;
    println!("{color} is a beautiful color");

    // ...

    Ok(())
}
