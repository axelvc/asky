use asky::Password;

fn main() -> std::io::Result<()> {
    let password = Password::new("What's your IG password?").prompt()?;

    if password.len() >= 1 {
        println!("Ultra secure!");
    }

    // ...

    Ok(())
}