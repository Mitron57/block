#![no_main]

use board_server::infrastructure::JwtConfig;
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

// Инициализируем один раз на весь процесс фаззера, а не на каждую итерацию.
static JWT: OnceLock<JwtConfig> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    let jwt = JWT.get_or_init(|| JwtConfig::new("fuzz-secret-key-at-least-32-bytes!!", 1));
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = jwt.verify(s);
    }
});
