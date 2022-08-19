use chrono::{Datelike, FixedOffset as ChronoFixedOffset, NaiveDate, NaiveDateTime, Utc as ChronoUtc};
use strum::EnumMessage;

use speedate::{Date, ParseError};

#[path = "./utils.rs"]
mod utils;
use utils::param_tests;

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

fn try_date_timestamp(ts: i64, check_timestamp: bool) {
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
    if check_timestamp {
        assert_eq!(d.timestamp(), ts);
    }
}

#[test]
fn date_from_timestamp_range() {
    for ts in (0..4_000_000_000).step_by(86_400) {
        try_date_timestamp(ts, true);
        try_date_timestamp(ts + 40_000, false);
        try_date_timestamp(-ts, true);
        try_date_timestamp(-ts - 40_000, false);
    }
}

#[test]
fn date_comparison() {
    let d1 = Date::parse_str("2020-02-03").unwrap();
    let d2 = Date::parse_str("2021-01-02").unwrap();
    assert!(d1 < d2);
    assert!(d1 <= d2);
    assert!(d1 <= d1.clone());
    assert!(d2 > d1);
    assert!(d2 >= d1);
    assert!(d2 >= d2.clone());
}

#[test]
fn date_timestamp() {
    let d = Date::from_timestamp(1_654_560_000).unwrap();
    assert_eq!(d.to_string(), "2022-06-07");
    assert_eq!(d.timestamp(), 1_654_560_000);
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

#[test]
fn date_today() {
    let today = Date::today(0).unwrap();
    let chrono_now = ChronoUtc::now();
    assert_eq!(
        today,
        Date {
            year: chrono_now.year() as u16,
            month: chrono_now.month() as u8,
            day: chrono_now.day() as u8,
        }
    );
}

#[test]
fn date_today_offset() {
    for offset in (-86399..86399).step_by(1000) {
        let today = Date::today(offset).unwrap();
        let chrono_now_utc = ChronoUtc::now();
        let chrono_tz = ChronoFixedOffset::east(offset);
        let chrono_now = chrono_now_utc.with_timezone(&chrono_tz);
        assert_eq!(
            today,
            Date {
                year: chrono_now.year() as u16,
                month: chrono_now.month() as u8,
                day: chrono_now.day() as u8,
            }
        );
    }
}
