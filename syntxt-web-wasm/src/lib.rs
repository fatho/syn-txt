mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    // Use `js_namespace` here to bind `console.log(..)` instead of just `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn run() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, syntxt-web-wasm!");
}

#[wasm_bindgen]
pub fn parse(code: &str) -> String {
    match syntxt_lang::parser::Parser::parse(code) {
        Err(err) => format!("Failed: {}", err.message),
        Ok(_) => format!("OK"),
    }
}
