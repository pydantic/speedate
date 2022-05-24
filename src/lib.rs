use std::fmt;
use std::iter::Copied;
use std::slice::Iter;

/// Parsing datetime values
/// Taken from toml-rs and modified extensively.

macro_rules! digit {
    ($bytes:expr) => {
        match $bytes.next() {
            Some(c) if (b'0'..=b'9').contains(&c) => c - b'0',
            _ => return Err(ParseError::InvalidChar),
        }
    };
}

/// A parsed Date
///
/// May be part of a `DateTime`.
/// Allowed values: `1979-05-27`.
#[derive(Debug, PartialEq, Clone)]
pub struct Date {
    /// Year: four digits
    pub year: u16,
    /// Month: 1 to 12
    pub month: u8,
    /// Day: 1 to {28, 29, 30, 31} (based on month/year)
    pub day: u8,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl Date {
    #[inline]
    pub fn parse_str(str: &str) -> Result<Self, ParseError> {
        Self::parse_bytes(str.as_bytes())
    }

    #[inline]
    pub fn parse_bytes(date: &[u8]) -> Result<Self, ParseError> {
        let mut bytes = date.iter().copied();
        Self::parse_iter(&mut bytes)
    }

    fn parse_iter(bytes: &mut Copied<Iter<u8>>) -> Result<Self, ParseError> {
        let y1 = digit!(bytes) as u16;
        let y2 = digit!(bytes) as u16;
        let y3 = digit!(bytes) as u16;
        let y4 = digit!(bytes) as u16;

        match bytes.next() {
            Some(b'-') => (),
            _ => return Err(ParseError::InvalidChar),
        }

        let m1 = digit!(bytes);
        let m2 = digit!(bytes);

        match bytes.next() {
            Some(b'-') => (),
            _ => return Err(ParseError::InvalidChar),
        }

        let d1 = digit!(bytes);
        let d2 = digit!(bytes);

        let year = y1 * 1000 + y2 * 100 + y3 * 10 + y4;
        let month = m1 * 10 + m2;

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
            _ => return Err(ParseError::OutOfRange),
        };

        let day = d1 * 10 + d2;

        if day < 1 || day > max_days {
            return Err(ParseError::OutOfRange);
        }

        Ok(Date { year, month, day })
    }
}

/// A parsed Time
///
/// May be part of a `DateTime`.
/// Allowed values: `07:32:00`, `00:32:00.999999`
///
/// Fractions of a second are to millisecond precision, if the value contains greater
/// precision, an error is raised (TODO).
#[derive(Debug, PartialEq, Clone)]
pub struct Time {
    /// Hour: 0 to 23
    pub hour: u8,
    /// Minute: 0 to 59
    pub minute: u8,
    /// Second: 0 to 59
    pub second: u8,
    /// microseconds: 0 to 999_999
    pub microsecond: u32,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)?;
        if self.microsecond != 0 {
            let s = format!("{:09}", self.microsecond);
            write!(f, ".{}", s.trim_end_matches('0'))?;
        }
        Ok(())
    }
}

impl Time {
    #[inline]
    pub fn parse_str(str: &str) -> Result<Self, ParseError> {
        Self::parse_bytes(str.as_bytes())
    }

    #[inline]
    pub fn parse_bytes(date: &[u8]) -> Result<Self, ParseError> {
        let mut bytes = date.iter().copied();
        Self::parse_iter(&mut bytes)
    }

