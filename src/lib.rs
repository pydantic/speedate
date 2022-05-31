#![doc = include_str!("../README.md")]
mod date;
mod datetime;
mod duration;
mod time;

pub use date::Date;
pub use datetime::DateTime;
pub use duration::Duration;
pub use time::Time;

/// Parsing datetime, date, time & duration values

// get a character from the bytes as as a decimal
macro_rules! get_digit {
    ($bytes:ident, $index:expr, $error:ident) => {
        match $bytes.get($index) {
            Some(c) if (b'0'..=b'9').contains(&c) => c - b'0',
            _ => return Err(ParseError::$error),
        }
    };
}
pub(crate) use get_digit;

// as above without bounds check, requires length to checked first!
macro_rules! get_digit_unchecked {
    ($bytes:ident, $index:expr, $error:ident) => {
        match $bytes.get_unchecked($index) {
            c if (b'0'..=b'9').contains(&c) => c - b'0',
            _ => return Err(ParseError::$error),
        }
    };
}
pub(crate) use get_digit_unchecked;

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    TooShort,
    ExtraCharacters,
    InvalidCharDateTimeSep,
    InvalidCharDateSep,
    InvalidCharYear,
    InvalidCharMonth,
    InvalidCharDay,
    InvalidCharTimeSep,
    InvalidCharHour,
    InvalidCharMinute,
    InvalidCharSecond,
    InvalidCharSecondFraction,
    InvalidCharTzSign,
    InvalidCharTzHour,
    InvalidCharTzMinute,
    OutOfRangeMonth,
    OutOfRangeDay,
    OutOfRangeHour,
    OutOfRangeMinute,
    OutOfRangeSecond,
    SecondFractionTooLong,
    SecondFractionMissing,
    DurationInvalidNumber,
    DurationTRepeated,
    DurationInvalidFraction,
    DurationInvalidTimeUnit,
    DurationInvalidDateUnit,
    DurationInvalidDays,
}
