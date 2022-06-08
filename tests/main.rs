use std::fs::File;
use std::io::Read;

use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use strum::EnumMessage;

use speedate::{Date, DateTime, Duration, ParseError, Time};

/// macro for expected values
macro_rules! expect_ok_or_error {
    ($type:ty, $name:ident, ok, $input:literal, $expected:expr) => {
        paste::item! {
            #[test]
            fn [< expect_ $name _ok >]() {
                let v = <$type>::parse_str($input).unwrap();
                assert_eq!(v.to_string(), $expected);
            }
        }
    };
    ($type:ty, $name:ident, err, $input:literal, $error:expr) => {
        paste::item! {
            #[test]
            fn [< expect_ $name _ $error:snake _error >]() {
                match <$type>::parse_str($input) {
                    Ok(t) => panic!("unexpectedly valid: {:?} -> {:?}", $input, t),
                    Err(e) => assert_eq!(e, ParseError::$error),
                }
            }
        }
    };
}

/// macro to define many tests for expected values
macro_rules! param_tests {
    ($type:ty, $($name:ident: $ok_or_err:ident => $input:literal, $expected:expr;)*) => {
        $(
            expect_ok_or_error!($type, $name, $ok_or_err, $input, $expected);
        )*
    }
}

#[test]
fn date() {
    let d = Date::parse_str("2020-01-01").unwrap();
    assert_eq!(
        d,
        Date {
            year: 2020,
            month: 1,
            day: 1
        }
    );
    assert_eq!(d.to_string(), "2020-01-01");
    assert_eq!(format!("{:?}", d), "Date { year: 2020, month: 1, day: 1 }");
}

#[test]
fn date_bytes_err() {
    // https://github.com/python/cpython/blob/5849af7a80166e9e82040e082f22772bd7cf3061/Lib/test/datetimetester.py#L3237
    // bytes of '\ud800'
    let bytes: Vec<u8> = vec![92, 117, 100, 56, 48, 48];
    match Date::parse_bytes(&bytes) {
        Ok(_) => panic!("unexpectedly valid"),
        Err(e) => assert_eq!(e, ParseError::TooShort),
    }
    let bytes: Vec<u8> = vec!['2' as u8, '0' as u8, '0' as u8, '0' as u8, 92, 117, 100, 56, 48, 48];
    match Date::parse_bytes(&bytes) {
        Ok(_) => panic!("unexpectedly valid"),
        Err(e) => assert_eq!(e, ParseError::InvalidCharDateSep),
    }
}

#[test]
fn error_str() {
    let error = match Date::parse_str("123") {
        Ok(_) => panic!("unexpectedly valid"),
        Err(e) => e,
    };
    assert_eq!(error, ParseError::TooShort);
    assert_eq!(error.to_string(), "too_short");
    assert_eq!(error.get_documentation(), Some("input is too short"));
}

param_tests! {
    Date,
    date_short_3: err => "123", TooShort;
    date_short_9: err => "2000:12:1", TooShort;
    date: err => "xxxx:12:31", InvalidCharYear;
    date_year_sep: err => "2020x12:13", InvalidCharDateSep;
    date_mo_sep: err => "2020-12x13", InvalidCharDateSep;
    date: err => "2020-13-01", OutOfRangeMonth;
    date: err => "2020-04-31", OutOfRangeDay;
    date_extra_space: err => "2020-04-01 ", ExtraCharacters;
    date_extra_xxx: err => "2020-04-01xxx", ExtraCharacters;
    // leap year dates
    date_simple: ok => "2020-04-01", "2020-04-01";
    date_normal_not_leap: ok => "2003-02-28", "2003-02-28";
    date_normal_not_leap: err => "2003-02-29", OutOfRangeDay;
    date_normal_leap_year: ok => "2004-02-29", "2004-02-29";
    date_special_100_not_leap: err => "1900-02-29", OutOfRangeDay;
    date_special_400_leap: ok => "2000-02-29", "2000-02-29";
}

