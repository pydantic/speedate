use std::fmt;

/// Parsing datetime values
/// Taken from toml-rs and modified extensively.

// get a character from the bytes as as a decimal
macro_rules! get_digit {
    ($bytes:ident, $index:expr, $error:ident) => {
        match $bytes.get($index) {
            Some(c) if (b'0'..=b'9').contains(&c) => c - b'0',
            _ => return Err(ParseError::$error),
        }
    };
}

// as above without bounds check, requires length to checked first!
macro_rules! get_digit_unchecked {
    ($bytes:ident, $index:expr, $error:ident) => {
        match $bytes.get_unchecked($index) {
            c if (b'0'..=b'9').contains(&c) => c - b'0',
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
    pub fn parse_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let d = Self::parse_bytes_internal(bytes)?;

        if bytes.len() > 10 {
            return Err(ParseError::ExtraCharacters);
        }

        Ok(d)
    }

    fn parse_bytes_internal(bytes: &[u8]) -> Result<Self, ParseError> {
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

/// A parsed Time
///
/// May be part of a `DateTime`.
/// Allowed values: `07:32`, `07:32:00`, `00:32:00.999999`
///
/// Fractions of a second are to microsecond precision, if the value contains greater
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
    pub fn parse_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let (t, length) = Self::parse_bytes_internal(bytes, 0)?;

        if bytes.len() > length {
            return Err(ParseError::ExtraCharacters);
        }

        Ok(t)
    }

    fn parse_bytes_internal(bytes: &[u8], offset: usize) -> Result<(Self, usize), ParseError> {
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

    pub fn parse_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        // First up, parse the full date if we can
        let date = Date::parse_bytes_internal(bytes)?;

        // Next parse the separator between date and time
        let sep = bytes.get(10).copied();
        if sep != Some(b'T') && sep != Some(b't') && sep != Some(b' ') && sep != Some(b'_') {
            return Err(ParseError::InvalidCharDateTimeSep);
        }

        // Next try to parse the time
        let (time, time_length) = Time::parse_bytes_internal(bytes, 11)?;
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
}

#[derive(Debug, PartialEq, Clone)]
pub struct Duration {
    pub day: i64,
    pub second: i32,
    pub microsecond: i32,
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "P")?;
        if self.day != 0 {
            let year = self.day / 365;
            if year != 0 {
                write!(f, "{}Y", year)?;
            }
            let day = self.day % 365;
            if day != 0 {
                write!(f, "{}D", day)?;
            }
        }
        if self.second != 0 || self.microsecond != 0 {
            write!(f, "T{}", self.second)?;
            if self.microsecond != 0 {
                let s = format!("{:06}", self.microsecond);
                write!(f, ".{}", s.trim_end_matches('0'))?;
            }
            write!(f, "S")?;
        }
        Ok(())
    }
}

impl Duration {
    #[inline]
    pub fn parse_str(str: &str) -> Result<Self, ParseError> {
        Self::parse_bytes(str.as_bytes())
    }

    #[inline]
    pub fn parse_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.get(0).copied() == Some(b'P') {
            Self::parse_iso_duration(bytes)
        } else {
            Err(ParseError::ToDo)
        }
    }

    fn parse_iso_duration(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut got_t = false;
        let mut position: usize = 1; // because 0 is 'P'
        let mut day = 0_i64;
        let mut second = 0;
        let mut microsecond = 0;
        loop {
            match bytes.get(position).copied() {
                Some(b'T') => {
                    if got_t {
                        return Err(ParseError::ToDo);
                    }
                    got_t = true;
                }
                Some(c) => {
                    let (value, new_pos) = Self::parse_number(bytes, c, position)?;
                    position = new_pos;
                    if got_t {
                        let mult = match bytes.get(position).copied() {
                            Some(b'H') => 3600.0,
                            Some(b'M') => 60.0,
                            Some(b'S') => 1.0,
                            _ => return Err(ParseError::ToDo),
                        };
                        let total_seconds = value * mult;
                        second += total_seconds.floor() as i32;
                        microsecond += (total_seconds % 1.0 * 1_000_000.0).round() as i32;
                    } else {
                        let mult = match bytes.get(position).copied() {
                            Some(b'Y') => 365.0,
                            Some(b'M') => 30.0,
                            Some(b'W') => 7.0,
                            Some(b'D') => 1.0,
                            _ => return Err(ParseError::ToDo),
                        };
                        let total_days = value * mult;
                        day += total_days.floor() as i64;
                        second += (total_days % 1.0 * 86_400.0).round() as i32;
                    }
                }
                None => break,
            }
            position += 1;
        }
        if position < 3 {
            return Err(ParseError::ToDo);
        }

        if bytes.len() > position {
            return Err(ParseError::ExtraCharacters);
        }
        if second > 86_400 {
            day += second as i64 / 86_400;
            second %= 86_400;
        }
        Ok(Self {
            day,
            second,
            microsecond,
        })
    }

    fn parse_number(bytes: &[u8], d1: u8, offset: usize) -> Result<(f64, usize), ParseError> {
        let mut v = match d1 {
            c if (b'0'..=b'9').contains(&d1) => (c - b'0') as f64,
            _ => return Err(ParseError::ToDo),
        };
        let mut position = offset + 1;
        loop {
            match bytes.get(position) {
                Some(c) if (b'0'..=b'9').contains(&c) => {
                    v *= 10.0;
                    v += (c - b'0') as f64;
                    position += 1;
                }
                Some(b'.') | Some(b',') => {
                    let dot_pos = position;
                    position += 1;
                    loop {
                        match bytes.get(position) {
                            Some(c) if (b'0'..=b'9').contains(&c) => {
                                let f = (c - b'0') as f64;
                                v += f / 10_i32.pow((position - dot_pos) as u32) as f64;
                                position += 1;
                            }
                            _ => return Ok((v, position)),
                        }
                    }
                }
                _ => return Ok((v, position)),
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    ToDo,
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
}
