use crate::utils::renderer::Renderer;
use bitflags::bitflags;
use std::io;
use text_style::AnsiColor::*;

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
    Toggle(bool), // if selected -> Toggle(true)
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

pub trait Style {
    fn begin<R: Renderer>(&self, renderer: &mut R, section: Section) -> io::Result<()>;
    fn end<R: Renderer>(&self, renderer: &mut R, section: Section) -> io::Result<()>;
}

impl Style for DefaultStyle {
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

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultStyle {
    pub ascii: bool,
}