#[test]
fn date_from_timestamp_extremes() {
    match Date::from_timestamp(i64::MIN) {
        Ok(dt) => panic!("unexpectedly valid, {}", dt),
        Err(e) => assert_eq!(e, ParseError::DateTooSmall),
    }
    match Date::from_timestamp(i64::MAX) {
        Ok(dt) => panic!("unexpectedly valid, {}", dt),
        Err(e) => assert_eq!(e, ParseError::DateTooLarge),
    }
    match Date::from_timestamp(-30_610_224_000_000) {
        Ok(dt) => panic!("unexpectedly valid, {}", dt),
        Err(e) => assert_eq!(e, ParseError::DateTooSmall),
    }
    let d = Date::from_timestamp(-11_676_096_000 + 1000).unwrap();
    assert_eq!(d.to_string(), "1600-01-01");
    let d = Date::from_timestamp(-11_673_417_600).unwrap();
    assert_eq!(d.to_string(), "1600-02-01");
    let d = Date::from_timestamp(253_402_300_799_000).unwrap();
    assert_eq!(d.to_string(), "9999-12-31");
    match Date::from_timestamp(253_402_300_800_000) {
        Ok(dt) => panic!("unexpectedly valid, {}", dt),
        Err(e) => assert_eq!(e, ParseError::DateTooLarge),
    }
}

#[test]
fn date_watershed() {
    let dt = Date::from_timestamp(20_000_000_000).unwrap();
    assert_eq!(dt.to_string(), "2603-10-11");
    let dt = Date::from_timestamp(20_000_000_001).unwrap();
    assert_eq!(dt.to_string(), "1970-08-20");
    match Date::from_timestamp(-20_000_000_000) {
        Ok(d) => panic!("unexpectedly valid, {}", d),
        Err(e) => assert_eq!(e, ParseError::DateTooSmall),
    }
    let dt = Date::from_timestamp(-20_000_000_001).unwrap();
    assert_eq!(dt.to_string(), "1969-05-14");
}

#[test]
fn date_from_timestamp_milliseconds() {
    let d1 = Date::from_timestamp(1_654_472_524).unwrap();
    assert_eq!(
        d1,
        Date {
            year: 2022,
            month: 6,
            day: 5
        }
    );
    let d2 = Date::from_timestamp(1_654_472_524_000).unwrap();
    assert_eq!(d2, d1);
}

fn try_date_timestamp(ts: i64) {
    let chrono_date = NaiveDateTime::from_timestamp(ts, 0).date();
    let d = Date::from_timestamp(ts).unwrap();
    // println!("{} => {:?}", ts, d);
    assert_eq!(
        d,
        Date {
            year: chrono_date.year() as u16,
            month: chrono_date.month() as u8,
            day: chrono_date.day() as u8,
        },
        "timestamp: {} => {}",
        ts,
        chrono_date
    );
}

#[test]
fn date_from_timestamp_range() {
    for ts in (0..4_000_000_000).step_by(86_400) {
        try_date_timestamp(ts);
        try_date_timestamp(ts + 40_000);
        try_date_timestamp(-ts);
        try_date_timestamp(-ts - 40_000);
    }
}

macro_rules! date_from_timestamp {
    ($($year:literal, $month:literal, $day:literal;)*) => {
        $(
        paste::item! {
            #[test]
            fn [< date_from_timestamp_ $year _ $month _ $day >]() {
                let chrono_date = NaiveDate::from_ymd($year, $month, $day);
                let ts = chrono_date.and_hms(0, 0, 0).timestamp();
                let d = Date::from_timestamp(ts).unwrap();
                assert_eq!(
                    d,
                    Date {
                        year: $year,
                        month: $month,
                        day: $day,
                    },
                    "timestamp: {} => {}",
                    ts,
                    chrono_date
                );
            }
        }
        )*
    }
}

date_from_timestamp! {
    1970, 1, 1;
    1970, 1, 31;
    1970, 2, 1;
    1970, 2, 28;
    1970, 3, 1;
    1600, 1, 1;
    1601, 1, 1;
    1700, 1, 1;
    1842, 8, 20;
    1900, 1, 1;
    1900, 6, 1;
    1901, 1, 1;
    1904, 1, 1;
    1904, 2, 29;
    1904, 6, 1;
    1924, 6, 1;
    2200, 1, 1;
}

