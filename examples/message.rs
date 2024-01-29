use asky::Message;

fn main() -> std::io::Result<()> {
    Message::call_to_action("Hello", "Press any key to continue").prompt()?;
    Message::new("That's the message.").prompt()?;
    // println!("That's the Message.");

    // ...

    Ok(())
}
