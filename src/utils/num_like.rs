use std::{fmt::Display, str::FromStr};

/// A utility trait to allow only numbers in [`Number`] prompt.
/// Also allows to custom handle they based on the type.
///
/// [`Number`]: crate::Number
pub trait NumLike: Default + Display + FromStr + Send + Copy {
    /// Check if it is a floating point number.
    fn is_float() -> bool {
        false
    }

    /// Check if it is a signed number.
    fn is_signed() -> bool {
        false
    }
}

impl NumLike for u8 {}
impl NumLike for u16 {}
impl NumLike for u32 {}
impl NumLike for u64 {}
impl NumLike for u128 {}
impl NumLike for usize {}

impl NumLike for i8 {
    fn is_signed() -> bool {
        true
    }
}

impl NumLike for i16 {
    fn is_signed() -> bool {
        true
    }
}

impl NumLike for i32 {
    fn is_signed() -> bool {
        true
    }
}

impl NumLike for i64 {
    fn is_signed() -> bool {
        true
    }
}

impl NumLike for i128 {
    fn is_signed() -> bool {
        true
    }
}

impl NumLike for isize {
    fn is_signed() -> bool {
        true
    }
}

impl NumLike for f32 {
    fn is_signed() -> bool {
        true
    }

    fn is_float() -> bool {
        true
    }
}

impl NumLike for f64 {
    fn is_signed() -> bool {
        true
    }

    fn is_float() -> bool {
        true
    }
}
