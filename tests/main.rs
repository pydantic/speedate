use std::fs::File;
use std::io::Read;

use not8601::{Date, DateTime, ParseError, Time};

/// macro for expected ParseError errors
macro_rules! expect_error {
    ($type:ty, $input:expr, $error:ident) => {
        match <$type>::parse_str($input) {
            Ok(t) => panic!("unexpectedly valid: {:?} -> {:?}", $input, t),
            Err(e) => assert_eq!(e, ParseError::$error),
        }
    };
}

/// macro to define many tests for unexpected errors
macro_rules! expect_error_tests {
    ($type:ty, $($name:ident: $input:expr, $error:ident;)*) => {
    $(
        paste::item! {
            #[allow(non_snake_case)]
            #[test]
            fn [< expect_error_ $name _ $error >]() {
                expect_error!($type, $input, $error);
            }
        }
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

expect_error_tests! {
    Date,
    date_short_3: "123", TooShort;
    date_short_9: "2000:12:1", TooShort;
    date: "xxxx:12:31", InvalidCharYear;
    date_year_sep: "2020x12:13", InvalidCharDateSep;
    date_mo_sep: "2020-12x13", InvalidCharDateSep;
    date: "2020-13-01", OutOfRangeMonth;
    date: "2020-04-31", OutOfRangeDay;
    date_extra_space: "2020-04-01 ", ExtraCharacters;
    date_extra_xxx: "2020-04-01xxx", ExtraCharacters;
}

#[test]
fn date_leap() {
    // normal not leap year
    assert_eq!(Date::parse_str("2003-02-28").unwrap().to_string(), "2003-02-28");
    expect_error!(Date, "2003-02-29", OutOfRangeDay);

    // normal leap year
    assert_eq!(Date::parse_str("2004-02-29").unwrap().to_string(), "2004-02-29");

    // special 100 not a leap year
    expect_error!(Date, "1900-02-29", OutOfRangeDay);

    // special 400 leap year
    assert_eq!(Date::parse_str("2000-02-29").unwrap().to_string(), "2000-02-29");
}

#[test]
fn time_fraction() {
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

#[test]
fn time_no_fraction() {
    let t = Time::parse_str("12:13:14").unwrap();
    assert_eq!(
        t,
        Time {
            hour: 12,
            minute: 13,
            second: 14,
            microsecond: 0,
        }
    );
    assert_eq!(t.to_string(), "12:13:14");
    assert_eq!(
        format!("{:?}", t),
        "Time { hour: 12, minute: 13, second: 14, microsecond: 0 }"
    );
}

#[test]
fn time_fraction_small() {
    let t = Time::parse_str("12:13:14.123").unwrap();
    assert_eq!(
        t,
        Time {
            hour: 12,
            minute: 13,
            second: 14,
            microsecond: 123000,
        }
    );
    assert_eq!(t.to_string(), "12:13:14.123");
}

#[test]
fn time_no_secs() {
    let t = Time::parse_str("12:13").unwrap();
    assert_eq!(
        t,
        Time {
            hour: 12,
            minute: 13,
            second: 0,
            microsecond: 0,
        }
    );
    assert_eq!(t.to_string(), "12:13:00");
}

expect_error_tests! {
    Time,
    time: "xxx", TooShort;
    time: "xx:12", InvalidCharHour;
    time_sep_hour: "12x12", InvalidCharTimeSep;
    time: "12:x0", InvalidCharMinute;
    time_sep_min: "12:13x", ExtraCharacters;
    time: "12:13:x", InvalidCharSecond;
    time: "12:13:12.", SecondFractionMissing;
    time: "12:13:12.1234567", SecondFractionTooLong;
    time: "24:00:00", OutOfRangeHour;
    time: "23:60:00", OutOfRangeMinute;
    time: "23:59:60", OutOfRangeSecond;
    time_extra_x: "23:59:59xxx", ExtraCharacters;
    time_extra_space: "23:59:59 ", ExtraCharacters;
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
fn datetime_longest() {
    let dt = DateTime::parse_str("2020-01-01T12:13:14.123456−02:15").unwrap();
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14.123456-02:15");
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
fn datetime_tz_negative() {
    let dt = DateTime::parse_str("2020-01-01T12:13:14-02:15").unwrap();
    assert_eq!(dt.offset, Some(-135));
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14-02:15");
}

#[test]
fn datetime_tz_negative_2212() {
    // using U+2212 for negative timezones
    let dt = DateTime::parse_str("2020-01-01T12:13:14−02:15").unwrap();
    assert_eq!(dt.offset, Some(-135));
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14-02:15");
}

#[test]
fn datetime_tz_negative_10() {
    let dt = DateTime::parse_str("2020-01-01T12:13:14-11:30").unwrap();
    assert_eq!(dt.offset, Some(-690));
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14-11:30");
}

#[test]
fn datetime_tz_no_colon() {
    let dt = DateTime::parse_str("2020-01-01T12:13:14+1234").unwrap();
    assert_eq!(dt.offset, Some(12 * 60 + 34));
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14+12:34");
}

#[test]
fn datetime_seconds_fraction_break() {
    let dt = DateTime::parse_str("2020-01-01 12:13:14.123z").unwrap();
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14.123Z");
}

#[test]
fn datetime_seconds_fraction_comma() {
    let dt = DateTime::parse_str("2020-01-01 12:13:14,123z").unwrap();
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14.123Z");
}

#[test]
fn datetime_underscore() {
    let dt = DateTime::parse_str("2020-01-01_12:13:14,123z").unwrap();
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14.123Z");
}

expect_error_tests! {
    DateTime,
    dt_short_date: "xxx", TooShort;
    dt_short_time: "2020-01-01T12:0", TooShort;
    dt: "202x-01-01", InvalidCharYear;
    dt: "2020-01-01x", InvalidCharDateTimeSep;
    dt: "2020-01-01Txx:00", InvalidCharHour;
    dt_1: "2020-01-01T12:00:00x", InvalidCharTzSign;
    // same first byte as U+2212, different second b'\xe2\x89\x92'.decode()
    dt_2: "2020-01-01T12:00:00≒", InvalidCharTzSign;
    // same first and second bytes as U+2212, different third b'\xe2\x88\x93'.decode()
    dt_3: "2020-01-01T12:00:00∓", InvalidCharTzSign;
    dt: "2020-01-01T12:00:00+x", InvalidCharTzHour;
    dt: "2020-01-01T12:00:00+00x", InvalidCharTzMinute;
    dt_extra_space_z: "2020-01-01T12:00:00Z ", ExtraCharacters;
    dt_extra_space_tz1: "2020-01-01T12:00:00+00:00 ", ExtraCharacters;
    dt_extra_space_tz2: "2020-01-01T12:00:00+0000 ", ExtraCharacters;
    dt_extra_xxx: "2020-01-01T12:00:00Zxxx", ExtraCharacters;
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
