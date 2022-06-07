#![no_main]
use chrono::{Datelike, NaiveDateTime, Timelike};
use libfuzzer_sys::fuzz_target;
use speedate::{Date, DateTime, Time};

fn check_timestamp(timestamp: i64, microseconds: u32) {
    if let Some(abs_ts) = timestamp.checked_abs() {
        if abs_ts < 20_000_000_000 {
            if let Some(mut chrono_dt) = NaiveDateTime::from_timestamp_opt(timestamp, microseconds * 1_000) {
                let year = chrono_dt.year();
                if year >= 1000 && year <= 2500 {
                    let dt = match DateTime::from_timestamp(timestamp, microseconds) {
                        Ok(dt) => dt,
                        Err(e) => panic!(
                            "got error {:?} for ({}, {}) ({})",
                            e, timestamp, microseconds, chrono_dt
                        ),
                    };
                    let mut microsecond = chrono_dt.nanosecond() as u32 / 1_000;
                    if microsecond >= 1_000_000 {
                        chrono_dt += chrono::Duration::seconds(1);
                        microsecond -= 1_000_000;
                    }
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
                                microsecond,
                            },
                            offset: None,
                        },
                        "timestamp: ({}, {}) => speedate({}) != chrono({})",
                        timestamp,
                        microseconds,
                        dt,
                        chrono_dt
                    );
                    return;
                }
            }
        }
    }
    // otherwise just check it doesn't panic
    match DateTime::from_timestamp(timestamp, microseconds) {
        Ok(_) => (),
        Err(_) => (),
    }
}

fuzz_target!(|args: (i64, u32)| { check_timestamp(args.0, args.1 / 1_000) });
