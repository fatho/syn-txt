use wasm_bindgen::prelude::*;
use yew::{prelude::*, web_sys::HtmlElement};
use yew::Properties;


pub struct Editor {
    #[allow(unused)]
    link: ComponentLink<Self>,
    container_ref: NodeRef,
    on_content_changed: Callback<String>,
    editor: Option<JsValue>,
    change_handler: Closure<dyn Fn(String)>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub on_content_changed: Callback<String>,
}

pub enum Msg {
    Test
}

impl Component for Editor {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let on_content_changed = props.on_content_changed.clone();
        Self {
            link,
            container_ref: NodeRef::default(),
            on_content_changed: props.on_content_changed,
            editor: None,
            change_handler: Closure::wrap(Box::new(move |value| {
                on_content_changed.emit(value);
            })),
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.on_content_changed != props.on_content_changed {
            let on_content_changed = props.on_content_changed.clone();
            self.on_content_changed = props.on_content_changed;

            // Update handler
            let new_change_handler = Closure::wrap(Box::new(move |value| {
                on_content_changed.emit(value);
            }) as Box<dyn Fn(String)>);

            if let Some(editor) = self.editor.clone() {
                onContentChanged(editor, &new_change_handler);
            }

            self.change_handler = new_change_handler;
        }
        false
    }

    fn view(&self) -> Html {
        html! {
            <div
                ref=self.container_ref.clone()
                style="height: 100%; width: 100%;"
                >
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let container = self.container_ref.cast::<HtmlElement>().unwrap();
            let editor = createEditor(&container);
            onContentChanged(editor.clone(), &self.change_handler);
            self.editor = Some(editor);
        }
    }

    fn destroy(&mut self) {
        if let Some(editor) = self.editor.take() {
            destroyEditor(editor);
        }
    }
}

#[wasm_bindgen]
extern "C" {
    // #[wasm_bindgen(js_namespace = monaco.editor, js_name = create)]
    // pub fn createEditor(container: &HtmlElement, props: JsValue);

    #[wasm_bindgen(js_namespace = syntxt_helpers, js_name = createEditor)]
    fn createEditor(container: &HtmlElement) -> JsValue;

    #[wasm_bindgen(js_namespace = syntxt_helpers, js_name = onContentChanged)]
    fn onContentChanged(editor: JsValue, callback: &Closure<dyn Fn(String)>);

    #[wasm_bindgen(js_namespace = syntxt_helpers, js_name = destroyEditor)]
    fn destroyEditor(editor: JsValue);
}
