use asky::prelude::*;

fn main() -> Result<(), Error> {
    let second = Toggle::new("Which is better?", ["Tabs", "Spaces"]).prompt()?;
    println!("I also prefer {}.", if second { "spaces" } else { "tabs" });

    // ...

    Ok(())
}
