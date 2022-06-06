use std::fmt;

use crate::{get_digit_unchecked, ParseError};

/// A Date
///
/// Allowed formats:
/// * `YYYY-MM-DD`
///
/// Leap years are correct calculated according to the Gregorian calendar.
/// Thus `2000-02-29` is a valid date, but `2001-02-29` is not.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Date {
    /// Year: four digits
    pub year: u16,
    /// Month: 1 to 12
    pub month: u8,
    /// Day: 1 to {28, 29, 30, 31} (based on month & year)
    pub day: u8,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

// 2e10 if greater than this, the number is in ms, if less than or equal, it's in seconds
// (in seconds this is 11th October 2603, in ms it's 20th August 1970)
const MS_WATERSHED: i64 = 20_000_000_000;
// 1600-01-01 as a unix timestamp (omitting the negative) used for from_timestamp below
const UNIX_1600: i64 = 11_676_096_000;

impl Date {
    /// Parse a date from a string
    ///
    /// # Arguments
    ///
    /// * `str` - The string to parse
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::Date;
    ///
    /// let d = Date::parse_str("2020-01-01").unwrap();
    /// assert_eq!(
    ///     d,
    ///     Date {
    ///         year: 2020,
    ///         month: 1,
    ///         day: 1
    ///     }
    /// );
    /// assert_eq!(d.to_string(), "2020-01-01");
    /// ```
    #[inline]
    pub fn parse_str(str: &str) -> Result<Self, ParseError> {
        Self::parse_bytes(str.as_bytes())
    }

    /// Parse a date from bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to parse
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::Date;
    ///
    /// let d = Date::parse_bytes(b"2020-01-01").unwrap();
    /// assert_eq!(
    ///     d,
    ///     Date {
    ///         year: 2020,
    ///         month: 1,
    ///         day: 1
    ///     }
    /// );
    /// assert_eq!(d.to_string(), "2020-01-01");
    /// ```
    #[inline]
    pub fn parse_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let d = Self::parse_bytes_partial(bytes)?;

        if bytes.len() > 10 {
            return Err(ParseError::ExtraCharacters);
        }

