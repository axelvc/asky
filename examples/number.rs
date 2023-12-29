use asky::Number;

fn main() -> std::io::Result<()> {
    let mut result;
    loop {
        result = Number::<u8>::new("How old are you?").prompt()?;
        if result.is_ok() {
            break;
        }
    }
    if let Ok(age) = result {
        if age <= 60 {
            println!("Pretty young.");
        }
    }

    // ...

    Ok(())
}
