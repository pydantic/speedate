#![no_main]
use libfuzzer_sys::fuzz_target;
use speedate::int_parse_str;

fn check_int_matches(data: &str) {
    match int_parse_str(data) {
        Some(i) => assert_eq!(data.parse::<i64>().unwrap(), i),
        None => assert!(data.parse::<i64>().is_err())
    }
}

fuzz_target!(|data: String| {
    check_int_matches(&data);
});
