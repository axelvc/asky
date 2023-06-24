use asky::Number;

fn main() -> std::io::Result<()> {
    if let Ok(age) = Number::<u8>::new("How old are you?").prompt()? {
        if age <= 60 {
            println!("Pretty young");
        }
    }

    // ...

    Ok(())
}
