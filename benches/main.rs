#![feature(test)]

extern crate test;

use not8601::{Date, DateTime, Time};
use test::{black_box, Bencher};

#[bench]
fn compare_dt_ok_not8601(bench: &mut Bencher) {
    let s1 = black_box("2000-01-01T00:02:03Z");
    let s2 = black_box("2000-01-02T00:02:03Z");
    let s3 = black_box("2000-01-02T00:02:03Z");
    let s4 = black_box("2000-01-03T00:02:03Z");
    bench.iter(|| {
        black_box(DateTime::parse_str(&s1).unwrap());
        black_box(DateTime::parse_str(&s2).unwrap());
        black_box(DateTime::parse_str(&s3).unwrap());
        black_box(DateTime::parse_str(&s4).unwrap());
    })
}

#[bench]
fn compare_dt_ok_iso8601(bench: &mut Bencher) {
    let s1 = black_box("2000-01-01T00:02:03Z");
    let s2 = black_box("2000-01-02T00:02:03Z");
    let s3 = black_box("2000-01-02T00:02:03Z");
    let s4 = black_box("2000-01-03T00:02:03Z");
    bench.iter(|| {
        black_box(iso8601::datetime(&s1).unwrap());
        black_box(iso8601::datetime(&s2).unwrap());
        black_box(iso8601::datetime(&s3).unwrap());
        black_box(iso8601::datetime(&s4).unwrap());
    })
}

#[bench]
fn compare_dt_ok_chrono(bench: &mut Bencher) {
    let s1 = black_box("2000-01-01T00:02:03Z");
    let s2 = black_box("2000-01-02T00:02:03Z");
    let s3 = black_box("2000-01-02T00:02:03Z");
    let s4 = black_box("2000-01-03T00:02:03Z");
    bench.iter(|| {
        black_box(chrono::DateTime::parse_from_rfc3339(&s1).unwrap());
        black_box(chrono::DateTime::parse_from_rfc3339(&s2).unwrap());
        black_box(chrono::DateTime::parse_from_rfc3339(&s3).unwrap());
        black_box(chrono::DateTime::parse_from_rfc3339(&s4).unwrap());
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
fn compare_dt_error_not8601(bench: &mut Bencher) {
    let s1 = black_box("2000-01-01T25:02:03Z");
    bench.iter(|| {
        let e = expect_error!(DateTime::parse_str(&s1));
        black_box(e);
        let e = expect_error!(DateTime::parse_str(&s1));
        black_box(e);
        let e = expect_error!(DateTime::parse_str(&s1));
        black_box(e);
        let e = expect_error!(DateTime::parse_str(&s1));
        black_box(e);
    })
}

#[bench]
fn compare_dt_error_iso8601(bench: &mut Bencher) {
    let s = black_box("2000-01-01T25:02:03Z");
    bench.iter(|| {
        let e = expect_error!(iso8601::datetime(&s));
        black_box(e);
        let e = expect_error!(iso8601::datetime(&s));
        black_box(e);
        let e = expect_error!(iso8601::datetime(&s));
        black_box(e);
        let e = expect_error!(iso8601::datetime(&s));
        black_box(e);
    })
}

#[bench]
fn compare_dt_error_chrono(bench: &mut Bencher) {
    let s = black_box("2000-01-01T25:02:03Z");
    bench.iter(|| {
        let e = expect_error!(chrono::DateTime::parse_from_rfc3339(&s));
        black_box(e);
        let e = expect_error!(chrono::DateTime::parse_from_rfc3339(&s));
        black_box(e);
        let e = expect_error!(chrono::DateTime::parse_from_rfc3339(&s));
        black_box(e);
        let e = expect_error!(chrono::DateTime::parse_from_rfc3339(&s));
        black_box(e);
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
