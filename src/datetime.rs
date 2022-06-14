use std::cmp::Ordering;
use std::fmt;

use crate::{get_digit, Date, ParseError, Time};

/// A DateTime
///
/// Combines a [Date], [Time] and optionally a timezone offset in minutes.
/// Allowed values:
/// * `YYYY-MM-DDTHH:MM:SS` - all the above time formats are allowed for the time part
/// * `YYYY-MM-DD HH:MM:SS` - `T`, `t`, ` ` and `_` are allowed as separators
/// * `YYYY-MM-DDTHH:MM:SSZ` - `Z` or `z` is allowed as timezone
/// * `YYYY-MM-DDTHH:MM:SS+08:00`- positive and negative timezone are allowed,
///   as per ISO 8601, U+2212 minus `−` is allowed as well as ascii minus `-` (U+002D)
/// * `YYYY-MM-DDTHH:MM:SS+0800` - the colon (`:`) in the timezone is optional
///
/// # Comparison
///
/// `DateTime` supports equality (`==`) and inequality (`>`, `<`, `>=`, `<=`) comparisons.
///
/// See [DateTime::partial_cmp] for how this works.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DateTime {
    /// date part of the datetime
    pub date: Date,
    /// time part of the datetime
    pub time: Time,
    /// timezone offset in seconds if provided, must be >-24h and <24h
    // This range is to match python,
    // Note: [Stack Overflow suggests](https://stackoverflow.com/a/8131056/949890) larger offsets can happen
    pub offset: Option<i32>,
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}T{}", self.date, self.time)?;
        if let Some(offset) = self.offset {
            if offset == 0 {
                write!(f, "Z")?;
            } else {
                let mins = offset / 60;
                write!(f, "{:+03}:{:02}", mins / 60, (mins % 60).abs())?;
            }
        }
        Ok(())
    }
}

