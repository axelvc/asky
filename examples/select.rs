use asky::Select;

fn main() -> std::io::Result<()> {
    let choice = Select::new("Choose number", 1..=30).prompt()?;
    println!("{choice}, interesting choice.");

    // ...

    Ok(())
}
