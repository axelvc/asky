use asky::prelude::*;

fn main() -> Result<(), Error> {
    if Confirm::new("Do you like coffe?").prompt()? {
        println!("Great, me too!");
    }

    // ...

    Ok(())
}
