use std::fmt;

use crate::{get_digit, get_digit_unchecked, ParseError};

/// A Time
///
/// Allowed formats:
/// * `HH:MM:SS`
/// * `HH:MM:SS.FFFFFF` 1 to 6 digits are allowed
/// * `HH:MM`
///
/// Fractions of a second are to microsecond precision, if the value contains greater
/// precision, an error is raised.
///
/// # Comparison
///
/// `Time` supports equality and inequality comparisons (`>`, `<`, `>=` & `<=`).
///
/// ```
/// use speedate::Time;
///
/// let t1 = Time::parse_str("12:10:20").unwrap();
/// let t2 = Time::parse_str("12:13:14").unwrap();
///
/// assert!(t2 > t1);
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct Time {
    /// Hour: 0 to 23
    pub hour: u8,
    /// Minute: 0 to 59
    pub minute: u8,
    /// Second: 0 to 59
    pub second: u8,
    /// microseconds: 0 to 999999
    pub microsecond: u32,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)?;
        if self.microsecond != 0 {
            let s = format!("{:06}", self.microsecond);
            write!(f, ".{}", s.trim_end_matches('0'))?;
        }
        Ok(())
    }
}

impl Time {
    /// Parse a time from a string
    ///
    /// # Arguments
    ///
    /// * `str` - The string to parse
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::Time;
    ///
    /// let d = Time::parse_str("12:13:14.123456").unwrap();
    /// assert_eq!(
    ///     d,
    ///     Time {
    ///         hour: 12,
    ///         minute: 13,
    ///         second: 14,
    ///         microsecond: 123456,
    ///     }
    /// );
    /// assert_eq!(d.to_string(), "12:13:14.123456");
    /// ```
    #[inline]
    pub fn parse_str(str: &str) -> Result<Self, ParseError> {
        Self::parse_bytes(str.as_bytes())
    }

    /// Parse a time from bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes to parse
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::Time;
    ///
    /// let d = Time::parse_bytes(b"12:13:14.123456").unwrap();
    /// assert_eq!(
    ///     d,
    ///     Time {
    ///         hour: 12,
    ///         minute: 13,
    ///         second: 14,
    ///         microsecond: 123456,
    ///     }
    /// );
    /// assert_eq!(d.to_string(), "12:13:14.123456");
    /// ```
    #[inline]
    pub fn parse_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let (t, length) = Self::parse_bytes_partial(bytes, 0)?;

        if bytes.len() > length {
            return Err(ParseError::ExtraCharacters);
        }

        Ok(t)
    }

    /// Create a time from seconds and microseconds.
    ///
    /// # Arguments
    ///
    /// * `timestamp_second` - timestamp in seconds
    /// * `timestamp_microsecond` - microseconds fraction of a second timestamp
    ///
    /// If `seconds + timestamp_microsecond` exceeds 86400, an error is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::Time;
    ///
    /// let d = Time::from_timestamp(3740, 123).unwrap();
    /// assert_eq!(d.to_string(), "01:02:20.000123");
    /// ```
    pub fn from_timestamp(timestamp_second: u32, timestamp_microsecond: u32) -> Result<Self, ParseError> {
        let mut second = timestamp_second;
        let mut microsecond = timestamp_microsecond;
        if microsecond >= 1_000_000 {
            second = second
                .checked_add(microsecond / 1_000_000)
                .ok_or(ParseError::TimeTooLarge)?;
            microsecond %= 1_000_000;
        }
        if second >= 86_400 {
            return Err(ParseError::TimeTooLarge);
        }
        Ok(Self {
            hour: (second / 3600) as u8,
            minute: ((second % 3600) / 60) as u8,
            second: (second % 60) as u8,
            microsecond,
        })
    }

    /// Parse a time from bytes with a starting index, no check is performed for extract characters at
    /// the end of the string
    pub(crate) fn parse_bytes_partial(bytes: &[u8], offset: usize) -> Result<(Self, usize), ParseError> {
        if bytes.len() - offset < 5 {
            return Err(ParseError::TooShort);
        }
        let hour: u8;
        let minute: u8;
        unsafe {
            let h1 = get_digit_unchecked!(bytes, offset, InvalidCharHour);
            let h2 = get_digit_unchecked!(bytes, offset + 1, InvalidCharHour);
            hour = h1 * 10 + h2;

            match bytes.get_unchecked(offset + 2) {
                b':' => (),
                _ => return Err(ParseError::InvalidCharTimeSep),
            }
            let m1 = get_digit_unchecked!(bytes, offset + 3, InvalidCharMinute);
            let m2 = get_digit_unchecked!(bytes, offset + 4, InvalidCharMinute);
            minute = m1 * 10 + m2;
        }

        if hour > 23 {
            return Err(ParseError::OutOfRangeHour);
        }

        if minute > 59 {
            return Err(ParseError::OutOfRangeMinute);
        }
        let mut length: usize = 5;

        let (second, microsecond) = match bytes.get(offset + 5) {
            Some(b':') => {
                let s1 = get_digit!(bytes, offset + 6, InvalidCharSecond);
                let s2 = get_digit!(bytes, offset + 7, InvalidCharSecond);
                let second = s1 * 10 + s2;
                if second > 59 {
                    return Err(ParseError::OutOfRangeSecond);
                }
                length = 8;

                let mut microsecond = 0;
                let frac_sep = bytes.get(offset + 8).copied();
                if frac_sep == Some(b'.') || frac_sep == Some(b',') {
                    length = 9;
                    let mut i: usize = 0;
                    loop {
                        match bytes.get(offset + length + i) {
                            Some(c) if (b'0'..=b'9').contains(c) => {
                                microsecond *= 10;
                                microsecond += (c - b'0') as u32;
                            }
                            _ => {
                                break;
                            }
                        }
                        i += 1;
                        if i > 6 {
                            return Err(ParseError::SecondFractionTooLong);
                        }
                    }
                    if i == 0 {
                        return Err(ParseError::SecondFractionMissing);
                    }
                    if i < 6 {
                        microsecond *= 10_u32.pow(6 - i as u32);
                    }
                    length += i;
                }
                (second, microsecond)
            }
            _ => (0, 0),
        };
        let t = Self {
            hour,
            minute,
            second,
            microsecond,
        };
        Ok((t, length))
    }

    /// Get the total seconds of the time
    ///
    /// E.g. hours + minutes + seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::Time;
    ///
    /// let d = Time::parse_str("12:13:14.123456").unwrap();
    /// assert_eq!(d.total_seconds(), 12 * 3600 + 13 * 60 + 14);
    /// ```
    pub fn total_seconds(&self) -> u32 {
        let mut total_seconds = self.hour as u32 * 3600;
        total_seconds += self.minute as u32 * 60;
        total_seconds += self.second as u32;
        total_seconds
    }
}