macro_rules! time_from_timestamp {
    ($($ts_secs:literal, $ts_micro:literal => $hour:literal, $minute:literal, $second:literal, $microsecond:literal;)*) => {
        $(
        paste::item! {
            #[test]
            fn [< time_from_timestamp_ $hour _ $minute _ $second _ $microsecond >]() {
                let d = Time::from_timestamp($ts_secs, $ts_micro).unwrap();
                assert_eq!(d, Time {
                    hour: $hour,
                    minute: $minute,
                    second: $second,
                    microsecond: $microsecond,
                },
                "timestamp: {} => {}:{}:{}.{}",
                $ts_secs,
                $hour, $minute, $second, $microsecond);
            }
        }
        )*
    }
}

time_from_timestamp! {
    0, 0 => 0, 0, 0, 0;
    1, 0 => 0, 0, 1, 0;
    3600, 0 => 1, 0, 0, 0;
    3700, 0 => 1, 1, 40, 0;
    86399, 0 => 23, 59, 59, 0;
    0, 100 => 0, 0, 0, 100;
    0, 5_000_000 => 0, 0, 5, 0;
    36_005, 1_500_000 => 10, 0, 6, 500_000;
}

#[test]
fn time_from_timestamp_error() {
    match Time::from_timestamp(86400, 0) {
        Ok(_) => panic!("unexpectedly valid"),
        Err(e) => assert_eq!(e, ParseError::TimeTooLarge),
    }
    match Time::from_timestamp(86390, 10_000_000) {
        Ok(_) => panic!("unexpectedly valid"),
        Err(e) => assert_eq!(e, ParseError::TimeTooLarge),
    }
    match Time::from_timestamp(u32::MAX, u32::MAX) {
        Ok(_) => panic!("unexpectedly valid"),
        Err(e) => assert_eq!(e, ParseError::TimeTooLarge),
    }
}

fn try_datetime_timestamp(chrono_dt: NaiveDateTime) {
    let ts = chrono_dt.timestamp();
    let dt = DateTime::from_timestamp(ts, chrono_dt.nanosecond() / 1_000).unwrap();
    // println!("{} ({}) => {}", ts, chrono_dt, dt);
    assert_eq!(
        dt,
        DateTime {
            date: Date {
                year: chrono_dt.year() as u16,
                month: chrono_dt.month() as u8,
                day: chrono_dt.day() as u8,
            },
            time: Time {
                hour: chrono_dt.hour() as u8,
                minute: chrono_dt.minute() as u8,
                second: chrono_dt.second() as u8,
                microsecond: chrono_dt.nanosecond() as u32 / 1_000,
            },
            offset: None,
        },
        "timestamp: {} => {}",
        ts,
        chrono_dt
    );
}

macro_rules! datetime_from_timestamp {
    ($($year:literal, $month:literal, $day:literal, $hour:literal, $minute:literal, $second:literal, $microsecond:literal;)*) => {
        $(
        paste::item! {
            #[test]
            fn [< datetime_from_timestamp_ $year _ $month _ $day _t_ $hour _ $minute _ $second _ $microsecond >]() {
                let chrono_dt = NaiveDate::from_ymd($year, $month, $day).and_hms_nano($hour, $minute, $second, $microsecond * 1_000);
                try_datetime_timestamp(chrono_dt);
            }
        }
        )*
    }
}

datetime_from_timestamp! {
    1970, 1, 1, 0, 0, 0, 0;
    1970, 1, 1, 0, 0, 1, 0;
    1970, 1, 1, 0, 1, 0, 0;
    1970, 1, 2, 0, 0, 0, 0;
    1970, 1, 2, 0, 0, 0, 500000;
    1969, 12, 30, 15, 51, 29, 10630;
}

#[test]
fn datetime_from_timestamp_range() {
    for ts in (0..157_766_400).step_by(757) {
        try_datetime_timestamp(NaiveDateTime::from_timestamp(ts, 0));
        try_datetime_timestamp(NaiveDateTime::from_timestamp(-ts, 0));
    }
}

