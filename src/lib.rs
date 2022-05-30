use std::fmt;

/// Parsing datetime values
/// Taken from toml-rs and modified extensively.

// Get the next character from the bytes iterator as a decimal
macro_rules! next_digit {
    ($bytes:ident, $error:ident) => {
        match $bytes.next() {
            Some(c) if (b'0'..=b'9').contains(&c) => c - b'0',
            _ => return Err(ParseError::$error),
        }
    };
}

macro_rules! next_two_digits {
    ($bytes:ident, $error:ident) => {
        match $bytes.next_two() {
            Some((c1, c2)) if (b'0'..=b'9').contains(&c1) && (b'0'..=b'9').contains(&c2) => {
                (c1 - b'0') * 10 + c2 - b'0'
            }
            _ => return Err(ParseError::$error),
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
        let mut bytes = ByteIter::new(date);
        let d = Self::parse_iter(&mut bytes)?;

        if bytes.peak().is_some() {
            return Err(ParseError::ExtraCharacters);
        }

        Ok(d)
    }

    fn parse_iter(bytes: &mut ByteIter) -> Result<Self, ParseError> {
        // let y1 = next_digit!(bytes, InvalidCharYear) as u16;
        // let y2 = next_digit!(bytes, InvalidCharYear) as u16;
        // let y3 = next_digit!(bytes, InvalidCharYear) as u16;
        // let y4 = next_digit!(bytes, InvalidCharYear) as u16;
        let y1 = next_two_digits!(bytes, InvalidCharYear) as u16;
        let y2 = next_two_digits!(bytes, InvalidCharYear) as u16;
        let year = y1 * 100 + y2;

        match bytes.next() {
            Some(b'-') => (),
            _ => return Err(ParseError::InvalidCharDateSep),
        }

        let month = next_two_digits!(bytes, InvalidCharMonth);
        // let m1 = next_digit!(bytes, InvalidCharMonth);
        // let m2 = next_digit!(bytes, InvalidCharMonth);

        match bytes.next() {
            Some(b'-') => (),
            _ => return Err(ParseError::InvalidCharDateSep),
        }

        let day = next_two_digits!(bytes, InvalidCharDay);
        // let d1 = next_digit!(bytes, InvalidCharDay);
        // let d2 = next_digit!(bytes, InvalidCharDay);

        // let year = y1 * 1000 + y2 * 100 + y3 * 10 + y4;
        // let month = m1 * 10 + m2;

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

        // let day = d1 * 10 + d2;

        if day < 1 || day > max_days {
            return Err(ParseError::OutOfRangeDay);
        }

        Ok(Date { year, month, day })
    }
}

/// A parsed Time
///
/// May be part of a `DateTime`.
/// Allowed values: `07:32`, `07:32:00`, `00:32:00.999999`
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
            let s = format!("{:06}", self.microsecond);
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
        let mut bytes = ByteIter::new(date);
        let t = Self::parse_iter(&mut bytes)?;

        if bytes.peak().is_some() {
            return Err(ParseError::ExtraCharacters);
        }

        Ok(t)
    }

    fn parse_iter(bytes: &mut ByteIter) -> Result<Self, ParseError> {
        // let h1 = next_digit!(bytes, InvalidCharHour);
        // let h2 = next_digit!(bytes, InvalidCharHour);
        // let hour = h1 * 10 + h2;
        let hour = next_two_digits!(bytes, InvalidCharHour);

        if hour > 23 {
            return Err(ParseError::OutOfRangeHour);
        }

        match bytes.next() {
            Some(b':') => (),
            _ => return Err(ParseError::InvalidCharTimeSep),
        }
        // let m1 = next_digit!(bytes, InvalidCharMinute);
        // let m2 = next_digit!(bytes, InvalidCharMinute);
        // let minute = m1 * 10 + m2;
        let minute = next_two_digits!(bytes, InvalidCharMinute);

        if minute > 59 {
            return Err(ParseError::OutOfRangeMinute);
        }

        let (second, microsecond) = match bytes.peak() {
            Some(b':') => {
                bytes.advance();
                // let s1 = next_digit!(bytes, InvalidCharSecond);
                // let s2 = next_digit!(bytes, InvalidCharSecond);
                // let second = s1 * 10 + s2;
                let second = next_two_digits!(bytes, InvalidCharSecond);
                if second > 59 {
                    return Err(ParseError::OutOfRangeSecond);
                }

                let mut microsecond = 0;
                let frac_sep = bytes.peak();
                if frac_sep == Some(b'.') || frac_sep == Some(b',') {
                    bytes.advance();
                    let mut i: u32 = 0;
                    loop {
                        match bytes.next() {
                            Some(c) if (b'0'..=b'9').contains(&c) => {
                                microsecond *= 10;
                                microsecond += (c - b'0') as u32;
                            }
                            _ => {
                                bytes.back();
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
                        microsecond *= 10_u32.pow(6 - i);
                    }
                }
                (second, microsecond)
            }
            _ => (0, 0),
        };

        Ok(Time {
            hour,
            minute,
            second,
            microsecond,
        })
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
                write!(f, "{:+03}:{:02}", offset / 60, (offset % 60).abs())?;
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
        let mut bytes = ByteIter::new(date);

        // First up, parse the full date if we can
        let date = Date::parse_iter(&mut bytes)?;

        // Next parse the separator between date and time
        let sep = bytes.next();
        if sep != Some(b'T') && sep != Some(b't') && sep != Some(b' ') && sep != Some(b'_') {
            return Err(ParseError::InvalidCharDateTimeSep);
        }

        // Next try to parse the time
        let time = Time::parse_iter(&mut bytes)?;

        // And finally, parse the offset
        let mut offset: Option<i16> = None;

        if let Some(next) = bytes.peak() {
            bytes.advance();
            if next == b'Z' || next == b'z' {
                offset = Some(0);
            } else {
                let sign = match next {
                    b'+' => 1,
                    b'-' => -1,
                    226 => {
                        // U+2212 MINUS "−" is allowed under ISO 8601 for negative timezones
                        // > python -c 'print([c for c in "−".encode()])'
                        // its raw byte values are [226, 136, 146]
                        if bytes.next() != Some(136) {
                            return Err(ParseError::InvalidCharTzSign);
                        }
                        if bytes.next() != Some(146) {
                            return Err(ParseError::InvalidCharTzSign);
                        }
                        -1
                    }
                    _ => return Err(ParseError::InvalidCharTzSign),
                };

                let h1 = next_digit!(bytes, InvalidCharTzHour) as i16;
                let h2 = next_digit!(bytes, InvalidCharTzHour) as i16;
                // let hour = next_two_digits!(bytes, InvalidCharTzHour) as i16;

                let m1 = match bytes.next() {
                    Some(b':') => next_digit!(bytes, InvalidCharTzMinute) as i16,
                    Some(c) if (b'0'..=b'9').contains(&c) => (c - b'0') as i16,
                    _ => return Err(ParseError::InvalidCharTzMinute),
                };
                let m2 = next_digit!(bytes, InvalidCharTzMinute) as i16;

                // offset = Some(sign * (hour * 60 + m1 * 10 + m2));
                offset = Some(sign * (h1 * 600 + h2 * 60 + m1 * 10 + m2));
            }
        }

        if bytes.peak().is_some() {
            return Err(ParseError::ExtraCharacters);
        }

        Ok(DateTime { date, time, offset })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
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
}

struct ByteIter<'a> {
    index: usize,
    bytes: &'a [u8],
}

impl<'a> ByteIter<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { index: 0, bytes }
    }

    fn peak(&self) -> Option<u8> {
        match self.bytes.get(self.index) {
            Some(b) => Some(*b),
            None => None,
        }
    }

    fn next_two(&mut self) -> Option<(u8, u8)> {
        let index = self.index;
        self.index += 2;
        if index + 1 < self.bytes.len() {
            unsafe { Some((*self.bytes.get_unchecked(index), *self.bytes.get_unchecked(index + 1))) }
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn back(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }
}

impl<'a> Iterator for ByteIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        match self.bytes.get(index) {
            Some(b) => Some(*b),
            None => None,
        }
    }
}
