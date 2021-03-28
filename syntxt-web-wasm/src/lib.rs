use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub mod components;
pub mod console;

use components::editor::Editor;

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    App::<AppModel>::new().mount_to_body();
    console_log!("syntxt initialized");
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

struct AppModel {
    link: ComponentLink<Self>,
}

enum Msg {
    SourceCodeChanged(String),
}

impl Component for AppModel {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SourceCodeChanged(code) => {
                console_log!("Received {} characters", code.len());
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <section style="height: 100vh; display: flex; flex-direction: column">
                <header style="flex: 0 0 48px; background-color: black">
                </header>
                <div style="flex: 1 1 0px; min-height: 0; min-width: 0;">
                    <Editor on_content_changed=self.link.callback(|code| Msg::SourceCodeChanged(code))/>
                </div>
                <footer style="flex: 0 0 48px; background-color: black">
                </footer>
            </section>
        }
    }
}
