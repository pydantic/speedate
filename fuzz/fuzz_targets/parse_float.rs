#![no_main]
use libfuzzer_sys::fuzz_target;
use speedate::{float_parse_str, IntFloat};

fn check_float_matches(data: &str) {
    match float_parse_str(data) {
        IntFloat::Int(i) => assert_eq!(data.parse::<i64>().unwrap(), i),
        IntFloat::Float(f) => assert_eq!(data.parse::<f64>().unwrap(), f),
        IntFloat::Err => assert!(data.parse::<f64>().is_err()),
    }
}

fuzz_target!(|data: String| {
    check_float_matches(&data);
});
