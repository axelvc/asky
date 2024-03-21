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
//! use asky::prelude::*;
//!
//! fn main() -> Result<(), Error> {
//! #   #[cfg(feature = "terminal")]
//!     let name = Text::new("Hi. What's your name?").prompt()?;
//!
//! #   #[cfg(feature = "terminal")]
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
//! # use asky::prelude::*;
//! # fn main() -> Result<(), Error> {
//! # #[cfg(feature = "terminal")]
//! Confirm::new("Do you like Rust?")
//!     .format(|prompt, out| {
//!         let state = if prompt.active { "Y/n" } else { "y/N" };
//!         write!(out, "{} {}\n", prompt.message, state)
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
//! # use asky::{Text, Error, Promptable, Printable};
//! # fn main() -> Result<(), Error> {
//! # #[cfg(feature = "terminal")]
//! Text::new("What is your name")
//!     .format(|prompt, out| {
//!         let cursor_col = prompt.input.col;
//!         let prefix = "> ";
//!
//!         let x = (prefix.len() + cursor_col);
//!         let y = 1;
//!
//!         write!(out, "{}\n{} {}", prompt.message, prefix, prompt.input.value)
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
#![cfg_attr(not(feature = "terminal"), allow(dead_code))]
mod prompts;
pub mod utils;
use std::borrow::Cow;

pub trait Valuable {
    type Output: Send;
    fn value(&self) -> Result<Self::Output, Error>;
}

pub trait SetValue {
    type Output: Send;
    fn set_value(&mut self, value: Self::Output) -> Result<(), Error>;
}

#[derive(Debug)]
pub enum Error {
    Cancel,
    InvalidInput,
    InvalidCount { expected: usize, actual: usize },
    ValidationFail,
    Message(Cow<'static, str>),
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Self {
        Error::Io(x)
    }
}

pub trait Promptable {
    type Output;
    fn prompt(&mut self) -> Result<Self::Output, crate::Error>;
}

pub use prompts::confirm::Confirm;
pub use prompts::message::Message;
pub use prompts::multi_select::MultiSelect;
pub use prompts::number::Number;
pub use prompts::password::Password;
pub use prompts::select::Select;
pub use prompts::text::Text;
pub use prompts::toggle::Toggle;

pub use prompts::select::{SelectInput, SelectOption};
pub use prompts::text::LineInput;
pub use utils::key_listener::Typeable;
pub use utils::num_like::NumLike;
pub use utils::renderer::{DrawTime, Printable};

pub mod prelude {
    pub use super::{utils::renderer::Printable, Error, Promptable, SelectOption, Valuable};
    pub use super::{Confirm, Message, MultiSelect, Number, Password, Select, Text, Toggle};
}

#[cfg(feature = "terminal")]
mod terminal;

#[cfg(feature = "bevy")]
pub mod bevy;
#[cfg(feature = "bevy")]
mod text_style_adapter;

pub mod style;
