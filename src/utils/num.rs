use std::{fmt::Display, str::FromStr};

/// A utilitiy trait to allow only number types in [`crate::prompts::number::Number`] prompt
pub trait Num: Default + Display + FromStr {
    fn is_float() -> bool {
        false
    }

    fn is_signed() -> bool {
        false
    }
}

impl Num for u8 {}
impl Num for u16 {}
impl Num for u32 {}
impl Num for u64 {}
impl Num for u128 {}
impl Num for usize {}

impl Num for i8 {
    fn is_signed() -> bool {
        true
    }
}

impl Num for i16 {
    fn is_signed() -> bool {
        true
    }
}

impl Num for i32 {
    fn is_signed() -> bool {
        true
    }
}

impl Num for i64 {
    fn is_signed() -> bool {
        true
    }
}

impl Num for i128 {
    fn is_signed() -> bool {
        true
    }
}

impl Num for isize {
    fn is_signed() -> bool {
        true
    }
}

impl Num for f32 {
    fn is_signed() -> bool {
        true
    }

    fn is_float() -> bool {
        true
    }
}

impl Num for f64 {
    fn is_signed() -> bool {
        true
    }

    fn is_float() -> bool {
        true
    }
}
