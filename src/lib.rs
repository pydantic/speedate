#![doc = include_str!("../README.md")]
extern crate strum;
use strum::{Display, EnumMessage};

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

/// Details about errors when parsing datetime, date, time & duration values
///
/// As well as comparing enum values, machine and human readable representations of
/// errors are provided.
///
/// # Examples
/// (Note: the `strum::EnumMessage` trait must be used to support `.get_documentation()`)
/// ```
/// use strum::EnumMessage;
/// use speedate::{Date, ParseError};
///
/// match Date::parse_str("invalid") {
///      Ok(_) => println!("Parsed successfully"),
///      Err(error) => {
///          assert_eq!(error, ParseError::TooShort);
///          assert_eq!(error.to_string(), "too_short");
///          assert_eq!(error.get_documentation(), Some("Input is too short"));
///      },
/// };
/// ```
#[derive(Debug, Display, EnumMessage, PartialEq, Eq, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum ParseError {
    /// Input is too short
    TooShort,
    /// Unexpected extra characters at the end of the input
    ExtraCharacters,
    /// Invalid datetime separator, expected `T`, `t`, `_` or space
    InvalidCharDateTimeSep,
    /// Invalid date separator, expected `-`
    InvalidCharDateSep,
    /// Invalid character in year
    InvalidCharYear,
    /// Invalid character in month
    InvalidCharMonth,
    /// Invalid character in day
    InvalidCharDay,
    /// Invalid time separator, expected `:`
    InvalidCharTimeSep,
    /// Invalid character in hour
    InvalidCharHour,
    /// Invalid character in minute
    InvalidCharMinute,
    /// Invalid character in second
    InvalidCharSecond,
    /// Invalid character in second fraction
    InvalidCharSecondFraction,
    /// Invalid timezone sign
    InvalidCharTzSign,
    /// Invalid timezone hour
    InvalidCharTzHour,
    /// Invalid timezone minute
    InvalidCharTzMinute,
    /// Month value is outside expected range of 1-12
    OutOfRangeMonth,
    /// Day value is outside expected range
    OutOfRangeDay,
    /// Hour value is outside expected range of 0-23
    OutOfRangeHour,
    /// Minute value is outside expected range of 0-59
    OutOfRangeMinute,
    /// Second value is outside expected range of 0-59
    OutOfRangeSecond,
    /// Second fraction value is more than 6 digits long
    SecondFractionTooLong,
    /// Second fraction digits missing after `.`
    SecondFractionMissing,
    /// Invalid digit in duration
    DurationInvalidNumber,
    /// `T` character repeated in duration
    DurationTRepeated,
    /// quantity fraction invalid in duration
    DurationInvalidFraction,
    /// quantity invalid in time part of duration
    DurationInvalidTimeUnit,
    /// quantity invalid in date part of duration
    DurationInvalidDateUnit,
    /// "day" identifier in duration not correctly formatted
    DurationInvalidDays,
    /// dates before 1600 are not supported as unix timestamps
    DateTooSmall,
    /// dates after XXXX in nanoseconds are not supported
    DateTooLarge,
    /// numeric times may not exceed 86,399 seconds
    TimeTooLarge,
}
