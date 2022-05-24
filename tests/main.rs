use not8601::{Date, DateTime, Time};

#[test]
fn test_date() {
    assert_eq!(
        Date::parse("2020-01-01").unwrap(),
        Date {
            year: 2020,
            month: 1,
            day: 1,
        }
    );
}

#[test]
fn test_time() {
    assert_eq!(
        Time::parse("12:13:14.123456").unwrap(),
        Time {
            hour: 12,
            minute: 13,
            second: 14,
            microsecond: 123456,
        }
    );
}

#[test]
fn test_datetime_naive() {
    assert_eq!(
        DateTime::parse("2020-01-01T12:13:14.123456").unwrap(),
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
}

#[test]
fn test_datetime_tz_z() {
    let dt = DateTime::parse("2020-01-01 12:13:14z").unwrap();
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
fn test_datetime_tz_2hours() {
    let dt = DateTime::parse("2020-01-01T12:13:14+02:00").unwrap();
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
fn test_datetime_tz_no_colon() {
    let dt = DateTime::parse("2020-01-01T12:13:14+1234").unwrap();
    assert_eq!(dt.offset, Some(12 * 60 + 34));
    assert_eq!(dt.to_string(), "2020-01-01T12:13:14+12:34");
}
