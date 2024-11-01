#![no_main]
use chrono::{Datelike, NaiveDateTime, Timelike};
use libfuzzer_sys::fuzz_target;
use speedate::{Date, DateTime, Time};

fn check_timestamp(timestamp: i64, microseconds: u32) {
    if let Some(ts_abs) = timestamp.checked_abs() {
        // adjust seconds and nanoseconds for chrono to match logic of speedate
        let (chrono_seconds, chrono_nano) = if ts_abs > 20_000_000_000 {
            let mut s = timestamp / 1_000;
            let mut total_nano = microseconds as i64 * 1_000 + (timestamp % 1_000) * 1_000_000;
            if total_nano > 1_000_000_000 {
                s += total_nano / 1_000_000_000;
                total_nano %= 1_000_000_000;
            }
            (s, total_nano as u32)
        } else {
            (timestamp, microseconds * 1_000)
        };

        if let Some(mut chrono_dt) = NaiveDateTime::from_timestamp_opt(chrono_seconds, chrono_nano) {
            let year = chrono_dt.year();
            if year >= 0 && year <= 9999 {
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
                            tz_offset: None,
                        },
                    },
                    "timestamp: ({}, {}) => speedate({}) != chrono({})",
                    timestamp,
                    microseconds,
                    dt,
                    chrono_dt
                );
                assert_eq!(dt.timestamp(), chrono_dt.timestamp(), "{}, timestamp comparison", dt);
                return;
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
