use std::fs::File;
use std::io::Read;

use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike, Utc as ChronoUtc};

use speedate::{Date, DateTime, ParseError, Time};

#[path = "./utils.rs"]
mod utils;
use utils::param_tests;

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
    assert_eq!(dt.timestamp(), ts);
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
fn datetime_now() {
    let speedate_now = DateTime::now(0).unwrap();
    let chrono_now = ChronoUtc::now();
    let diff = speedate_now.timestamp() as f64 - chrono_now.timestamp() as f64;
    assert!(diff.abs() < 0.1);
}

#[test]
fn datetime_now_offset() {
    let speedate_now = DateTime::now(3600).unwrap();
    let chrono_now = ChronoUtc::now();
    let diff = speedate_now.timestamp() as f64 - chrono_now.timestamp() as f64 - 3600.0;
    assert!(diff.abs() < 0.1);
    let diff = speedate_now.timestamp_tz() as f64 - chrono_now.timestamp() as f64;
    assert!(diff.abs() < 0.1);
}

#[test]
fn datetime_with_tz_offset() {
    let dt_z = DateTime::parse_str("2022-01-01T12:13:14.567+00:00").unwrap();

    let dt_m8 = dt_z.with_timezone_offset(Some(-8 * 3600)).unwrap();
    assert_eq!(dt_m8.to_string(), "2022-01-01T12:13:14.567-08:00");

    let dt_naive = dt_z.with_timezone_offset(None).unwrap();
    assert_eq!(dt_naive.to_string(), "2022-01-01T12:13:14.567");

    let dt_naive = DateTime::parse_str("2000-01-01T00:00:00").unwrap();

    let dt_p16 = dt_naive.with_timezone_offset(Some(16 * 3600)).unwrap();
    assert_eq!(dt_p16.to_string(), "2000-01-01T00:00:00+16:00");

    let error = match dt_naive.with_timezone_offset(Some(86_400)) {
        Ok(_) => panic!("unexpectedly valid"),
        Err(e) => e,
    };
    assert_eq!(error, ParseError::OutOfRangeTz);
}

#[test]
fn datetime_in_timezone() {
    let dt_z = DateTime::parse_str("2000-01-01T15:00:00.567Z").unwrap();

    let dt_p1 = dt_z.in_timezone(3_600).unwrap();
    assert_eq!(dt_p1.to_string(), "2000-01-01T16:00:00.567+01:00");

    let dt_m2 = dt_z.in_timezone(-7_200).unwrap();
    assert_eq!(dt_m2.to_string(), "2000-01-01T13:00:00.567-02:00");

    let dt_naive = DateTime::parse_str("2000-01-01T00:00:00").unwrap();
    let error = match dt_naive.in_timezone(3_600) {
        Ok(_) => panic!("unexpectedly valid"),
        Err(e) => e,
    };
    assert_eq!(error, ParseError::TzRequired);

    let error = match dt_z.in_timezone(86_400) {
        Ok(_) => panic!("unexpectedly valid"),
        Err(e) => e,
    };
    assert_eq!(error, ParseError::OutOfRangeTz);
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

#[test]
fn datetime_timestamp() {
    let dt = DateTime::from_timestamp(1_000_000_000, 999_999).unwrap();
    assert_eq!(dt.to_string(), "2001-09-09T01:46:40.999999");
    assert_eq!(dt.timestamp(), 1_000_000_000);

    // using ms unix timestamp
    let dt = DateTime::from_timestamp(1_000_000_000_000, 999_999).unwrap();
    assert_eq!(dt.to_string(), "2001-09-09T01:46:40.999999");
    assert_eq!(dt.timestamp(), 1_000_000_000);
    // using ms unix timestamp

    let d_naive = DateTime::parse_str("1970-01-02T00:00").unwrap();
    assert_eq!(d_naive.timestamp(), 86400);
}

#[test]
fn datetime_timestamp_tz() {
    let t_naive = DateTime::parse_str("1970-01-02T00:00").unwrap();
    assert_eq!(t_naive.timestamp(), 24 * 3600);
    assert_eq!(t_naive.timestamp_tz(), 24 * 3600);

    let dt_zulu = DateTime::parse_str("1970-01-02T00:00Z").unwrap();
    assert_eq!(dt_zulu.timestamp(), 24 * 3600);
    assert_eq!(dt_zulu.timestamp_tz(), 24 * 3600);

    let dt_plus_1 = DateTime::parse_str("1970-01-02T00:00+01:00").unwrap();
    assert_eq!(dt_plus_1.timestamp(), 24 * 3600);
    assert_eq!(dt_plus_1.timestamp_tz(), 23 * 3600);
}

#[test]
fn datetime_comparison_naive() {
    let dt1 = DateTime::parse_str("2020-02-03T04:05:06.07").unwrap();
    let dt2 = DateTime::parse_str("2021-01-02T03:04:05.06").unwrap();

    assert!(dt2 > dt1);
    assert!(dt2 >= dt1);
    assert!(dt2 >= dt2.clone());
    assert!(dt1 < dt2);
    assert!(dt1 <= dt2);
    assert!(dt1 <= dt1.clone());

    let dt3 = DateTime::parse_str("2020-02-03T04:05:06.123").unwrap();
    let dt4 = DateTime::parse_str("2020-02-03T04:05:06.124").unwrap();
    assert!(dt4 > dt3);
    assert!(dt3 < dt4);
}

#[test]
fn datetime_comparison_timezone() {
    let dt1 = DateTime::parse_str("2000-01-01T00:00:00+01:00").unwrap();
    let dt2 = DateTime::parse_str("2000-01-01T00:00:00+02:00").unwrap();

    assert!(dt1 > dt2);
    assert!(dt1 >= dt2);
    assert!(dt2 < dt1);
    assert!(dt2 <= dt1);

    let dt3 = DateTime::parse_str("2000-01-01T00:00:00").unwrap();

    assert!(dt1 >= dt3);
    assert!(dt3 <= dt1);

    let dt4 = DateTime::parse_str("1970-01-01T04:00:00.222+02:00").unwrap();
    assert_eq!(dt4.timestamp_tz(), 2 * 3600);
    let dt5 = DateTime::parse_str("1970-01-01T03:00:00Z").unwrap();
    assert_eq!(dt5.timestamp_tz(), 3 * 3600);
    assert!(dt5 > dt4);
    assert_eq!(dt4 > dt4.clone(), false);

    // assert that microseconds are used for comparison here
    let dt6 = DateTime::parse_str("1970-01-01T04:00:00.333+02:00").unwrap();
    assert_eq!(dt6.timestamp_tz(), 2 * 3600);
    assert!(dt6 > dt4);
    assert_ne!(dt6, dt4);

    // even on different dates, tz has an effect
    let dt7 = DateTime::parse_str("2022-01-01T23:00:00Z").unwrap();
    let dt8 = DateTime::parse_str("2022-01-02T01:00:00+03:00").unwrap();
    assert!(dt7 > dt8);
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
    tz_pos_2359: ok => "2020-01-01T12:00:00+23:59", "2020-01-01T12:00:00+23:59";
    tz_neg_2359: ok => "2020-01-01T12:00:00-23:59", "2020-01-01T12:00:00-23:59";
    tz_60mins: err => "2020-01-01T12:00:00+00:60", OutOfRangeTzMinute;
    tz_pos_gt2359: err => "2020-01-01T12:00:00+24:00", OutOfRangeTz;
    tz_neg_gt2359: err => "2020-01-01T12:00:00-24:00", OutOfRangeTz;
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