        Ok(d)
    }

    pub fn from_timestamp(timestamp: i64) -> Result<Self, ParseError> {
        let (timestamp_second, _) = Self::timestamp_watershed(timestamp)?;
        Self::from_timestamp_calc(timestamp_second)
    }

    pub(crate) fn timestamp_watershed(timestamp: i64) -> Result<(i64, u32), ParseError> {
        let ts_abs = timestamp.abs();
        let (mut seconds, mut microseconds) = if ts_abs > MS_WATERSHED * 1_000_000 {
            // timestamp is in nanoseconds
            (timestamp / 1_000_000_000, timestamp % 1_000_000_000 / 1_000)
        } else if ts_abs > MS_WATERSHED * 1_000 {
            (timestamp / 1_000_000, timestamp % 1_000_000)
        } else if ts_abs > MS_WATERSHED {
            (timestamp / 1_000, timestamp % 1_000 * 1000)
        } else {
            (timestamp, 0)
        };
        if microseconds < 0 {
            seconds -= 1;
            microseconds += 1_000_000;
        }
        if seconds.abs() > MS_WATERSHED {
            return Err(ParseError::DateTooLarge);
        }
        Ok((seconds, microseconds as u32))
    }

    pub(crate) fn from_timestamp_calc(timestamp_second: i64) -> Result<Self, ParseError> {
        if timestamp_second < -UNIX_1600 {
            return Err(ParseError::DateTooSmall);
        }
        let seconds_diff = UNIX_1600 + timestamp_second;
        let delta_days = seconds_diff / 86_400;
        let delta_years = delta_days / 365;
        let leap_years = if delta_years == 0 {
            0
        } else {
            // plus one because 1600 itself was a leap year
            (delta_years - 1) / 4 - (delta_years - 1) / 100 + (delta_years - 1) / 400 + 1
        };

        // year day is the day of the year, starting from 1
        let mut year_day: i16 = (delta_days % 365 - leap_years + 1) as i16;
        let mut year: u16 = (1_600 + delta_years) as u16;
        let mut leap_year: bool = if year % 100 == 0 {
            year % 400 == 0
        } else {
            year % 4 == 0
        };
        if year_day < 1 {
            year -= 1;
            leap_year = year % 4 == 0;
            year_day += if leap_year { 366 } else { 365 };
        }
        let (month, day) = match leap_year {
            true => leap_year_month_day(year_day),
            false => non_leap_year_month_day(year_day),
        };
        Ok(Self { year, month, day })
    }

    /// Parse a date from bytes, no check is performed for extract characters at the end of the string
    pub(crate) fn parse_bytes_partial(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() < 10 {
            return Err(ParseError::TooShort);
        }
        let year: u16;
        let month: u8;
        let day: u8;
        unsafe {
            let y1 = get_digit_unchecked!(bytes, 0, InvalidCharYear) as u16;
            let y2 = get_digit_unchecked!(bytes, 1, InvalidCharYear) as u16;
            let y3 = get_digit_unchecked!(bytes, 2, InvalidCharYear) as u16;
            let y4 = get_digit_unchecked!(bytes, 3, InvalidCharYear) as u16;
            year = y1 * 1000 + y2 * 100 + y3 * 10 + y4;

            match bytes.get_unchecked(4) {
                b'-' => (),
                _ => return Err(ParseError::InvalidCharDateSep),
            }

            let m1 = get_digit_unchecked!(bytes, 5, InvalidCharMonth);
            let m2 = get_digit_unchecked!(bytes, 6, InvalidCharMonth);
            month = m1 * 10 + m2;

            match bytes.get_unchecked(7) {
                b'-' => (),
                _ => return Err(ParseError::InvalidCharDateSep),
            }

            let d1 = get_digit_unchecked!(bytes, 8, InvalidCharDay);
            let d2 = get_digit_unchecked!(bytes, 9, InvalidCharDay);
            day = d1 * 10 + d2;
        }

        // calculate the maximum number of days in the month, accounting for leap years in the
        // gregorian calendar
        let max_days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => return Err(ParseError::OutOfRangeMonth),
        };

        if day < 1 || day > max_days {
            return Err(ParseError::OutOfRangeDay);
        }

        Ok(Self { year, month, day })
    }
}

fn leap_year_month_day(day: i16) -> (u8, u8) {
    match day {
        0..=31 => (1, day as u8),
        32..=60 => (2, day as u8 - 31),
        61..=91 => (3, day as u8 - 60),
        92..=121 => (4, day as u8 - 91),
        122..=152 => (5, day as u8 - 121),
        153..=182 => (6, day as u8 - 152),
        183..=213 => (7, day as u8 - 182),
        214..=244 => (8, day as u8 - 213),
        245..=274 => (9, (day - 244) as u8),
        275..=305 => (10, (day - 274) as u8),
        306..=335 => (11, (day - 305) as u8),
        _ => (12, (day - 335) as u8),
    }
}

fn non_leap_year_month_day(day: i16) -> (u8, u8) {
    match day {
        0..=31 => (1, day as u8),
        32..=59 => (2, day as u8 - 31),
        60..=90 => (3, day as u8 - 59),
        91..=120 => (4, day as u8 - 90),
        121..=151 => (5, day as u8 - 120),
        152..=181 => (6, day as u8 - 151),
        182..=212 => (7, day as u8 - 181),
        213..=243 => (8, day as u8 - 212),
        244..=273 => (9, (day - 243) as u8),
        274..=304 => (10, (day - 273) as u8),
        305..=334 => (11, (day - 304) as u8),
        _ => (12, (day - 334) as u8),
    }
}
