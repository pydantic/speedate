#![no_main]
use chrono::{DateTime as ChronoDateTime, Datelike, Timelike};
use libfuzzer_sys::fuzz_target;
use speedate::{Date, DateTime, Time};

fuzz_target!(|data: &[u8]| {
    if let Ok(dt) = DateTime::parse_bytes(data) {
        if let Ok(s) = String::from_utf8(data.to_vec()) {
            if let Ok(chrono_dt) = ChronoDateTime::parse_from_rfc3339(&s) {
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
                            tz_offset: Some((chrono_dt.offset().local_minus_utc() / 60) as i16),
                        },
                    },
                    "timestamp: {:?} => speedate({}) != chrono({})",
                    s,
                    dt,
                    chrono_dt
                );
            }
        }
    }
});
