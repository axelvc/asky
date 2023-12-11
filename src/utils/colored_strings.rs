use colored::*;
use std::ops::{Deref, DerefMut, Range};
use std::fmt;

/// A collection of colored strings. It can be used like a `Vec<ColoredString>`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ColoredStrings(pub Vec<ColoredString>);

impl<'a> Deref for ColoredStrings {
    type Target = Vec<ColoredString>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ColoredStrings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ColoredStrings {
    /// Return a new empty ColoredStrings container.
    pub fn new() -> Self {
        ColoredStrings::default()
    }

    // /// Split a ColoredStrings by a character.
    // pub fn split(self, pat: char) -> Vec<ColoredStrings> {
    //     let mut accum: Vec<ColoredStrings> = Vec::new();
    //     accum.push(ColoredStrings::default());
    //     for colored_string in self.0 {
    //         let mut i = Self::colored_string_split(colored_string, pat);
    //         if let Some(first) = i.next() {
    //             accum.last_mut().unwrap().push(first);
    //         }
    //         accum.extend(i.map(|s| {
    //             ColoredStrings(vec![s])
    //         }));
    //     }
    //     accum
    // }

    // // We can use Pattern once it becomes stable.
    // // fn split<P: Pattern<'a>>(&'a self, pat: P) -> impl Iterator<Item = ColoredString<'a>>{
    // fn colored_string_split(cs: ColoredString, pat: char) -> impl Iterator<Item = ColoredString> {
    //     let mut a = None;
    //     let mut b = None;
    //     let input = format!("{}", cs);
    //     if input.find(pat).is_none() {
    //         a = Some(cs);
    //     } else {
    //         let i = input.clone();
    //         b = Some(input.split(pat).map(move |s| ColoredString {
    //             input: s.to_string(),
    //             .. cs
    //         }).collect::<Vec<_>>());
    //     }
    //     // Here's how you coalesce two iterators, only one of which has content.
    //     a.into_iter().chain(b.into_iter().flatten())
    // }
}

impl fmt::Display for ColoredStrings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for s in &self.0 {
            s.fmt(f)?
        }
        Ok(())
    }
}
