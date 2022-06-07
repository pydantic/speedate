use std::fmt;

use crate::{get_digit, Date, ParseError, Time};

/// A DateTime
///
/// Combines a `Date`, `Time` and optionally a timezone offset in minutes.
/// Allowed values:
/// * `YYYY-MM-DDTHH:MM:SS` - all the above time formats are allowed for the time part
/// * `YYYY-MM-DD HH:MM:SS` - `T`, `t`, ` ` and `_` are allowed as separators
/// * `YYYY-MM-DDTHH:MM:SSZ` - `Z` or `z` is allowed as timezone
/// * `YYYY-MM-DDTHH:MM:SS+08:00`- positive and negative timezone are allowed,
///   as per ISO 8601, U+2212 minus `−` is allowed as well as ascii minus `-` (U+002D)
/// * `YYYY-MM-DDTHH:MM:SS+0800` - the colon (`:`) in the timezone is optional
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DateTime {
    /// date part of the datetime
    pub date: Date,
    /// time part of the datetime
    pub time: Time,
    /// timezone offset in minutes if provided
    pub offset: Option<i16>,
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}T{}", self.date, self.time)?;
        if let Some(offset) = self.offset {
            if offset == 0 {
                write!(f, "Z")?;
            } else {
                write!(f, "{:+03}:{:02}", offset / 60, (offset % 60).abs())?;
            }
        }
        Ok(())
    }
}

impl DateTime {
    /// Parse a datetime from a string
    ///
    /// # Arguments
    ///
    /// * `str` - The string to parse
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::{DateTime, Date, Time};
    ///
    /// let dt = DateTime::parse_str("2022-01-01T12:13:14Z").unwrap();
    /// assert_eq!(
    ///     dt,
    ///     DateTime {
    ///         date: Date {
    ///             year: 2022,
    ///             month: 1,
    ///             day: 1,
    ///         },
    ///         time: Time {
    ///             hour: 12,
    ///             minute: 13,
    ///             second: 14,
    ///             microsecond: 0,
    ///         },
    ///         offset: Some(0),
    ///     }
    /// );
    /// assert_eq!(dt.to_string(), "2022-01-01T12:13:14Z");
    /// ```
    #[inline]
    pub fn parse_str(str: &str) -> Result<Self, ParseError> {
        Self::parse_bytes(str.as_bytes())
    }

    /// Parse a datetime from bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to parse
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::{DateTime, Date, Time};
    ///
    /// let dt = DateTime::parse_bytes(b"2022-01-01T12:13:14Z").unwrap();
    /// assert_eq!(
    ///     dt,
    ///     DateTime {
    ///         date: Date {
    ///             year: 2022,
    ///             month: 1,
    ///             day: 1,
    ///         },
    ///         time: Time {
    ///             hour: 12,
    ///             minute: 13,
    ///             second: 14,
    ///             microsecond: 0,
    ///         },
    ///         offset: Some(0),
    ///     }
    /// );
    /// assert_eq!(dt.to_string(), "2022-01-01T12:13:14Z");
    /// ```
    pub fn parse_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        // First up, parse the full date if we can
        let date = Date::parse_bytes_partial(bytes)?;

        // Next parse the separator between date and time
        let sep = bytes.get(10).copied();
        if sep != Some(b'T') && sep != Some(b't') && sep != Some(b' ') && sep != Some(b'_') {
            return Err(ParseError::InvalidCharDateTimeSep);
        }

        // Next try to parse the time
        let (time, time_length) = Time::parse_bytes_partial(bytes, 11)?;
        let mut position = 11 + time_length;

        // And finally, parse the offset
        let mut offset: Option<i16> = None;

        if let Some(next_char) = bytes.get(position).copied() {
            position += 1;
            if next_char == b'Z' || next_char == b'z' {
                offset = Some(0);
            } else {
                let sign = match next_char {
                    b'+' => 1,
                    b'-' => -1,
                    226 => {
                        // U+2212 MINUS "−" is allowed under ISO 8601 for negative timezones
                        // > python -c 'print([c for c in "−".encode()])'
                        // its raw byte values are [226, 136, 146]
                        if bytes.get(position).copied() != Some(136) {
                            return Err(ParseError::InvalidCharTzSign);
                        }
                        if bytes.get(position + 1).copied() != Some(146) {
                            return Err(ParseError::InvalidCharTzSign);
                        }
                        position += 2;
                        -1
                    }
                    _ => return Err(ParseError::InvalidCharTzSign),
                };

                let h1 = get_digit!(bytes, position, InvalidCharTzHour) as i16;
                let h2 = get_digit!(bytes, position + 1, InvalidCharTzHour) as i16;

                let m1 = match bytes.get(position + 2) {
                    Some(b':') => {
                        position += 3;
                        get_digit!(bytes, position, InvalidCharTzMinute) as i16
                    }
                    Some(c) if (b'0'..=b'9').contains(c) => {
                        position += 2;
                        (c - b'0') as i16
                    }
                    _ => return Err(ParseError::InvalidCharTzMinute),
                };
                let m2 = get_digit!(bytes, position + 1, InvalidCharTzMinute) as i16;

                offset = Some(sign * (h1 * 600 + h2 * 60 + m1 * 10 + m2));
                position += 2;
            }
        }
        if bytes.len() > position {
            return Err(ParseError::ExtraCharacters);
        }

        Ok(Self { date, time, offset })
    }

    /// Create a datetime from a Unix Timestamp in seconds or milliseconds
    ///
    /// ("Unix Timestamp" means number of seconds or milliseconds since 1970-01-01)
    ///
    /// Datetimes much be between `1600-01-01T00:00:00` and `9999-12-31T23:59:59.999999` inclusive.
    ///
    /// If the absolute value is > 2e10 (`20_000_000_000`) it is interpreted as being in milliseconds.
    ///
    /// That means:
    /// * `20_000_000_000` is `2603-10-11T11:33:20`
    /// * `20_000_000_001` is `1970-08-20T11:33:20.001`
    /// * `-20_000_000_000` gives an error - `DateTooSmall`
    /// * `-20_000_000_001` is `1969-05-14T12:26:39.999`
    ///
    /// # Arguments
    ///
    /// * `timestamp` - timestamp in either seconds or milliseconds
    /// * `timestamp_microsecond` - microseconds fraction of a second timestamp
    ///
    /// Where `timestamp` is interrupted  as milliseconds and is not a whole second, the remainder is added to
    /// `timestamp_microsecond`.
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::DateTime;
    ///
    /// let d = DateTime::from_timestamp(1654619320, 123).unwrap();
    /// assert_eq!(d.to_string(), "2022-06-07T16:28:40.000123");
    /// ```
    pub fn from_timestamp(timestamp: i64, timestamp_microsecond: u32) -> Result<Self, ParseError> {
        let (mut second, extra_microsecond) = Date::timestamp_watershed(timestamp)?;
        let mut total_microsecond = timestamp_microsecond
            .checked_add(extra_microsecond)
            .ok_or(ParseError::TimeTooLarge)?;
        if total_microsecond >= 1_000_000 {
            second = second
                .checked_add(total_microsecond as i64 / 1_000_000)
                .ok_or(ParseError::TimeTooLarge)?;
            total_microsecond %= 1_000_000;
        }
        let date = Date::from_timestamp_calc(second)?;
        // rem_euclid since if `timestamp_second = -100`, we want `time_second = 86300` (e.g. `86400 - 100`)
        let time_second = second.rem_euclid(86_400) as u32;
        Ok(Self {
            date,
            time: Time::from_timestamp(time_second, total_microsecond)?,
            offset: None,
        })
    }
}
