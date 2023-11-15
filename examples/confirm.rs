use asky::{Confirm, Message};

fn main() -> std::io::Result<()> {
    if Confirm::new("Do you like coffe?").prompt()? {
        Message::new("Great, me too!").prompt()?;
        // println!("Great, me too!");
    }

    // ...

    Ok(())
}
