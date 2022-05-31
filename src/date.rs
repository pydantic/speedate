use std::fmt;

use crate::{get_digit_unchecked, ParseError};

/// A Date
///
/// Allowed formats:
/// * `YYYY-MM-DD`
///
/// Leap years are correct calculated according to the Gregorian calendar.
/// Thus `2000-02-29` is a valid date, but `2001-02-29` is not.
#[derive(Debug, PartialEq, Clone)]
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
