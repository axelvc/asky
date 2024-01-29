use asky::prelude::*;
use asky::style::NoStyle;

fn main() -> Result<(), Error> {
    let color = Text::new("What's your \nfavorite color?")
        // .style(NoStyle)
        .prompt()?;
    println!("{color} is a beautiful color");

    // ...

    Ok(())
}
