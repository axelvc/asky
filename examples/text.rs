use asky::prelude::*;

fn main() -> Result<(), Error> {
    let color = Text::new("What's your \nfavorite color?")
        .prompt()?;
    let mut chars = color.chars();
    let mut capitalized: String = chars.next().map(|c| c.to_uppercase().collect()).unwrap();
    capitalized.extend(chars);
    println!("\n{capitalized} is a beautiful color.");

    // ...

    Ok(())
}
