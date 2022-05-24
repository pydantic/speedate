#![feature(test)]

extern crate test;

use not8601::{Date, DateTime, Time};
use test::{black_box, Bencher};

#[bench]
fn dt_z(bench: &mut Bencher) {
    let s = black_box("1997-09-09T09:09:09Z");
    bench.iter(|| {
        black_box(DateTime::parse(&s).unwrap());
    })
}

#[bench]
fn dt_z_iso8601(bench: &mut Bencher) {
    let s = black_box("1997-09-09T09:09:09Z");
    bench.iter(|| {
        black_box(iso8601::datetime(&s).unwrap());
    })
}

#[bench]
fn dt_z_chrono(bench: &mut Bencher) {
    let s = black_box("1997-09-09T09:09:09Z");
    bench.iter(|| {
        black_box(chrono::DateTime::parse_from_rfc3339(&s).unwrap());
    })
}

#[bench]
fn dt_custom_tz(bench: &mut Bencher) {
    let s = black_box("1997-09-09T09:09:09-09:09");
    bench.iter(|| {
        black_box(DateTime::parse(&s).unwrap());
    })
}

#[bench]
fn dt_naive(bench: &mut Bencher) {
    let s = black_box("1997-09-09T09:09:09");
    bench.iter(|| {
        black_box(DateTime::parse(&s).unwrap());
    })
}

#[bench]
fn date(bench: &mut Bencher) {
    let s = black_box("1997-09-09");
    bench.iter(|| {
        black_box(Date::parse(&s).unwrap());
    })
}

#[bench]
fn time(bench: &mut Bencher) {
    let s = black_box("09:09:09.09");
    bench.iter(|| {
        black_box(Time::parse(&s).unwrap());
    })
}
