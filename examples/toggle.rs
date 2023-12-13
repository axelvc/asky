use asky::Toggle;

fn main() -> std::io::Result<()> {
    let _tabs = Toggle::new("Which is better?", ["Tabs".into(), "Spaces".into()]).prompt()?;
    println!("Great choice");

    // ...

    Ok(())
}
