// use termcolor::{Color, ColorSpec, WriteColor};
use crossterm::{queue, execute, Command, {style::{Color, SetForegroundColor, SetBackgroundColor, Print, ResetColor}}};
use std::io::{Error,Write};
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct MyStyle {
    pub ascii: bool,
}

pub trait Style2 {
    fn begin(&self, section: Section) -> StyledGroup<Self> where Self: Sized + Clone{
        StyledGroup(self.clone(), Group::Begin(section))
    }

    fn end(&self, section: Section) -> StyledGroup<Self> where Self: Sized + Clone{
        StyledGroup(self.clone(), Group::End(section))
    }

    // fn wrap(&self, command: dyn Command, section: Section) -> StyledGroup<Self> where Self: Sized + Clone{
    //     StyledGroup(self.clone(), Group::Wrap(section, Box::new(command)))
    // }

    // fn wrap(&self, section: Section, command: Box<dyn Command>) -> StyledGroup<Self> where Self: Sized + Clone{
    //     StyledGroup(self.clone(), Group::End(section))
    // }

    fn write_ansi(&self, group: Group, f: &mut impl fmt::Write) -> fmt::Result;
}

impl Style2 for MyStyle {
    fn write_ansi(&self, group: Group, f: &mut impl fmt::Write) -> fmt::Result {
        match group {
            Group::Begin(section) => match section {
                Section::Query => {
                    SetForegroundColor(Color::Blue).write_ansi(f)?;
                    Print(if self.ascii { "[ ]" } else { "▣" }).write_ansi(f)?;
                    // Print("▣").write_ansi(f)?;
                    SetForegroundColor(Color::Reset).write_ansi(f)?;
                    Print(" ").write_ansi(f)?;
                },
                _ => todo!(),
            },
            Group::End(section) => match section {
                Section::Query => {
                    Print("\n").write_ansi(f)?;
                },
                _ => todo!(),
            },
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Group {
    Begin(Section),
    End(Section),
    // Wrap(Section, Box<dyn Command>),
}

pub struct StyledGroup<T>(T, Group);

#[derive(Clone, Copy, Debug)]
pub enum Section {
    Query,
    Answered,
    Answer,
    Option(bool), // if selected -> Option(true)
    OptionExclusive(bool),
    List,
    ListItem,
    Cursor,
}

impl<T: Style2> Command for StyledGroup<T> {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        self.0.write_ansi(self.1, f)
    }
}

impl Command for Group {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        match self {
            Group::Begin(section) => match section {
                Section::Query => {
                    SetForegroundColor(Color::Blue).write_ansi(f)?;
                    // Print(if self.ascii { "[ ]" } else { "▣" }).write_ansi(f)?;
                    Print("▣").write_ansi(f)?;
                    SetForegroundColor(Color::Reset).write_ansi(f)?;
                    Print(" ").write_ansi(f)?;
                },
                _ => todo!(),
            },
            Group::End(section) => match section {
                Section::Query => {
                    Print("\n").write_ansi(f)?;
                },
                _ => todo!(),
            },
        }
        Ok(())
    }
}

pub struct DefaultStyle {
    ascii: bool,
    stack: Vec<Section>
}

impl Default for DefaultStyle {
    fn default() -> Self {
        Self {
            ascii: false,
            stack: Vec::new()
        }
    }
}

pub trait Style<T,E> {
    fn begin(&mut self, writer: &mut T, section: Section) -> Result<(), E>;
    fn end(&mut self, writer: &mut T) -> Result<(), E>;
}

// impl<T: WriteColor> Style<T, Error> for DefaultStyle {
//     fn begin(&mut self, writer: &mut T, section: Section) -> Result<(), Error> {
//         match &section {
//             Section::Query => {
//                 writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
//                 if self.ascii {
//                     write!(writer, "[x]")?;
//                 } else {
//                     write!(writer, "▣")?;
//                 }
//                 writer.reset()?;
//             },
//             _ => todo!(),
//         }
//         self.stack.push(section);
//         Ok(())
//     }

//     fn end(&mut self, writer: &mut T) -> Result<(),Error> {
//         match self.stack.pop().expect("No matching begin") {
//             Section::Query => write!(writer, "\n")?,
//             _ => todo!(),
//         }
//         Ok(())
//     }
// }

impl<T: Write> Style<T, Error> for DefaultStyle {
    fn begin(&mut self, writer: &mut T, section: Section) -> Result<(), Error> {
        match &section {
            Section::Query => {
                queue!(writer,
                       SetForegroundColor(Color::Blue),
                       Print(if self.ascii { "[ ]" } else { "▣" }),
                       SetForegroundColor(Color::Reset),
                       Print(" "))?;
            },
            Section::Answered => {
                queue!(writer, SetForegroundColor(Color::Green))?;
                if self.ascii {
                    write!(writer, "[x] ")?;
                } else {
                    write!(writer, "■ ")?;
                }
                queue!(writer, SetForegroundColor(Color::Reset))?;
            },
            Section::Answer => queue!(writer, SetForegroundColor(Color::Magenta))?, // was purple
            Section::Option(selected) =>
                if *selected {
                    queue!(writer,
                             SetForegroundColor(Color::Black),
                             SetBackgroundColor(Color::Blue),
                             Print(" "))?;
                } else {
                    queue!(writer,
                             SetForegroundColor(Color::White),
                             SetBackgroundColor(Color::DarkGrey), // bright black
                             Print(" "))?;
                },
            _ => todo!(),
        }
        self.stack.push(section);
        Ok(())
    }

    fn end(&mut self, writer: &mut T) -> Result<(),Error> {
        // match self.stack.pop().expect("No matching begin") {
        match self.stack.pop().unwrap_or(Section::OptionExclusive(false)) {
            Section::Query => queue!(writer, Print("\n"))?,
            Section::Answer => queue!(writer, ResetColor, Print("\n"))?,
            Section::Answered => queue!(writer, Print(" "))?,
            Section::Option(_) => queue!(writer, Print(" "), ResetColor, Print("  "))?,
            // Section::Answer => queue!(writer, ResetColor)?,
            // _ => todo!(),
            _ => queue!(writer, ResetColor)?,
        }
        Ok(())
    }
}

// impl Styler for ColoredStrings {

//     fn message(&mut self, writer: &mut WriteColor) -> Result<()> {
//         writer.set_color(ColorSpec::new().set_fg(&Some(Color::Blue)));
//         write!(writer, "▣");
//         writer.reset();
//         write!(writer, " {}", );
//         self.extend(
//         out.extend(["▣".blue(), " ".into(), message.into()]);
//     }
// }