#[test]
fn datetime_from_timestamp_specific() {
    let dt = DateTime::from_timestamp(-11676095999, 4291493).unwrap();
    assert_eq!(dt.to_string(), "1600-01-01T00:00:05.291493");
    let dt = DateTime::from_timestamp(-1, 1667444).unwrap();
    assert_eq!(dt.to_string(), "1970-01-01T00:00:00.667444");
    let dt = DateTime::from_timestamp(32_503_680_000_000, 0).unwrap();
    assert_eq!(dt.to_string(), "3000-01-01T00:00:00");
    let dt = DateTime::from_timestamp(-11_676_096_000, 0).unwrap();
    assert_eq!(dt.to_string(), "1600-01-01T00:00:00");
    let dt = DateTime::from_timestamp(1_095_216_660_480, 3221223).unwrap();
    assert_eq!(dt.to_string(), "2004-09-15T02:51:03.701223");

    let d = DateTime::from_timestamp(253_402_300_799_000, 999999).unwrap();
    assert_eq!(d.to_string(), "9999-12-31T23:59:59.999999");
    match Date::from_timestamp(253_402_300_800_000) {
        Ok(dt) => panic!("unexpectedly valid, {}", dt),
        Err(e) => assert_eq!(e, ParseError::DateTooLarge),
    }
}

#[test]
fn datetime_watershed() {
    let dt = DateTime::from_timestamp(20_000_000_000, 0).unwrap();
    assert_eq!(dt.to_string(), "2603-10-11T11:33:20");
    let dt = DateTime::from_timestamp(20_000_000_001, 0).unwrap();
    assert_eq!(dt.to_string(), "1970-08-20T11:33:20.001");
    match DateTime::from_timestamp(-20_000_000_000, 0) {
        Ok(dt) => panic!("unexpectedly valid, {}", dt),
        Err(e) => assert_eq!(e, ParseError::DateTooSmall),
    }
    let dt = DateTime::from_timestamp(-20_000_000_001, 0).unwrap();
    assert_eq!(dt.to_string(), "1969-05-14T12:26:39.999");
}

#[test]
fn time() {
    let t = Time::parse_str("12:13:14.123456").unwrap();
    assert_eq!(
        t,
        Time {
            hour: 12,
            minute: 13,
            second: 14,
            microsecond: 123456,
        }
    );
    assert_eq!(t.to_string(), "12:13:14.123456");
    assert_eq!(
        format!("{:?}", t),
        "Time { hour: 12, minute: 13, second: 14, microsecond: 123456 }"
    );
}

param_tests! {
    Time,
    time_min: ok => "00:00:00.000000", "00:00:00";
    time_max: ok => "23:59:59.999999", "23:59:59.999999";
    time_no_fraction: ok => "12:13:14", "12:13:14";
    time_fraction_small: ok => "12:13:14.123", "12:13:14.123";
    time_no_sec: ok => "12:13", "12:13:00";
    time: err => "xxx", TooShort;
    time: err => "xx:12", InvalidCharHour;
    time_sep_hour: err => "12x12", InvalidCharTimeSep;
    time: err => "12:x0", InvalidCharMinute;
    time_sep_min: err => "12:13x", ExtraCharacters;
    time: err => "12:13:x", InvalidCharSecond;
    time: err => "12:13:12.", SecondFractionMissing;
    time: err => "12:13:12.1234567", SecondFractionTooLong;
    time: err => "24:00:00", OutOfRangeHour;
    time: err => "23:60:00", OutOfRangeMinute;
    time: err => "23:59:60", OutOfRangeSecond;
    time_extra_x: err => "23:59:59xxx", ExtraCharacters;
    time_extra_space: err => "23:59:59 ", ExtraCharacters;
}

#[test]
fn datetime_naive() {
    let dt = DateTime::parse_str("2020-01-01T12:13:14.123456").unwrap();
    assert_eq!(
        dt,
        DateTime {
            date: Date {
                year: 2020,
                month: 1,
                day: 1,
            },
            time: Time {
                hour: 12,
                minute: 13,
                second: 14,
                microsecond: 123456,
            },
            offset: None,
        }
    );
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14.123456");
    assert_eq!(
        format!("{:?}", dt),
        "DateTime { date: Date { year: 2020, month: 1, day: 1 }, time: Time { hour: 12, minute: 13, second: 14, microsecond: 123456 }, offset: None }"
    );
}

