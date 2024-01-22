// use termcolor::{Color, ColorSpec, WriteColor};
use crossterm::{queue, execute, Command, {style::{Color, SetForegroundColor, SetBackgroundColor, Print, ResetColor}}};
use std::io::{Error,Write};
use std::fmt;

// pub struct Style {
//     pub ascii: bool,
// }

pub trait Style {
    fn begin(&self, section: Section) -> StyledRegion<Self> where Self: Sized + Clone{
        StyledRegion(self.clone(), Region::Begin(section))
    }

    fn end(&self, section: Section) -> StyledRegion<Self> where Self: Sized + Clone{
        StyledRegion(self.clone(), Region::End(section))
    }

    // fn wrap(&self, command: dyn Command, section: Section) -> StyledRegion<Self> where Self: Sized + Clone{
    //     StyledRegion(self.clone(), Region::Wrap(section, Box::new(command)))
    // }

    // fn wrap(&self, section: Section, command: Box<dyn Command>) -> StyledRegion<Self> where Self: Sized + Clone{
    //     StyledRegion(self.clone(), Region::End(section))
    // }

    fn write_ansi(&self, group: Region, f: &mut impl fmt::Write) -> fmt::Result;
}

impl Style for DefaultStyle {
    fn write_ansi(&self, group: Region, f: &mut impl fmt::Write) -> fmt::Result {
        use Region::*;
        use Section::*;
        match group {
            Begin(section) => match section {
                Query(answered) => {
                    if answered {
                        SetForegroundColor(Color::Green).write_ansi(f)?;
                        Print(if self.ascii { "[x]" } else { "■" }).write_ansi(f)?;
                        SetForegroundColor(Color::Reset).write_ansi(f)?;
                        Print(" ").write_ansi(f)?;
                    } else {
                        SetForegroundColor(Color::Blue).write_ansi(f)?;
                        Print(if self.ascii { "[ ]" } else { "▣" }).write_ansi(f)?;
                        SetForegroundColor(Color::Reset).write_ansi(f)?;
                        Print(" ").write_ansi(f)?;
                    }
                },
                Answer => SetForegroundColor(Color::Magenta).write_ansi(f)?, // was purple
                Option(selected) =>
                    if selected {
                        SetForegroundColor(Color::Black).write_ansi(f)?;
                        SetBackgroundColor(Color::Blue).write_ansi(f)?;
                        Print(" ").write_ansi(f)?;
                    } else {
                        SetForegroundColor(Color::White).write_ansi(f)?;
                        SetBackgroundColor(Color::DarkGrey).write_ansi(f)?;
                        Print(" ").write_ansi(f)?;
                    },
                Message => {},
                Validator(valid) => {
                    SetForegroundColor(if valid { Color::Blue } else { Color::Red }).write_ansi(f)?;
                    Print(if self.ascii { ">" } else { "›" }).write_ansi(f)?;
                    SetForegroundColor(Color::Reset).write_ansi(f)?;
                    Print(" ").write_ansi(f)?;
                },
                Placeholder => {
                    SetForegroundColor(Color::DarkGrey).write_ansi(f)?;
                    Print("Default: ").write_ansi(f)?;
                },
                Input => {
                    // SetForegroundColor(Color::DarkGrey).write_ansi(f)?;
                },
                x => todo!("{:?} not impl", x),
                },
            End(section) => match section {
                Query(answered) => if answered {
                    Print(" ").write_ansi(f)?;
                } else {
                    Print("\n").write_ansi(f)?;
                },
                Answer => {
                    ResetColor.write_ansi(f)?;
                    Print("\n").write_ansi(f)?;
                },
                Option(_) => {
                    Print(" ").write_ansi(f)?;
                    ResetColor.write_ansi(f)?;
                    Print("  ").write_ansi(f)?;
                },
                Message => Print("\n").write_ansi(f)?,
                _ => ResetColor.write_ansi(f)?,
            },
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Region {
    Begin(Section),
    End(Section),
    // Wrap(Section, Box<dyn Command>),
}

pub struct StyledRegion<T>(T, Region);

#[derive(Clone, Copy, Debug)]
pub enum Section {
    Query(bool), // if answered -> Query(true)
    // Answered,
    Answer,
    DefaultAnswer,
    Message,
    Option(bool), // if selected -> Option(true)
    OptionExclusive(bool),
    List,
    ListItem,
    Cursor,
    Placeholder,
    Validator(bool), // if valid -> Validator(true)
    Input,
}

impl<T: Style> Command for StyledRegion<T> {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        self.0.write_ansi(self.1, f)
    }
}

// impl Command for Region {
//     fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
//         match self {
//             Region::Begin(section) => match section {
//                 Section::Query => {
//                     SetForegroundColor(Color::Blue).write_ansi(f)?;
//                     // Print(if self.ascii { "[ ]" } else { "▣" }).write_ansi(f)?;
//                     Print("▣").write_ansi(f)?;
//                     SetForegroundColor(Color::Reset).write_ansi(f)?;
//                     Print(" ").write_ansi(f)?;
//                 },
//                 _ => todo!(),
//             },
//             Region::End(section) => match section {
//                 Section::Query => {
//                     Print("\n").write_ansi(f)?;
//                 },
//                 _ => todo!(),
//             },
//         }
//         Ok(())
//     }
// }

#[derive(Clone, Copy, Debug)]
pub struct DefaultStyle {
    pub ascii: bool,
}

impl Default for DefaultStyle {
    fn default() -> Self {
        Self {
            ascii: false,
        }
    }
}

// pub trait Style<T,E> {
//     fn begin(&mut self, writer: &mut T, section: Section) -> Result<(), E>;
//     fn end(&mut self, writer: &mut T) -> Result<(), E>;
// }

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

// impl<T: Write> Style<T, Error> for DefaultStyle {
//     fn begin(&mut self, writer: &mut T, section: Section) -> Result<(), Error> {
//         match &section {
//             Section::Query => {
//                 queue!(writer,
//                        SetForegroundColor(Color::Blue),
//                        Print(if self.ascii { "[ ]" } else { "▣" }),
//                        SetForegroundColor(Color::Reset),
//                        Print(" "))?;
//             },
//             Section::Answered => {
//                 queue!(writer, SetForegroundColor(Color::Green))?;
//                 if self.ascii {
//                     write!(writer, "[x] ")?;
//                 } else {
//                     write!(writer, "■ ")?;
//                 }
//                 queue!(writer, SetForegroundColor(Color::Reset))?;
//             },
//             Section::Answer => queue!(writer, SetForegroundColor(Color::Magenta))?, // was purple
//             Section::Option(selected) =>
//                 if *selected {
//                     queue!(writer,
//                              SetForegroundColor(Color::Black),
//                              SetBackgroundColor(Color::Blue),
//                              Print(" "))?;
//                 } else {
//                     queue!(writer,
//                              SetForegroundColor(Color::White),
//                              SetBackgroundColor(Color::DarkGrey), // bright black
//                              Print(" "))?;
//                 },
//             _ => todo!(),
//         }
//         self.stack.push(section);
//         Ok(())
//     }

//     fn end(&mut self, writer: &mut T) -> Result<(),Error> {
//         // match self.stack.pop().expect("No matching begin") {
//         match self.stack.pop().unwrap_or(Section::OptionExclusive(false)) {
//             Section::Query(answered) => queue!(writer, Print(if answered { " " } else { "\n" }))?,
//             Section::Answer => queue!(writer, ResetColor, Print("\n"))?,
//             // Section::Answered => queue!(writer, Print(" "))?,
//             Section::Option(_) => queue!(writer, Print(" "), ResetColor, Print("  "))?,
//             // Section::Answer => queue!(writer, ResetColor)?,
//             // _ => todo!(),
//             _ => queue!(writer, ResetColor)?,
//         }
//         Ok(())
//     }
// }

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
