use asky::prelude::*;

fn main() -> Result<(), Error> {
    let mut result;
    loop {
        result = Number::<u8>::new("How old are you?").prompt();
        if result.is_ok() {
            break;
        } else {
            eprintln!("Sorry. I don't understand.");
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
