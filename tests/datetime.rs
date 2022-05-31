use std::fs::File;
use std::io::Read;

use speedate::{Date, DateTime, ParseError, Time};

mod common;
use common::param_tests;

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