#[test]
fn datetime_tz_z() {
    let dt = DateTime::parse_str("2020-01-01 12:13:14z").unwrap();
    assert_eq!(
        dt,
        DateTime {
            date: Date {
                year: 2020,
                month: 1,
                day: 1,
            },
            time: Time {
                hour: 12,
                minute: 13,
                second: 14,
                microsecond: 0,
            },
            offset: Some(0),
        }
    );
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14Z");
}

#[test]
fn datetime_bytes() {
    let dt = DateTime::parse_bytes(b"2020-01-01 12:13:14z").unwrap();
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14Z");
}

#[test]
fn datetime_tz_2hours() {
    let dt = DateTime::parse_str("2020-01-01T12:13:14+02:00").unwrap();
    assert_eq!(
        dt,
        DateTime {
            date: Date {
                year: 2020,
                month: 1,
                day: 1,
            },
            time: Time {
                hour: 12,
                minute: 13,
                second: 14,
                microsecond: 0,
            },
            offset: Some(7_200),
        }
    );
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14+02:00");
}

#[test]
fn datetime_tz_negative_2212() {
    // using U+2212 for negative timezones
    let dt = DateTime::parse_str("2020-01-01T12:13:14−02:15").unwrap();
    assert_eq!(dt.offset, Some(-8100));
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14-02:15");
}

param_tests! {
    DateTime,
    dt_longest: ok => "2020-01-01T12:13:14.123456−02:15", "2020-01-01T12:13:14.123456-02:15";
    dt_tz_negative: ok => "2020-01-01T12:13:14-02:15", "2020-01-01T12:13:14-02:15";
    dt_tz_negative_10: ok => "2020-01-01T12:13:14-11:30", "2020-01-01T12:13:14-11:30";
    dt_tz_no_colon: ok => "2020-01-01T12:13:14+1234", "2020-01-01T12:13:14+12:34";
    dt_seconds_fraction_break: ok => "2020-01-01 12:13:14.123z", "2020-01-01T12:13:14.123Z";
    dt_seconds_fraction_comma: ok => "2020-01-01 12:13:14,123z", "2020-01-01T12:13:14.123Z";
    dt_underscore: ok => "2020-01-01_12:13:14,123z", "2020-01-01T12:13:14.123Z";
    dt_short_date: err => "xxx", TooShort;
    dt_short_time: err => "2020-01-01T12:0", TooShort;
    dt: err => "202x-01-01", InvalidCharYear;
    dt: err => "2020-01-01x", InvalidCharDateTimeSep;
    dt: err => "2020-01-01Txx:00", InvalidCharHour;
    dt_1: err => "2020-01-01T12:00:00x", InvalidCharTzSign;
    // same first byte as U+2212, different second b'\xe2\x89\x92'.decode()
    dt_2: err => "2020-01-01T12:00:00≒", InvalidCharTzSign;
    // same first and second bytes as U+2212, different third b'\xe2\x88\x93'.decode()
    dt_3: err => "2020-01-01T12:00:00∓", InvalidCharTzSign;
    dt: err => "2020-01-01T12:00:00+x", InvalidCharTzHour;
    dt: err => "2020-01-01T12:00:00+00x", InvalidCharTzMinute;
    dt_extra_space_z: err => "2020-01-01T12:00:00Z ", ExtraCharacters;
    dt_extra_space_tz1: err => "2020-01-01T12:00:00+00:00 ", ExtraCharacters;
    dt_extra_space_tz2: err => "2020-01-01T12:00:00+0000 ", ExtraCharacters;
    dt_extra_xxx: err => "2020-01-01T12:00:00Zxxx", ExtraCharacters;
    tz_pos_30hr: ok => "2020-01-01T12:00:00+30:00", "2020-01-01T12:00:00+30:00";
    tz_neg_30hr: ok => "2020-01-01T12:00:00-30:00", "2020-01-01T12:00:00-30:00";
    tz_60mins: err => "2020-01-01T12:00:00+00:60", OutOfRangeTzMinute;
    tz_pos_gt30hr: err => "2020-01-01T12:00:00+30:01", OutOfRangeTz;
    tz_neg_gt30hr: err => "2020-01-01T12:00:00-30:01", OutOfRangeTz;
    tz_pos_99hr: err => "2020-01-01T12:00:00+99:59", OutOfRangeTz;
    tz_neg_99hr: err => "2020-01-01T12:00:00-99:59", OutOfRangeTz;
}

