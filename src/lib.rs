//! Good looking prompts for the terminal
//!
//! # Available prompts
//!
//! - [`Confirm`] - Ask yes/no questions.
//! - [`Toggle`] - Choose between two options.
//! - [`Text`] - One-line user input.
//! - [`Number`] - One-line user input of numbers.
//! - [`Password`] - One-line user input as password.
//! - [`Select`] - Select an item from a list.
//! - [`MultiSelect`] - Select multiple items from a list.
//!
//! # Simple Example
//!
//! ```rust, no_run
//! use asky::{Confirm, Text};
//!
//! fn main() -> std::io::Result<()> {
//!     let name = Text::new("Hi. What's your name?").prompt()?;
//!
//!     if Confirm::new("Do you like coffee?").prompt()? {
//!         println!("Great! Me too");
//!     } else {
//!         println!("Hmm... Interesting");
//!     }
//!
//!     // ...
//!
//!     Ok(())
//! }
//! ```
//!
//! # Customization
//!
//! If you'd like to use this crate but don't want the default styles or just want to customize as you like,
//! all the prompts allow setting a custom formatter using `format()` method.
//!
//! The formatter receives a prompt state reference and a [`DrawTime`],
//! and returns the string to display in the terminal.
//!
//! > Note: When using a custom formatter, you are responsible for the presentation of the prompt,
//! > so you must handle the colors, icons, etc. by yourself.
//!
//! #### Example
//!
//! ```rust, no_run
//! # use asky::Confirm;
//! # fn main() -> std::io::Result<()> {
//! Confirm::new("Do you like Rust?")
//!     .format(|prompt, _draw_time| {
//!         let state = if prompt.active { "Y/n" } else { "y/N" };
//!         format!("{} {}\n", prompt.message, state)
//!     })
//!     .prompt();
//! # Ok(())
//! # }
//! ```
//!
//! This will prints
//!
//! ```bash
//! Do you like Rust? y/N
//! ```
//!
//! ## Cursor Position
//!
//! Almost all the prompts just need a custom string, but some prompts like [`Text`] also requires an array of `[x, y]`
//! position for the cursor, due to these prompts also depends on the cursor position in the process.
//!
//! #### Example
//!
//! ```rust, no_run
//! # use asky::Text;
//! # fn main() -> std::io::Result<()> {
//! Text::new("What is your name")
//!     .format(|prompt, _draw_time| {
//!         let cursor_col = prompt.input.col;
//!         let prefix = "> ";
//!
//!         let x = (prefix.len() + cursor_col);
//!         let y = 1;
//!
//!         (
//!             format!("{}\n{} {}", prompt.message, prefix, prompt.input.value),
//!             [x, y],
//!         )
//!     })
//!     .prompt();
//! # Ok(())
//! # }
//!
//! ```
//!
//! This will prints
//!
//! ```bash
//! What is your name?
//! > |
//! ```
//!
//! Where `|` is the cursor position.
// #![deny(missing_docs)]

mod prompts;
mod utils;

#[cfg(feature="bevy")]
pub mod bevy;

pub use prompts::confirm::Confirm;
pub use prompts::multi_select::MultiSelect;
pub use prompts::number::Number;
pub use prompts::password::Password;
pub use prompts::select::Select;
pub use prompts::text::Text;
pub use prompts::toggle::Toggle;

pub use prompts::select::{SelectInput, SelectOption};
pub use prompts::text::LineInput;
pub use utils::num_like::NumLike;
pub use utils::renderer::DrawTime;
pub use utils::key_listener::Typeable;
