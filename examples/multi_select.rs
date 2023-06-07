use asky::MultiSelect;

fn main() -> std::io::Result<()> {
    let opts = ["Dog", "Cat", "Fish", "Bird", "Other"];
    let choices = MultiSelect::new("What kind of pets do you have?", opts).prompt()?;

    if choices.len() > 2 {
        println!("So you love pets");
    }

    // ...

    Ok(())
}
