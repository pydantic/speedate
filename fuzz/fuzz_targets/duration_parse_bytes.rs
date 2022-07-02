#![no_main]
use libfuzzer_sys::fuzz_target;
use speedate::Duration;

fuzz_target!(|data: &[u8]| {
    match Duration::parse_bytes(data) {
        Ok(_) => (),
        Err(_) => (),
    }
});
