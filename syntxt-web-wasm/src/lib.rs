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
    console_error_panic_hook::set_once();
    console_log!("syntxt initialized");
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, syntxt-web-wasm!");
}

#[wasm_bindgen]
pub fn parse(code: &str) -> Box<[JsValue]> {
    match syntxt_lang::parser::Parser::parse(code) {
        Err(err) => {
            Box::new([
                JsValue::from_f64(err.pos.start.line as f64),
                JsValue::from_f64(err.pos.start.column as f64),
                JsValue::from_f64(err.pos.end.line as f64),
                JsValue::from_f64(err.pos.end.column as f64),
                JsValue::from_str(&err.message),
            ])
        },
        Ok(_) => Box::new([]),
    }
}