impl PartialOrd for DateTime {
    /// Compare two datetimes by inequality.
    ///
    /// `DateTime` supports equality and inequality comparisons (`>`, `<`, `>=` & `<=`).
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::DateTime;
    ///
    /// let dt1 = DateTime::parse_str("2020-02-03T04:05:06.07").unwrap();
    /// let dt2 = DateTime::parse_str("2020-02-03T04:05:06.08").unwrap();
    ///
    /// assert!(dt2 > dt1);
    /// ```
    ///
    ///  # Comparison with Timezones
    ///
    /// When comparing two datetimes, we want "less than" or "greater than" refer to "earlier" or "later"
    /// in the absolute course of time. We therefore need to be careful when comparing datetimes with different
    /// timezones. (If it wasn't for timezones, we could omit all this extra logic and thinking and just compare
    /// struct members directly as we do with [Time], [Date] and [crate::Duration]).
    ///
    /// From [wikipedia](https://en.wikipedia.org/wiki/UTC_offset#Time_zones_and_time_offsets)
    ///
    /// > The UTC offset is an amount of time subtracted from or added to UTC time to specify the local solar time...
    ///
    /// So, we can imagine that at 3pm in the UK (UTC+0) (in winter, to avoid DST confusion) it's 4pm in France (UTC+1).
    ///
    /// Thus to compare two datetimes in absolute terms we need to **SUBTRACT** the timezone offset.
    ///
    /// As if timezones weren't complicated enough, there are three extra considerations here:
    /// 1. **naïve vs. non-naïve:** We also have to consider the case where one datetime has a timezone and the other
    ///    does not (e.g. is "timezone "naïve"). When comparing naïve datetimes to non-naïve, this library
    ///    assumes the naïve datetime has the same timezone as the non-naïve, th is is different to other
    ///    implementations (e.g. python) where such comparisons fail.
    /// 2. **Direction:** As described in PostgreSQL's docs, in the POSIX Time Zone Specification
    ///    "The positive sign is used for zones west of Greenwich", which is opposite to the ISO-8601 sign convention.
    ///    In other words, the offset is reversed, see the end of
    ///    [this blog](http://blog.untrod.com/2016/08/actually-understanding-timezones-in-postgresql.html)
    ///    and the [PostgreSQL docs](https://www.postgresql.org/docs/14/datetime-posix-timezone-specs.html) for more
    ///    info.
    /// 3. **Equality comparison:** None of this logic is used for equality (`==`) comparison where we can just compare
    ///    struct members directly, e.g. require the timezone offset to be the same for two datetimes to be equal.
    ///
    /// ## Timezone Examples
    ///
    /// ```
    /// use speedate::DateTime;
    ///
    /// let dt_uk_3pm = DateTime::parse_str("2000-01-01T15:00:00Z").unwrap();
    /// let dt_france_4pm = DateTime::parse_str("2000-01-01T16:00:00+01:00").unwrap();
    ///
    /// assert!(dt_uk_3pm >= dt_france_4pm);  // the two dts are actually the same instant
    /// assert!(dt_uk_3pm <= dt_france_4pm);  // the two dts are actually the same instant
    /// assert_ne!(dt_uk_3pm, dt_france_4pm);  // no equal because timezones much match for equality
    ///
    /// let dt_uk_330pm = DateTime::parse_str("2000-01-01T15:30:00Z").unwrap();
    ///
    /// assert!(dt_uk_330pm > dt_uk_3pm);
    /// assert!(dt_uk_330pm > dt_france_4pm);
    ///
    /// // as described in point 1 above, naïve datetimes are assumed to
    /// // have the same timezone as the non-naïve
    /// let dt_naive_330pm = DateTime::parse_str("2000-01-01T15:30:00").unwrap();
    /// assert!(dt_uk_3pm < dt_naive_330pm);
    /// assert!(dt_france_4pm > dt_naive_330pm);
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.offset, other.offset) {
            (Some(_), Some(_)) => match self.timestamp_tz().partial_cmp(&other.timestamp_tz()) {
                Some(Ordering::Equal) => self.time.microsecond.partial_cmp(&other.time.microsecond),
                otherwise => otherwise,
            },
            _ => match self.date.partial_cmp(&other.date) {
                Some(Ordering::Equal) => self.time.partial_cmp(&other.time),
                otherwise => otherwise,
            },
        }
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
    ///
    /// With a non-zero timezone
    /// (we also use a different separator and omit the colon in timezone here):
    ///
    /// ```
    /// use speedate::{DateTime, Date, Time};
    ///
    /// let dt = DateTime::parse_str("2000-02-29 12:13:14-0830").unwrap();
    /// assert_eq!(
    ///     dt,
    ///     DateTime {
    ///         date: Date {
    ///             year: 2000,
    ///             month: 2,
    ///             day: 29,
    ///         },
    ///         time: Time {
    ///             hour: 12,
    ///             minute: 13,
    ///             second: 14,
    ///             microsecond: 0,
    ///         },
    ///         offset: Some(-30600),
    ///     }
    /// );
    /// assert_eq!(dt.to_string(), "2000-02-29T12:13:14-08:30");
    /// ```
    /// (note: the string representation is still canonical ISO8601)
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
        let mut offset: Option<i32> = None;

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

                let h1 = get_digit!(bytes, position, InvalidCharTzHour) as i32;
                let h2 = get_digit!(bytes, position + 1, InvalidCharTzHour) as i32;

                let m1 = match bytes.get(position + 2) {
                    Some(b':') => {
                        position += 3;
                        get_digit!(bytes, position, InvalidCharTzMinute) as i32
                    }
                    Some(c) if (b'0'..=b'9').contains(c) => {
                        position += 2;
                        (c - b'0') as i32
                    }
                    _ => return Err(ParseError::InvalidCharTzMinute),
                };
                let m2 = get_digit!(bytes, position + 1, InvalidCharTzMinute) as i32;

                let minute_seconds = m1 * 600 + m2 * 60;
                if minute_seconds >= 3600 {
                    return Err(ParseError::OutOfRangeTzMinute);
                }

                let offset_val = sign * (h1 * 36000 + h2 * 3600 + minute_seconds);
                // TZ must be less than 24 hours to match python
                if offset_val.abs() >= 24 * 3600 {
                    return Err(ParseError::OutOfRangeTz);
                }
                offset = Some(offset_val);
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
    /// Input must be between `-11,676,096,000` (`1600-01-01T00:00:00`) and
    /// `253,402,300,799,000` (`9999-12-31T23:59:59.999999`) inclusive.
    ///
    /// If the absolute value is > 2e10 (`20,000,000,000`) it is interpreted as being in milliseconds.
    ///
    /// That means:
    /// * `20_000_000_000` is `2603-10-11T11:33:20`
    /// * `20_000_000_001` is `1970-08-20T11:33:20.001`
    /// * `-20_000_000_000` gives an error - `DateTooSmall` as it would be before 1600
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
    /// let d = DateTime::from_timestamp(1_654_619_320, 123).unwrap();
    /// assert_eq!(d.to_string(), "2022-06-07T16:28:40.000123");
    ///
    /// let d = DateTime::from_timestamp(1_654_619_320_123, 123_000).unwrap();
    /// assert_eq!(d.to_string(), "2022-06-07T16:28:40.246");
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

    /// Unix timestamp (seconds since epoch, 1970-01-01T00:00:00) omitting timezone offset
    /// (or equivalently comparing to 1970-01-01T00:00:00 in the same timezone as self)
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::DateTime;
    ///
    /// let dt = DateTime::from_timestamp(1_654_619_320, 123).unwrap();
    /// assert_eq!(dt.to_string(), "2022-06-07T16:28:40.000123");
    /// assert_eq!(dt.timestamp(), 1_654_619_320);
    ///
    /// let dt = DateTime::parse_str("1970-01-02T00:00").unwrap();
    /// assert_eq!(dt.timestamp(), 24 * 3600);
    /// ```
    pub fn timestamp(&self) -> i64 {
        self.date.timestamp() + self.time.total_seconds() as i64
    }

    /// Unix timestamp assuming epoch is in zulu timezone (1970-01-01T00:00:00Z) and accounting for
    /// timezone offset.
    ///
    /// This is effectively [Self::timestamp] minus [Self::offset], see [Self::partial_cmp] for details on
    /// why timezone offset is subtracted. If [Self::offset] if `None`, this is the same as [Self::timestamp].
    ///
    /// # Examples
    ///
    /// ```
    /// use speedate::DateTime;
    ///
    /// let dt_naive = DateTime::parse_str("1970-01-02T00:00").unwrap();
    /// assert_eq!(dt_naive.timestamp_tz(), 24 * 3600);
    ///
    /// let dt_zulu = DateTime::parse_str("1970-01-02T00:00Z").unwrap();
    /// assert_eq!(dt_zulu.timestamp_tz(), 24 * 3600);
    ///
    /// let dt_plus_1 = DateTime::parse_str("1970-01-02T00:00+01:00").unwrap();
    /// assert_eq!(dt_plus_1.timestamp_tz(), 23 * 3600);
    /// ```
    pub fn timestamp_tz(&self) -> i64 {
        let adjustment = match self.offset {
            Some(offset) => -offset as i64,
            None => 0,
        };
        self.timestamp() + adjustment
    }
}
