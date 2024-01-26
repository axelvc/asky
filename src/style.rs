use std::fmt;
use bitflags::bitflags;
use crossterm::{
    // queue,
    execute,
    QueueableCommand,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    Command,
};
use crate::utils::renderer::Renderer;
use std::io;
use text_style::{AnsiColor::*};
use core::iter::repeat;

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
    Option(Flags),
    OptionExclusive(Flags),
    List,
    ListItem(bool), // if first -> ListItem(true)
    // Cursor,
    Placeholder,
    Validator(bool), // if valid -> Validator(true)
    Input,
    Page(u8, u8), // Page 0 of 8 -> Page(0, 8)
    Custom(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub enum Region {
    Begin(Section),
    End(Section),
}

pub trait Style2 {
    fn begin<R: Renderer>(&self, renderer: &mut R, section: Section) -> io::Result<()>;
    fn end<R: Renderer>(&self, renderer: &mut R, section: Section) -> io::Result<()>;
}

impl Style2 for DefaultStyle {
    fn begin<R: Renderer>(&self, r: &mut R, section: Section) -> io::Result<()> {
        use Section::*;
        match section {
            Query(answered) => {
                if answered {
                    r.set_foreground(Green.dark())?;
                    write!(r, "{}", if self.ascii { "[x]" } else { "■" })?;
                    r.reset_color()?;
                    write!(r, " ")?;
                } else {
                    r.set_foreground(Blue.dark())?;
                    write!(r, "{}", if self.ascii { "[ ]" } else { "▣" })?;
                    r.reset_color()?;
                    write!(r, " ")?;
                }
            }
            Answer(show) => {
                r.set_foreground(Magenta.dark())?;
                if !show {
                    write!(r, "{}", if self.ascii { "..." } else { "…" })?;
                }
            } // was purple
            Toggle(selected) => {
                if selected {
                    r.set_foreground(Black.dark())?;
                    r.set_background(Blue.dark())?;
                    write!(r, " ")?;
                } else {
                    r.set_foreground(White.dark())?;
                    r.set_background(Black.light())?;
                    write!(r, " ")?;
                }
            }
            OptionExclusive(flags) => {
                match (
                    flags.contains(Flags::Focused),
                    flags.contains(Flags::Disabled),
                ) {
                    (false, _) => {
                        r.set_foreground(Black.light())?;
                        write!(r, "{}", if self.ascii { "( )" } else { "○" })?;
                        r.reset_color()?;
                    }
                    (true, true) => {
                        r.set_foreground(Red.dark())?;
                        write!(r, "{}", if self.ascii { "( )" } else { "○" })?;
                        r.reset_color()?;
                    }
                    (true, false) => {
                        r.set_foreground(Blue.dark())?;
                        write!(r, "{}", if self.ascii { "(x)" } else { "●" })?;
                        r.reset_color()?;
                    }
                }
                write!(r, " ")?;
                match (
                    flags.contains(Flags::Focused),
                    flags.contains(Flags::Disabled),
                ) {
                    (_, true) => {
                        r.set_foreground(Black.light())?;
                        // SetAttribute(Attribute::OverLined).write_ansi(f)?;
                    }
                    (true, false) => {
                        r.set_foreground(Blue.dark())?;
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
                    (true, _, true) => r.set_foreground(Red.dark())?,
                    (true, _, false) => r.set_foreground(Blue.dark())?,
                    (false, true, _) => r.reset_color()?,
                    (false, false, _) => r.set_foreground(Black.light())?,
                };
                write!(r, "{}", prefix)?;
                r.reset_color()?;
                write!(r, " ")?;
                match (
                    flags.contains(Flags::Focused),
                    flags.contains(Flags::Disabled),
                ) {
                    (_, true) => {
                        r.set_foreground(Black.light())?;
                        // SetAttribute(Attribute::OverLined).write_ansi(f)?;
                    }
                    (true, false) => {
                        r.set_foreground(Blue.dark())?;
                    }
                    (false, false) => {}
                }
            }
            Message => {}
            Validator(valid) => {
                r.set_foreground(if valid { Blue.dark() } else { Red.dark() })?;
            }
            Placeholder => {
                r.set_foreground(Black.light())?;
                write!(r, "Default: ")?;
            }
            Input => {
                r.set_foreground(Blue.dark())?;
                write!(r, "{}", if self.ascii { ">" } else { "›" })?;
                r.reset_color()?;
                write!(r, " ")?;
            }
            List => write!(r, "[")?,
            ListItem(first) => {
                if !first {
                    write!(r, ", ")?;
                }
            }
            Page(i, count) => {
                if count != 1 {
                    let icon = if self.ascii { "*" } else { "•" };
                    write!(r, "\n")?;
                    write!(r, "{}", " ".repeat(if self.ascii { 4 } else { 2 }))?;
                    r.set_foreground(Black.light())?;
                    write!(r, "{}", icon.repeat(i as usize))?;
                    r.reset_color()?;
                    write!(r, "{}", icon)?;
                    r.set_foreground(Black.light())?;
                    write!(r, "{}", icon.repeat(count.saturating_sub(i + 1) as usize))?;
                    r.reset_color()?;
                    write!(r, "\n")?;
                }
            }
            x => todo!("{:?} not impl", x),
            // x => {},
        }
        Ok(())
    }
    fn end<R: Renderer>(&self, r: &mut R, section: Section) -> io::Result<()> {
        use Section::*;
        match section {
            Query(answered) => {
                if answered {
                    write!(r, " ")?;
                } else {
                    write!(r, "\n")?;
                }
            }
            Answer(_) => {
                r.reset_color()?;
                write!(r, "\n")?;
            }
            Toggle(_) => {
                write!(r, " ")?;
                r.reset_color()?;
                write!(r, "  ")?;
            }
            OptionExclusive(_flags) | Option(_flags) => {
                write!(r, "\n")?;
                r.reset_color()?;
            }
            List => write!(r, "]")?,
            ListItem(_) => {}
            Message => write!(r, "\n")?,
            _ => r.reset_color()?,
        }
        Ok(())
    }
}

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

    // fn wrap(&self, section: Section, command: Box<dyn Command>) -> StyledRegion<Self> where Self: Sized + Clone{
    //     StyledRegion(self.clone(), Region::End(section))
    // }

    fn write_ansi(&self, group: Region, f: &mut impl fmt::Write) -> fmt::Result;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultStyle {
    pub ascii: bool,
}

pub struct StyledRegion<T>(T, Region);

impl<T: Style> Command for StyledRegion<T> {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        self.0.write_ansi(self.1, f)
    }
}


// pub struct Help;

// impl Help {
//     fn write_ansi(&mut self, group: Region, f: &mut (impl QueueableCommand + std::io::Write)) -> std::io::Result<&mut Self> {
//         use Region::*;
//         use Section::*;
//         match group {
//             Begin(section) => match section {
//                 Query(answered) => {
//                     if answered {
//                         queue!(f,
//                             SetForegroundColor(Color::Green))?;

//                         // f.queue(
//                     }
//                 }
//                 _ => todo!(),
//             }
//             _ => todo!(),
//         }
//         Ok(self)
//     }
// }

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
                    if !first {
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
                OptionExclusive(_flags) | Option(_flags) => {
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

