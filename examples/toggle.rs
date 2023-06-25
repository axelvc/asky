use asky::Toggle;

fn main() -> std::io::Result<()> {
    let _tabs = Toggle::new("Which is better?", ["Tabs", "Spaces"]).prompt()?;
    println!("Great choice");

    // ...

    Ok(())
}
