#![feature(test)]

extern crate test;

use not8601::{Date, DateTime, Time};
use test::{black_box, Bencher};

#[bench]
fn compare_dt_ok_not8601(bench: &mut Bencher) {
    let s = black_box("1997-09-09T09:09:09Z");
    bench.iter(|| {
        black_box(DateTime::parse_str(&s).unwrap());
    })
}

#[bench]
fn compare_dt_ok_iso8601(bench: &mut Bencher) {
    let s = black_box("1997-09-09T09:09:09Z");
    bench.iter(|| {
        black_box(iso8601::datetime(&s).unwrap());
    })
}

#[bench]
fn compare_dt_ok_chrono(bench: &mut Bencher) {
    let s = black_box("1997-09-09T09:09:09Z");
    bench.iter(|| {
        black_box(chrono::DateTime::parse_from_rfc3339(&s).unwrap());
    })
}

#[bench]
fn compare_dt_error_not8601(bench: &mut Bencher) {
    let s = black_box("1997-09-09T25:09:09Z");
    bench.iter(|| {
        let e = match DateTime::parse_str(&s) {
            Ok(_) => panic!("unexpectedly valid"),
            Err(e) => e,
        };
        black_box(e);
    })
}

#[bench]
fn compare_dt_error_iso8601(bench: &mut Bencher) {
    let s = black_box("1997-09-09T25:09:09Z");
    bench.iter(|| {
        let e = match iso8601::datetime(&s) {
            Ok(_) => panic!("unexpectedly valid"),
            Err(e) => e,
        };
        black_box(e);
    })
}

#[bench]
fn compare_dt_error_chrono(bench: &mut Bencher) {
    let s = black_box("1997-09-09T25:09:09Z");
    bench.iter(|| {
        let e = match chrono::DateTime::parse_from_rfc3339(&s) {
            Ok(_) => panic!("unexpectedly valid"),
            Err(e) => e,
        };
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
