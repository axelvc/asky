use asky::Confirm;

fn main() -> std::io::Result<()> {
    if Confirm::new("Do you like coffe?").prompt()? {
        println!("Great, me too!");
    }

    // ...

    Ok(())
}
