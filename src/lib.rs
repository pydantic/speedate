#![doc = include_str!("../README.md")]
extern crate core;
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
///          assert_eq!(error.get_documentation(), Some("input is too short"));
///      },
/// };
/// ```
#[derive(Debug, Display, EnumMessage, PartialEq, Eq, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum ParseError {
    /// input is too short
    TooShort,
    /// unexpected extra characters at the end of the input
    ExtraCharacters,
    /// invalid datetime separator, expected `T`, `t`, `_` or space
    InvalidCharDateTimeSep,
    /// invalid date separator, expected `-`
    InvalidCharDateSep,
    /// invalid character in year
    InvalidCharYear,
    /// invalid character in month
    InvalidCharMonth,
    /// invalid character in day
    InvalidCharDay,
    /// invalid time separator, expected `:`
    InvalidCharTimeSep,
    /// invalid character in hour
    InvalidCharHour,
    /// invalid character in minute
    InvalidCharMinute,
    /// invalid character in second
    InvalidCharSecond,
    /// invalid character in second fraction
    InvalidCharSecondFraction,
    /// invalid timezone sign
    InvalidCharTzSign,
    /// invalid timezone hour
    InvalidCharTzHour,
    /// invalid timezone minute
    InvalidCharTzMinute,
    /// timezone minute value is outside expected range of 0-59
    OutOfRangeTzMinute,
    /// timezone offset must be less than 24 hours
    OutOfRangeTz,
    /// timezone is required to adjust to a new timezone
    TzRequired,
    /// Error getting system time
    SystemTimeError,
    /// month value is outside expected range of 1-12
    OutOfRangeMonth,
    /// day value is outside expected range
    OutOfRangeDay,
    /// hour value is outside expected range of 0-23
    OutOfRangeHour,
    /// minute value is outside expected range of 0-59
    OutOfRangeMinute,
    /// second value is outside expected range of 0-59
    OutOfRangeSecond,
    /// second fraction value is more than 6 digits long
    SecondFractionTooLong,
    /// second fraction digits missing after `.`
    SecondFractionMissing,
    /// invalid digit in duration
    DurationInvalidNumber,
    /// `t` character repeated in duration
    DurationTRepeated,
    /// quantity fraction invalid in duration
    DurationInvalidFraction,
    /// quantity invalid in time part of duration
    DurationInvalidTimeUnit,
    /// quantity invalid in date part of duration
    DurationInvalidDateUnit,
    /// "day" identifier in duration not correctly formatted
    DurationInvalidDays,
    /// a numeric value in the duration is too large
    DurationValueTooLarge,
    /// durations may not exceed 999,999,999 days
    DurationDaysTooLarge,
    /// dates before 1600 are not supported as unix timestamps
    DateTooSmall,
    /// dates after 9999 are not supported as unix timestamps
    DateTooLarge,
    /// numeric times may not exceed 86,399 seconds
    TimeTooLarge,
}
