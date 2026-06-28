#![no_main]

use board_server::infrastructure::JwtConfig;
use libfuzzer_sys::fuzz_target;

/// Fuzz target: проверяем, что verify никогда не паникует на произвольном вводе.
/// Корректный JWT, неправильная подпись, мусор — всё должно возвращать Err.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let jwt = JwtConfig::new("fuzz-secret-key-at-least-32-bytes!!", 1);
        let _ = jwt.verify(s);
    }
});
