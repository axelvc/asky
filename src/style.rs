use std::fmt;
use std::io::{Error, Write};
use bitflags::bitflags;
use crossterm::{
    execute, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    Command,
};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Flags: u8 {
        const Focused  = 0b00000001;
        const Selected = 0b00000010;
        const Disabled = 0b00000100;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Section {
    Query(bool), // if answered -> Query(true)
    // Answered,
    Answer(bool), // if show -> Answer(true)
    DefaultAnswer,
    Message,
    Toggle(bool),  // if selected -> Toggle(true)
    Option(Flags), // if selected -> Toggle(true)
    OptionExclusive(Flags),
    List,
    ListItem(bool), // if first -> ListItem(true)
    Cursor,
    Placeholder,
    Validator(bool), // if valid -> Validator(true)
    Input,
    Page(u8, u8), // Page 0 of 8 -> Page(0, 8)
}

#[derive(Clone, Copy, Debug)]
pub enum Region {
    Begin(Section),
    End(Section),
    // Wrap(Section, Box<dyn Command>),
}

pub struct StyledRegion<T>(T, Region);

pub trait Style {
    fn begin(&self, section: Section) -> StyledRegion<Self>
    where
        Self: Sized + Clone,
    {
        StyledRegion(self.clone(), Region::Begin(section))
    }

    fn end(&self, section: Section) -> StyledRegion<Self>
    where
        Self: Sized + Clone,
    {
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

#[derive(Clone, Copy, Debug)]
pub struct DefaultStyle {
    pub ascii: bool,
}

impl Default for DefaultStyle {
    fn default() -> Self {
        Self { ascii: false }
    }
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
                }
                Answer(show) => {
                    SetForegroundColor(Color::Magenta).write_ansi(f)?;
                    if !show {
                        Print(if self.ascii { "..." } else { "…" }).write_ansi(f)?;
                    }
                } // was purple
                Toggle(selected) => {
                    if selected {
                        SetForegroundColor(Color::Black).write_ansi(f)?;
                        SetBackgroundColor(Color::Blue).write_ansi(f)?;
                        Print(" ").write_ansi(f)?;
                    } else {
                        SetForegroundColor(Color::White).write_ansi(f)?;
                        SetBackgroundColor(Color::DarkGrey).write_ansi(f)?;
                        Print(" ").write_ansi(f)?;
                    }
                }
                OptionExclusive(flags) => {
                    match (
                        flags.contains(Flags::Focused),
                        flags.contains(Flags::Disabled),
                    ) {
                        (false, _) => {
                            SetForegroundColor(Color::DarkGrey).write_ansi(f)?;
                            Print(if self.ascii { "( )" } else { "○" }).write_ansi(f)?;
                            SetForegroundColor(Color::Reset).write_ansi(f)?;
                        }
                        (true, true) => {
                            SetForegroundColor(Color::Red).write_ansi(f)?;
                            Print(if self.ascii { "( )" } else { "○" }).write_ansi(f)?;
                            SetForegroundColor(Color::Reset).write_ansi(f)?;
                        }
                        (true, false) => {
                            SetForegroundColor(Color::Blue).write_ansi(f)?;
                            Print(if self.ascii { "(x)" } else { "●" }).write_ansi(f)?;
                            SetForegroundColor(Color::Reset).write_ansi(f)?;
                        }
                    }
                    Print(" ").write_ansi(f)?;
                    match (
                        flags.contains(Flags::Focused),
                        flags.contains(Flags::Disabled),
                    ) {
                        (_, true) => {
                            SetForegroundColor(Color::DarkGrey).write_ansi(f)?;
                            SetAttribute(Attribute::OverLined).write_ansi(f)?;
                        }
                        (true, false) => {
                            SetForegroundColor(Color::Blue).write_ansi(f)?;
                        }
                        (false, false) => {}
                    }
                }
                Option(flags) => {
                    let prefix = match (
                        flags.contains(Flags::Selected),
                        flags.contains(Flags::Focused),
                    ) {
                        (true, true) => {
                            if self.ascii {
                                "(o)"
                            } else {
                                "◉"
                            }
                        }
                        (true, false) => {
                            if self.ascii {
                                "(x)"
                            } else {
                                "●"
                            }
                        }
                        _ => {
                            if self.ascii {
                                "( )"
                            } else {
                                "○"
                            }
                        }
                    };

                    match (
                        flags.contains(Flags::Focused),
                        flags.contains(Flags::Selected),
                        flags.contains(Flags::Disabled),
                    ) {
                        (true, _, true) => SetForegroundColor(Color::Red).write_ansi(f)?,
                        (true, _, false) => SetForegroundColor(Color::Blue).write_ansi(f)?,
                        (false, true, _) => SetForegroundColor(Color::Reset).write_ansi(f)?,
                        (false, false, _) => SetForegroundColor(Color::DarkGrey).write_ansi(f)?,
                    };
                    Print(prefix).write_ansi(f)?;
                    SetForegroundColor(Color::Reset).write_ansi(f)?;
                    Print(" ").write_ansi(f)?;
                    match (
                        flags.contains(Flags::Focused),
                        flags.contains(Flags::Disabled),
                    ) {
                        (_, true) => {
                            SetForegroundColor(Color::DarkGrey).write_ansi(f)?;
                            SetAttribute(Attribute::OverLined).write_ansi(f)?;
                        }
                        (true, false) => {
                            SetForegroundColor(Color::Blue).write_ansi(f)?;
                        }
                        (false, false) => {}
                    }
                }
                Message => {}
                Validator(valid) => {
                    SetForegroundColor(if valid { Color::Blue } else { Color::Red })
                        .write_ansi(f)?;
                }
                Placeholder => {
                    SetForegroundColor(Color::DarkGrey).write_ansi(f)?;
                    Print("Default: ").write_ansi(f)?;
                }
                Input => {
                    SetForegroundColor(Color::Blue).write_ansi(f)?;
                    Print(if self.ascii { ">" } else { "›" }).write_ansi(f)?;
                    SetForegroundColor(Color::Reset).write_ansi(f)?;
                    Print(" ").write_ansi(f)?;
                    // SetForegroundColor(Color::DarkGrey).write_ansi(f)?;
                }
                List => Print("[").write_ansi(f)?,
                ListItem(first) => {
                    if (!first) {
                        Print(", ").write_ansi(f)?;
                    }
                }
                Page(i, count) => {
                    if count != 1 {
                        let icon = if self.ascii { "*" } else { "•" };

                        Print("\n").write_ansi(f)?;
                        Print(" ".repeat(if self.ascii { 4 } else { 2 })).write_ansi(f)?;
                        SetForegroundColor(Color::DarkGrey).write_ansi(f)?;
                        Print(icon.repeat(i as usize)).write_ansi(f)?;
                        SetForegroundColor(Color::Reset).write_ansi(f)?;
                        Print(icon).write_ansi(f)?;
                        SetForegroundColor(Color::DarkGrey).write_ansi(f)?;
                        Print(icon.repeat(count.saturating_sub(i + 1) as usize)).write_ansi(f)?;
                        SetForegroundColor(Color::Reset).write_ansi(f)?;
                        Print("\n").write_ansi(f)?;
                    }
                }
                x => todo!("{:?} not impl", x),
            },
            End(section) => match section {
                Query(answered) => {
                    if answered {
                        Print(" ").write_ansi(f)?;
                    } else {
                        Print("\n").write_ansi(f)?;
                    }
                }
                Answer(_) => {
                    ResetColor.write_ansi(f)?;
                    Print("\n").write_ansi(f)?;
                }
                Toggle(_) => {
                    Print(" ").write_ansi(f)?;
                    ResetColor.write_ansi(f)?;
                    Print("  ").write_ansi(f)?;
                }
                OptionExclusive(flags) | Option(flags) => {
                    Print("\n").write_ansi(f)?;
                    ResetColor.write_ansi(f)?;
                }
                List => Print("]").write_ansi(f)?,
                ListItem(_) => {}
                Message => Print("\n").write_ansi(f)?,
                _ => ResetColor.write_ansi(f)?,
            },
        }
        Ok(())
    }
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