fn extract_values(line: &str, prefix: &str) -> (String, String) {
    let parts: Vec<&str> = line.trim_start_matches(prefix).split("->").collect();
    assert_eq!(parts.len(), 2);
    (parts[0].trim().to_string(), parts[1].trim().to_string())
}

#[test]
fn test_ok_values_txt() {
    let mut f = File::open("./tests/values_ok.txt").unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    let mut success = 0;
    for (i, line) in contents.split("\n").enumerate() {
        let line_no = i + 1;
        if line.starts_with("#") || line.is_empty() {
            continue;
        } else if line.starts_with("date:") {
            let (input, expected_str) = extract_values(line, "date:");
            let d = Date::parse_str(&input)
                .map_err(|e| panic!("error on line {} {:?}: {:?}", line_no, line, e))
                .unwrap();
            assert_eq!(d.to_string(), expected_str, "error on line {}", line_no);
        } else if line.starts_with("time:") {
            let (input, expected_str) = extract_values(line, "time:");
            let t = Time::parse_str(&input)
                .map_err(|e| panic!("error on line {} {:?}: {:?}", line_no, line, e))
                .unwrap();
            assert_eq!(t.to_string(), expected_str, "error on line {}", line_no);
        } else if line.starts_with("dt:") {
            let (input, expected_str) = extract_values(line, "dt:");
            let dt = DateTime::parse_str(&input)
                .map_err(|e| panic!("error on line {} {:?}: {:?}", line_no, line, e))
                .unwrap();
            assert_eq!(dt.to_string(), expected_str, "error on line {}", line_no);
        } else {
            panic!("unexpected line: {:?}", line);
        }
        success += 1;
    }
    println!("{} formats successfully parsed", success);
}

#[test]
fn test_err_values_txt() {
    let mut f = File::open("./tests/values_err.txt").unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    let mut success = 0;
    for (i, line) in contents.split("\n").enumerate() {
        let line_no = i + 1;
        if line.starts_with("#") || line.is_empty() {
            continue;
        }
        match DateTime::parse_str(line.trim()) {
            Ok(_) => panic!("unexpected valid line {}: {:?}", line_no, line),
            Err(_) => (),
        }
        success += 1;
    }
    println!("{} correctly invalid", success);
}

#[test]
fn duration_simple() {
    let d = Duration::parse_str("P1Y").unwrap();
    assert_eq!(
        d,
        Duration {
            positive: true,
            day: 365,
            second: 0,
            microsecond: 0
        }
    );
    assert_eq!(d.to_string(), "P1Y");
}

#[test]
fn duration_total_seconds() {
    let d = Duration::parse_str("P1MT1.5S").unwrap();
    assert_eq!(
        d,
        Duration {
            positive: true,
            day: 30,
            second: 1,
            microsecond: 500_000
        }
    );
    assert_eq!(d.to_string(), "P30DT1.5S");
    assert_eq!(d.signed_total_seconds(), 30 * 86_400 + 1);
    assert_eq!(d.signed_microseconds(), 500_000);
}

#[test]
fn duration_total_seconds_neg() {
    let d = Duration::parse_str("-P1DT42.123456S").unwrap();
    assert_eq!(
        d,
        Duration {
            positive: false,
            day: 1,
            second: 42,
            microsecond: 123_456
        }
    );
    assert_eq!(d.to_string(), "-P1DT42.123456S");
    assert_eq!(d.signed_total_seconds(), -86_442);
    assert_eq!(d.signed_microseconds(), -123_456);
}

#[test]
fn duration_fractions() {
    let d = Duration::parse_str("P1.123W").unwrap();
    assert_eq!(
        d,
        Duration {
            positive: true,
            day: 7,
            second: 74390,
            microsecond: 400_000
        }
    );
}

