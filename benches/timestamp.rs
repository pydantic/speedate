#![feature(test)]

extern crate test;

use speedate::DateTime;
use test::{black_box, Bencher};

#[bench]
fn parse_timestamp_str(bench: &mut Bencher) {
    let timestamps = [
        "1654646400",
        "-1654646400",
        "1654646404",
        "-1654646404",
        "1654646404.5",
        "1654646404.123456",
        "1654646404000.5",
        "1654646404123.456",
        "-1654646404.123456",
        "-1654646404000.123",
    ];

    bench.iter(|| {
        for &timestamp in &timestamps {
            black_box(DateTime::parse_str(timestamp).unwrap());
        }
    });
}

#[bench]
fn parse_timestamp_str_1654646400(bench: &mut Bencher) {
    bench.iter(|| black_box(DateTime::parse_str("1654646400").unwrap()));
}

#[bench]
fn parse_timestamp_str_neg_1654646400(bench: &mut Bencher) {
    bench.iter(|| black_box(DateTime::parse_str("-1654646400").unwrap()));
}

#[bench]
fn parse_timestamp_str_1654646404(bench: &mut Bencher) {
    bench.iter(|| black_box(DateTime::parse_str("1654646404").unwrap()));
}

#[bench]
fn parse_timestamp_str_neg_1654646404(bench: &mut Bencher) {
    bench.iter(|| black_box(DateTime::parse_str("-1654646404").unwrap()));
}

#[bench]
fn parse_timestamp_str_1654646404_5(bench: &mut Bencher) {
    bench.iter(|| black_box(DateTime::parse_str("1654646404.5").unwrap()));
}

#[bench]
fn parse_timestamp_str_1654646404_123456(bench: &mut Bencher) {
    bench.iter(|| black_box(DateTime::parse_str("1654646404.123456").unwrap()));
}

#[bench]
fn parse_timestamp_str_1654646404000_5(bench: &mut Bencher) {
    bench.iter(|| black_box(DateTime::parse_str("1654646404000.5").unwrap()));
}

#[bench]
fn parse_timestamp_str_1654646404123_456(bench: &mut Bencher) {
    bench.iter(|| black_box(DateTime::parse_str("1654646404123.456").unwrap()));
}

#[bench]
fn parse_timestamp_str_neg_1654646404_123456(bench: &mut Bencher) {
    bench.iter(|| black_box(DateTime::parse_str("-1654646404.123456").unwrap()));
}

#[bench]
fn parse_timestamp_str_neg_1654646404000_123(bench: &mut Bencher) {
    bench.iter(|| black_box(DateTime::parse_str("-1654646404000.123").unwrap()));
}