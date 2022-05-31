use std::fs::File;
use std::io::Read;

use not8601::{Date, DateTime, Duration, ParseError, Time};

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
    ($type:ty, $name:ident, error, $input:literal, $error:expr) => {
        paste::item! {
            #[allow(non_snake_case)]
            #[test]
            fn [< expect_ $name _ $error _error >]() {
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
    ($type:ty, $($name:ident: $ok_or_error:ident => $input:literal, $expected:expr;)*) => {
        $(
            expect_ok_or_error!($type, $name, $ok_or_error, $input, $expected);
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

param_tests! {
    Date,
    date_short_3: error => "123", TooShort;
    date_short_9: error => "2000:12:1", TooShort;
    date: error => "xxxx:12:31", InvalidCharYear;
    date_year_sep: error => "2020x12:13", InvalidCharDateSep;
    date_mo_sep: error => "2020-12x13", InvalidCharDateSep;
    date: error => "2020-13-01", OutOfRangeMonth;
    date: error => "2020-04-31", OutOfRangeDay;
    date_extra_space: error => "2020-04-01 ", ExtraCharacters;
    date_extra_xxx: error => "2020-04-01xxx", ExtraCharacters;
    // leap year dates
    date_simple: ok => "2020-04-01", "2020-04-01";
    date_normal_not_leap: ok => "2003-02-28", "2003-02-28";
    date_normal_not_leap: error => "2003-02-29", OutOfRangeDay;
    date_normal_leap_year: ok => "2004-02-29", "2004-02-29";
    date_special_100_not_leap: error => "1900-02-29", OutOfRangeDay;
    date_special_400_leap: ok => "2000-02-29", "2000-02-29";
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
    time: error => "xxx", TooShort;
    time: error => "xx:12", InvalidCharHour;
    time_sep_hour: error => "12x12", InvalidCharTimeSep;
    time: error => "12:x0", InvalidCharMinute;
    time_sep_min: error => "12:13x", ExtraCharacters;
    time: error => "12:13:x", InvalidCharSecond;
    time: error => "12:13:12.", SecondFractionMissing;
    time: error => "12:13:12.1234567", SecondFractionTooLong;
    time: error => "24:00:00", OutOfRangeHour;
    time: error => "23:60:00", OutOfRangeMinute;
    time: error => "23:59:60", OutOfRangeSecond;
    time_extra_x: error => "23:59:59xxx", ExtraCharacters;
    time_extra_space: error => "23:59:59 ", ExtraCharacters;
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
            offset: Some(120),
        }
    );
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14+02:00");
}

#[test]
fn datetime_tz_negative_2212() {
    // using U+2212 for negative timezones
    let dt = DateTime::parse_str("2020-01-01T12:13:14−02:15").unwrap();
    assert_eq!(dt.offset, Some(-135));
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
    dt_short_date: error => "xxx", TooShort;
    dt_short_time: error => "2020-01-01T12:0", TooShort;
    dt: error => "202x-01-01", InvalidCharYear;
    dt: error => "2020-01-01x", InvalidCharDateTimeSep;
    dt: error => "2020-01-01Txx:00", InvalidCharHour;
    dt_1: error => "2020-01-01T12:00:00x", InvalidCharTzSign;
    // same first byte as U+2212, different second b'\xe2\x89\x92'.decode()
    dt_2: error => "2020-01-01T12:00:00≒", InvalidCharTzSign;
    // same first and second bytes as U+2212, different third b'\xe2\x88\x93'.decode()
    dt_3: error => "2020-01-01T12:00:00∓", InvalidCharTzSign;
    dt: error => "2020-01-01T12:00:00+x", InvalidCharTzHour;
    dt: error => "2020-01-01T12:00:00+00x", InvalidCharTzMinute;
    dt_extra_space_z: error => "2020-01-01T12:00:00Z ", ExtraCharacters;
    dt_extra_space_tz1: error => "2020-01-01T12:00:00+00:00 ", ExtraCharacters;
    dt_extra_space_tz2: error => "2020-01-01T12:00:00+0000 ", ExtraCharacters;
    dt_extra_xxx: error => "2020-01-01T12:00:00Zxxx", ExtraCharacters;
}

#[test]
fn test_rfc_3339_values_txt() {
    let mut f = File::open("./tests/rfc-3339-values.txt").unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    let mut success = 0;
    for line in contents.split("\n") {
        if line.starts_with("#") || line.is_empty() {
            continue;
        } else if line.starts_with("date:") {
            Date::parse_str(line.trim_start_matches("date:").trim())
                .map_err(|e| panic!("error on line {:?}: {:?}", line, e))
                .unwrap();
        } else if line.starts_with("time:") {
            Time::parse_str(line.trim_start_matches("time:").trim())
                .map_err(|e| panic!("error on line {:?}: {:?}", line, e))
                .unwrap();
        } else if line.starts_with("dt:") {
            DateTime::parse_str(line.trim_start_matches("dt:").trim())
                .map_err(|e| panic!("error on line {:?}: {:?}", line, e))
                .unwrap();
        } else {
            panic!("unexpected line: {:?}", line);
        }
        success += 1;
    }
    println!("{} formats successfully parsed", success);
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
fn duration_fractions() {
    let d = Duration::parse_str("P1Y1DT2H0.5S").unwrap();
    assert_eq!(
        d,
        Duration {
            positive: true,
            day: 366,
            second: 7200,
            microsecond: 500_000
        }
    );
    assert_eq!(d.to_string(), "P1Y1DT7200.5S");
    assert_eq!(
        format!("{:?}", d),
        "Duration { positive: true, day: 366, second: 7200, microsecond: 500000 }"
    );
}

#[test]
fn duration_1() {
    let d = Duration::parse_str("P1DT1S").unwrap();
    assert_eq!(
        d,
        Duration {
            positive: true,
            day: 1,
            second: 1,
            microsecond: 0
        }
    );
    assert_eq!(d.to_string(), "P1DT1S");
}
