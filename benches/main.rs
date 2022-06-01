#![feature(test)]

extern crate test;

use speedate::{Date, DateTime, Duration, Time};
use test::{black_box, Bencher};

#[bench]
fn compare_datetime_ok_speedate(bench: &mut Bencher) {
    let s = black_box("2000-01-01T00:02:03Z");
    bench.iter(|| {
        let dt = DateTime::parse_str(&s).unwrap();
        black_box((
            dt.date.year,
            dt.date.month,
            dt.date.day,
            dt.time.hour,
            dt.time.minute,
            dt.time.second,
            dt.time.microsecond,
        ));
    })
}

#[bench]
fn compare_datetime_ok_iso8601(bench: &mut Bencher) {
    let s = black_box("2000-01-01T00:02:03Z");
    bench.iter(|| {
        // No way to actually get the numeric values from iso8601!
        black_box(iso8601::datetime(&s).unwrap());
    })
}

#[bench]
fn compare_datetime_ok_chrono(bench: &mut Bencher) {
    use chrono::{Datelike, Timelike};
    let s = black_box("2000-01-01T00:02:03Z");
    bench.iter(|| {
        let dt = chrono::DateTime::parse_from_rfc3339(&s).unwrap();
        black_box((
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second(),
            dt.nanosecond(),
        ));
    })
}

#[bench]
fn compare_datetime_ok_python_direct(bench: &mut Bencher) {
    let gil = pyo3::prelude::Python::acquire_gil();
    let py = gil.python();
    let datetime_module = py.import("datetime").unwrap();
    let datetime: &pyo3::PyAny = datetime_module.getattr("datetime").unwrap();
    let fromisoformat = datetime.getattr("fromisoformat").unwrap();
    let s = black_box("2000-01-01T00:02:03");
    let d = fromisoformat.call1((s,)).unwrap();
    println!("{:?}", d);
    bench.iter(|| {
        fromisoformat.call1((s,)).unwrap();
    });
}

#[bench]
fn compare_datetime_ok_python_speedate(bench: &mut Bencher) {
    let gil = pyo3::prelude::Python::acquire_gil();
    let py = gil.python();
    let s = black_box("2000-01-01T00:02:03");
    let dt = DateTime::parse_str(&s).unwrap();
    bench.iter(|| {
        let dt = DateTime::parse_str(&s).unwrap();
        pyo3::types::PyDateTime::new(
            py,
            dt.date.year as i32,
            dt.date.month,
            dt.date.day,
            dt.time.hour,
            dt.time.minute,
            dt.time.second,
            dt.time.microsecond,
            None,
        )
        .unwrap();
    });
}

#[bench]
fn compare_duration_ok_speedate(bench: &mut Bencher) {
    let s = black_box("P1Y2M3DT4H5M6S");
    bench.iter(|| {
        black_box(Duration::parse_str(&s).unwrap());
    })
}

#[bench]
fn compare_duration_ok_iso8601(bench: &mut Bencher) {
    let s = black_box("P1Y2M3DT4H5M6S");
    bench.iter(|| {
        black_box(iso8601::duration(&s).unwrap());
    })
}

macro_rules! expect_error {
    ($expr:expr) => {
        match $expr {
            Ok(t) => panic!("unexpectedly valid: {:?}", t),
            Err(e) => e,
        }
    };
}

#[bench]
fn compare_datetime_error_speedate(bench: &mut Bencher) {
    let s = black_box("2000-01-01T25:02:03Z");
    bench.iter(|| {
        let e = expect_error!(DateTime::parse_str(&s));
        black_box(e);
    })
}

#[bench]
fn compare_datetime_error_iso8601(bench: &mut Bencher) {
    let s = black_box("2000-01-01T25:02:03Z");
    bench.iter(|| {
        let e = expect_error!(iso8601::datetime(&s));
        black_box(e);
    })
}

#[bench]
fn compare_datetime_error_chrono(bench: &mut Bencher) {
    let s = black_box("2000-01-01T25:02:03Z");
    bench.iter(|| {
        let e = expect_error!(chrono::DateTime::parse_from_rfc3339(&s));
        black_box(e);
    })
}

#[bench]
fn compare_timestamp_ok_speedate(bench: &mut Bencher) {
    let ts = black_box(1654617803);
    bench.iter(|| {
        let dt = DateTime::from_timestamp(ts, 0).unwrap();
        black_box((
            dt.date.year,
            dt.date.month,
            dt.date.day,
            dt.time.hour,
            dt.time.minute,
            dt.time.second,
            dt.time.microsecond,
        ));
    })
}

#[bench]
fn compare_timestamp_ok_chrono(bench: &mut Bencher) {
    use chrono::{Datelike, Timelike};
    let ts = black_box(1654617803);
    bench.iter(|| {
        let dt = chrono::NaiveDateTime::from_timestamp_opt(ts, 0).unwrap();
        black_box((
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second(),
            dt.nanosecond(),
        ));
    })
}

#[bench]
fn dt_custom_tz(bench: &mut Bencher) {
    let s = black_box("1997-09-09T09:09:09-09:09");
    bench.iter(|| {
        black_box(DateTime::parse_str(&s).unwrap());
    })
}

#[bench]
fn dt_naive(bench: &mut Bencher) {
    let s = black_box("1997-09-09T09:09:09");
    bench.iter(|| {
        black_box(DateTime::parse_str(&s).unwrap());
    })
}

#[bench]
fn date(bench: &mut Bencher) {
    let s = black_box("1997-09-09");
    bench.iter(|| {
        black_box(Date::parse_str(&s).unwrap());
    })
}

#[bench]
fn time(bench: &mut Bencher) {
    let s = black_box("09:09:09.09");
    bench.iter(|| {
        black_box(Time::parse_str(&s).unwrap());
    })
}

#[bench]
fn x_combined(bench: &mut Bencher) {
    let dt1 = black_box("1997-09-09T09:09:09Z");
    let dt2 = black_box("1997-09-09 09:09:09");
    let dt3 = black_box("2000-02-29 01:01:50.123456");
    let dt4 = black_box("2000-02-29 01:01:50.123456+08:00");

    let d1 = black_box("1997-09-09");
    let d2 = black_box("2000-02-29");
    let d3 = black_box("2001-02-28");
    let d4 = black_box("2001-12-28");

    let t1 = black_box("12:13");
    let t2 = black_box("12:13:14");
    let t3 = black_box("12:13:14.123");
    let t4 = black_box("12:13:14.123456");
    bench.iter(|| {
        black_box(DateTime::parse_str(&dt1).unwrap());
        black_box(DateTime::parse_str(&dt2).unwrap());
        black_box(DateTime::parse_str(&dt3).unwrap());
        black_box(DateTime::parse_str(&dt4).unwrap());

        black_box(Date::parse_str(&d1).unwrap());
        black_box(Date::parse_str(&d2).unwrap());
        black_box(Date::parse_str(&d3).unwrap());
        black_box(Date::parse_str(&d4).unwrap());

        black_box(Time::parse_str(&t1).unwrap());
        black_box(Time::parse_str(&t2).unwrap());
        black_box(Time::parse_str(&t3).unwrap());
        black_box(Time::parse_str(&t4).unwrap());
    })
}
