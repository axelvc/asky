use asky::prelude::*;

fn main() -> Result<(), Error> {
    let password = Password::new("What's your IG password?").prompt()?;

    if !password.is_empty() {
        println!("Ultra secure!");
    }

    // ...

    Ok(())
}
