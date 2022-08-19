use speedate::{ParseError, Time};

#[path = "./utils.rs"]
mod utils;
use utils::param_tests;

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

#[test]
fn time_comparison() {
    let t1 = Time::parse_str("12:13:14").unwrap();
    let t2 = Time::parse_str("12:10:20").unwrap();

    assert!(t1 > t2);
    assert!(t1 >= t2);
    assert!(t1 >= t1.clone());
    assert!(t2 < t1);
    assert!(t2 <= t1);
    assert!(t2 <= t2.clone());
    assert!(t1.eq(&t1.clone()));
    assert!(!t1.eq(&t2.clone()));

    let t3 = Time::parse_str("12:13:14.123").unwrap();
    let t4 = Time::parse_str("12:13:13.999").unwrap();
    assert!(t3 > t4);
}

#[test]

fn time_total_seconds() {
    let t = Time::parse_str("01:02:03.04").unwrap();
    assert_eq!(t.total_seconds(), 1 * 3600 + 2 * 60 + 3);

    let t = Time::parse_str("12:13:14.999999").unwrap();
    assert_eq!(t.total_seconds(), 12 * 3600 + 13 * 60 + 14);
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
