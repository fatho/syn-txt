// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2021  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use serde::{Deserialize, Serialize};
use serde_repr::*;
use wasm_bindgen::prelude::*;
use yew::Properties;
use yew::{prelude::*, web_sys::HtmlElement};

use super::WeakComponentLink;

pub struct Editor {
    #[allow(unused)]
    link: ComponentLink<Self>,
    weak_link: WeakComponentLink<Editor>,

    container_ref: NodeRef,
    on_content_changed: Callback<String>,
    editor: Option<JsValue>,
    change_handler: Closure<dyn Fn(String)>,
    markers: Vec<ModelMarker>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub on_content_changed: Callback<String>,
    #[prop_or_default]
    pub markers: Vec<ModelMarker>,
    pub weak_link: WeakComponentLink<Editor>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelMarker {
    pub start_line_number: u32,
    pub start_column: u32,
    pub end_line_number: u32,
    pub end_column: u32,
    pub message: String,
    pub severity: MarkerSeverity,
}

#[derive(Serialize_repr, Deserialize_repr, Clone, PartialEq, Debug, Copy)]
#[repr(u32)]
pub enum MarkerSeverity {
    Error = 8,
    Hint = 1,
    Info = 2,
    Warning = 4,
}

pub enum Msg {
    Load { text: String },
    GoTo { line: u32, column: u32 },
}

impl Editor {
    fn register_on_content_changed(&mut self) {
        if let Some(editor) = self.editor.clone() {
            onContentChanged(editor, &self.change_handler);
        }
    }
}

impl Component for Editor {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        props.weak_link.attach(link.clone());
        let on_content_changed = props.on_content_changed.clone();
        Self {
            link,
            weak_link: props.weak_link,
            container_ref: NodeRef::default(),
            on_content_changed: props.on_content_changed,
            editor: None,
            change_handler: Closure::wrap(Box::new(move |value| {
                on_content_changed.emit(value);
            })),
            markers: props.markers,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GoTo { line, column } => {
                if let Some(editor) = self.editor.clone() {
                    jumpTo(editor, line, column);
                }
            }
            Msg::Load { text } => {
                if let Some(editor) = self.editor.clone() {
                    load(editor, text);
                }
                self.register_on_content_changed();
            }
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.on_content_changed != props.on_content_changed {
            self.on_content_changed = props.on_content_changed;

            let on_content_changed = self.on_content_changed.clone();
            let new_change_handler = Closure::wrap(Box::new(move |value| {
                on_content_changed.emit(value);
            }) as Box<dyn Fn(String)>);
            // Must keep the handler alive until we unregistered it
            let old_change_handler =
                std::mem::replace(&mut self.change_handler, new_change_handler);
            self.register_on_content_changed();
            drop(old_change_handler);
        }
        if self.markers != props.markers {
            self.markers = props.markers;
            if let Some(editor) = self.editor.clone() {
                let marker_value = JsValue::from_serde(&self.markers).unwrap();
                setModelMarkers(&editor, &marker_value);
            }
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
        self.weak_link.detach();
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

    #[wasm_bindgen(js_namespace = syntxt_helpers, js_name = jumpTo)]
    fn jumpTo(editor: JsValue, line: u32, column: u32);

    #[wasm_bindgen(js_namespace = syntxt_helpers, js_name = setModelMarkers)]
    fn setModelMarkers(editor: &JsValue, markers: &JsValue);

    #[wasm_bindgen(js_namespace = syntxt_helpers, js_name = load)]
    fn load(editor: JsValue, text: String);
}