#[test]
fn duration_new_normalise() {
    let d = Duration::new(false, 1, 86500, 1_000_123);
    assert_eq!(
        d,
        Duration {
            positive: false,
            day: 2,
            second: 101,
            microsecond: 123,
        }
    );
}

#[test]
fn duration_new_normalise2() {
    let d = Duration::new(true, 0, 0, 1_000_000);
    assert_eq!(
        d,
        Duration {
            positive: true,
            day: 0,
            second: 1,
            microsecond: 0,
        }
    );
}

param_tests! {
    Duration,
    duration_too_short1: err => "", TooShort;
    duration_too_short2: err => "+", TooShort;
    duration_too_short3: err => "P", TooShort;
    duration_1y: ok => "P1Y", "P1Y";
    duration_123y: ok => "P123Y", "P123Y";
    duration_123_8y: ok => "P123.8Y", "P123Y292D";
    duration_1m: ok => "P1M", "P30D";
    duration_1_5m: ok => "P1.5M", "P45D";
    duration_1w: ok => "P1W", "P7D";
    duration_1_1w: ok => "P1.1W", "P7DT60480S";
    duration_1_123w: ok => "P1.123W", "P7DT74390.4S";
    duration_simple_negative: ok => "-P1Y", "-P1Y";
    duration_simple_positive: ok => "+P1Y", "P1Y";
    duration_fraction1: ok => "PT0.555555S", "PT0.555555S";
    duration_fraction2: ok => "P1Y1DT2H0.5S", "P1Y1DT7200.5S";
    duration_1: ok => "P1DT1S", "P1DT1S";
    duration_all: ok => "P1Y2M3DT4H5M6S", "P1Y63DT14706S";
    duration: err => "PD", DurationInvalidNumber;
    duration: err => "P1DT1MT1S", DurationTRepeated;
    duration: err => "P1DT1.1M1S", DurationInvalidFraction;
    duration: err => "P1DT1X", DurationInvalidTimeUnit;
    duration_invalid_day_unit1: err => "P1X", DurationInvalidDateUnit;
    duration_invalid_day_unit2: err => "P1", DurationInvalidDateUnit;
    duration_time_42s: ok => "00:00:42", "PT42S";
    duration_time_1m: ok => "00:01", "PT60S";
    duration_time_1h_2m_3s: ok => "01:02:03", "PT3723S";
    duration_time_fraction: ok => "00:01:03.123", "PT63.123S";
    duration_time_extra: err => "00:01:03.123x", ExtraCharacters;
    duration_days_1day1: ok => "1 day", "P1D";
    duration_days_1day2: ok => "1day", "P1D";
    duration_days_1day3: ok => "1 day,", "P1D";
    duration_days_1day4: ok => "1 day, ", "P1D";
    duration_days_1day5: ok => "1days", "P1D";
    duration_days_1day6: ok => "1DAYS", "P1D";
    duration_days_1day7: ok => "1d", "P1D";
    duration_days_1day8: ok => "1d ", "P1D";
    duration_days_too_short: err => "x", DurationInvalidNumber;
    duration_days_invalid1: err => "1x", DurationInvalidDays;
    duration_days_invalid2: err => "1dx", TooShort;
    duration_days_invalid3: err => "1da", DurationInvalidDays;
    duration_days_invalid4: err => "1", DurationInvalidDays;
    duration_days_invalid5: err => "1 ", DurationInvalidDays;
    duration_days_invalid6: err => "1 x", DurationInvalidDays;
    duration_days_neg: ok => "-1 day", "-P1D";
    duration_days_pos: ok => "+1 day", "P1D";
    duration_days_123days: ok => "123days", "P123D";
    duration_days_time: ok => "1 day 00:00:42", "P1DT42S";
    duration_days_time_neg: ok => "-1 day 00:00:42", "-P1DT42S";
    duration_exceeds_day: ok => "PT86500S", "P1DT100S";
    duration_days_time_too_shoert: err => "1 day 00:", TooShort;
    duration_days_time_wrong: err => "1 day 00:xx", InvalidCharMinute;
    duration_days_time_extra: err => "1 day 00:00:00.123 ", ExtraCharacters;
}