    fn parse_iter(bytes: &mut Copied<Iter<u8>>) -> Result<Self, ParseError> {
        let h1 = digit!(bytes);
        let h2 = digit!(bytes);

        match bytes.next() {
            Some(b':') => (),
            _ => return Err(ParseError::InvalidChar),
        }
        let m1 = digit!(bytes);
        let m2 = digit!(bytes);

        match bytes.next() {
            Some(b':') => (),
            _ => return Err(ParseError::InvalidChar),
        }
        let s1 = digit!(bytes);
        let s2 = digit!(bytes);

        let mut microsecond = 0;
        let mut micro_bytes = bytes.clone();
        if micro_bytes.next() == Some(b'.') {
            let mut i: u32 = 0;
            for digit in micro_bytes {
                match digit {
                    b'0'..=b'9' => {
                        i += 1;
                        if i > 6 {
                            return Err(ParseError::InvalidChar);
                        }
                        microsecond *= 10;
                        microsecond += (digit - b'0') as u32;
                    }
                    _ => {
                        if i == 0 {
                            return Err(ParseError::InvalidChar);
                        }
                        break;
                    }
                }
            }
            if i < 6 {
                microsecond *= 10_u32.pow(6 - i);
            }
            bytes.nth(i as usize);
        }

        let time = Time {
            hour: h1 * 10 + h2,
            minute: m1 * 10 + m2,
            second: s1 * 10 + s2,
            microsecond,
        };

        if time.hour > 24 {
            return Err(ParseError::OutOfRange);
        }
        if time.minute > 59 {
            return Err(ParseError::OutOfRange);
        }
        if time.second > 59 {
            return Err(ParseError::OutOfRange);
        }
        if time.microsecond > 999_999 {
            return Err(ParseError::OutOfRange);
        }

        Ok(time)
    }
}

/// A parsed DateTime
///
/// Combines a `Date`, `Time` and optionally a timezone offset in minutes.
/// Allowed values: `1979-05-27T07:32:00Z`, `1979-05-27T00:32:00-07:00`, `1979-05-27 07:32:00Z`
///
/// For the sake of readability, you may replace the T delimiter between date
/// and time with a space character (as permitted by RFC 3339 section 5.6).
#[derive(Debug, PartialEq, Clone)]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
    // offset in minutes if provided
    pub offset: Option<i16>,
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}T{}", self.date, self.time)?;
        if let Some(offset) = self.offset {
            if offset == 0 {
                write!(f, "Z")?;
            } else {
                write!(f, "{:+03}:{:02}", offset / 60, offset % 60)?;
            }
        }
        Ok(())
    }
}

impl DateTime {
    #[inline]
    pub fn parse_str(str: &str) -> Result<Self, ParseError> {
        Self::parse_bytes(str.as_bytes())
    }

    pub fn parse_bytes(date: &[u8]) -> Result<Self, ParseError> {
        let mut bytes = date.iter().copied();
        if bytes.len() < 3 {
            return Err(ParseError::TooShort);
        }

        // First up, parse the full date if we can
        let date = Date::parse_iter(&mut bytes)?;

        // Next parse the separator between date and time
        let sep = bytes.next();
        if sep != Some(b'T') && sep != Some(b't') && sep != Some(b' ') {
            return Err(ParseError::InvalidChar);
        }

        // Next try to parse the time
        let time = Time::parse_iter(&mut bytes)?;

        // And finally, parse the offset
        let mut offset: Option<i16> = None;

        if let Some(next) = bytes.next() {
            if next == b'Z' || next == b'z' {
                offset = Some(0);
            } else {
                let sign = match next {
                    b'+' => 1,
                    b'-' => -1,
                    _ => return Err(ParseError::TooShort),
                };

                let h1 = digit!(bytes) as i16;
                let h2 = digit!(bytes) as i16;

                let m1 = match bytes.next() {
                    Some(b':') => digit!(bytes) as i16,
                    Some(c) if (b'0'..=b'9').contains(&c) => (c - b'0') as i16,
                    _ => return Err(ParseError::InvalidChar),
                };
                let m2 = digit!(bytes) as i16;

                offset = Some(sign * h1 * 600 + h2 * 60 + m1 * 10 + m2);
            }
        }

        if bytes.next().is_some() {
            return Err(ParseError::TooLong);
        }

        Ok(DateTime { date, time, offset })
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    TooShort,
    TooLong,
    InvalidChar,
    OutOfRange,
}
