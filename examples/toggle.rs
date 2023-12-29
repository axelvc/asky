use asky::Toggle;

fn main() -> std::io::Result<()> {
    let tabs = Toggle::new("Which is better?", "Tabs", "Spaces").prompt()?;
    println!("I also prefer {tabs}.");

    // ...

    Ok(())
}
