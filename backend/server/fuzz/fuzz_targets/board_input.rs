#![no_main]

use board_server::fuzz_helpers::fuzz_board_bodies;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        fuzz_board_bodies(s);
    }
});
