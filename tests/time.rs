use speedate::{ParseError, Time};

mod common;
use common::param_tests;

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
