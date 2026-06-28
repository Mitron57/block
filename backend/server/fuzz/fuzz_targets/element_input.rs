#![no_main]

use board_server::fuzz_helpers::fuzz_element_body;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        fuzz_element_body(s);
    }
});
