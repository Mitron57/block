#![no_main]

use board_server::fuzz_helpers::{fuzz_login_body, fuzz_register_body};
use libfuzzer_sys::fuzz_target;

/// Fuzz target: десериализация и базовая валидация входных данных auth-эндпоинтов.
/// Произвольный JSON-подобный ввод не должен вызывать panic в парсере или
/// в синхронной логике валидации (длина пароля, формат email).
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        fuzz_register_body(s);
        fuzz_login_body(s);
    }
});
