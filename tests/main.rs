use not8601::{Date, DateTime, ParseError, Time};

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

macro_rules! expect_error {
    ($expr:expr, $error:ident) => {
        match $expr {
            Ok(_) => panic!("unexpectedly valid"),
            Err(e) => assert_eq!(e, ParseError::$error),
        }
    };
}

#[test]
fn date_error() {
    expect_error!(Date::parse_str("xxx"), InvalidCharYear);
    expect_error!(Date::parse_str("2020x"), InvalidCharDateSep);
    expect_error!(Date::parse_str("2020-12x"), InvalidCharDateSep);
    expect_error!(Date::parse_str("2020-13-01"), OutOfRangeMonth);
    expect_error!(Date::parse_str("2020-04-31"), OutOfRangeDay);
}

#[test]
fn date_leap() {
    // normal not leap year
    assert_eq!(Date::parse_str("2003-02-28").unwrap().to_string(), "2003-02-28");
    expect_error!(Date::parse_str("2003-02-29"), OutOfRangeDay);

    // normal leap year
    assert_eq!(Date::parse_str("2004-02-29").unwrap().to_string(), "2004-02-29");

    // special 100 not a leap year
    expect_error!(Date::parse_str("1900-02-29"), OutOfRangeDay);

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
fn time_error() {
    expect_error!(Time::parse_str("xxx"), InvalidCharHour);
    expect_error!(Time::parse_str("12x"), InvalidCharTimeSep);
    expect_error!(Time::parse_str("12:x"), InvalidCharMinute);
    expect_error!(Time::parse_str("12:13x"), InvalidCharTimeSep);
    expect_error!(Time::parse_str("12:13:x"), InvalidCharSecond);
    expect_error!(Time::parse_str("12:13:12."), SecondFractionMissing);
    expect_error!(Time::parse_str("12:13:12.1234567"), SecondFractionTooLong);
    expect_error!(Time::parse_str("24:00:00"), OutOfRangeHour);
    expect_error!(Time::parse_str("23:60:00"), OutOfRangeMinute);
    expect_error!(Time::parse_str("23:59:60"), OutOfRangeSecond);
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
fn datetime_tz_negative() {
    let dt = DateTime::parse_str("2020-01-01T12:13:14-02:15").unwrap();
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
fn datetime_seconds_break() {
    let dt = DateTime::parse_str("2020-01-01 12:13:14.123z").unwrap();
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14.123Z");
}

#[test]
fn datetime_error() {
    expect_error!(DateTime::parse_str("xxx"), InvalidCharYear);
    expect_error!(DateTime::parse_str("2020-01-01x"), InvalidCharDateTimeSep);
    expect_error!(DateTime::parse_str("2020-01-01Tx"), InvalidCharHour);
    expect_error!(DateTime::parse_str("2020-01-01T12:00:00x"), InvalidCharTzSign);
    expect_error!(DateTime::parse_str("2020-01-01T12:00:00+x"), InvalidCharTzHour);
    expect_error!(DateTime::parse_str("2020-01-01T12:00:00+00x"), InvalidCharTzMinute);
    expect_error!(DateTime::parse_str("2020-01-01T12:00:00Z "), ExtraCharacters);
}
