use asky::Message;

fn main() -> std::io::Result<()> {
    Message::wait("Hello", "Press any key to continue").prompt()?;
    Message::new("That's the message.").prompt()?;
    // println!("That's the Message.");

    // ...

    Ok(())
}
