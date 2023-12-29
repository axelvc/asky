use asky::Message;

fn main() -> std::io::Result<()> {
    Message::new("You can press any key to continue.").prompt()?;
    println!("That's the Message.");

    // ...

    Ok(())
}
