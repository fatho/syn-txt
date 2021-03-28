use syntxt_lang::line_map::Pos;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub mod components;
pub mod console;

use components::list::List;
use components::{
    editor::{self, Editor},
    list::ListItem,
    WeakComponentLink,
};

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    App::<AppModel>::new().mount_to_body();
    console_log!("syntxt initialized");
}

#[wasm_bindgen]
pub fn parse(code: &str) -> Box<[JsValue]> {
    match syntxt_lang::parser::Parser::parse(code) {
        Err(err) => Box::new([
            JsValue::from_f64(err.pos.start.line as f64),
            JsValue::from_f64(err.pos.start.column as f64),
            JsValue::from_f64(err.pos.end.line as f64),
            JsValue::from_f64(err.pos.end.column as f64),
            JsValue::from_str(&err.message),
        ]),
        Ok(_) => Box::new([]),
    }
}

struct AppModel {
    link: ComponentLink<Self>,
    editor: WeakComponentLink<Editor>,
    showing_issues: bool,
    issues: Vec<Issue>,
}

enum Msg {
    SourceCodeChanged(String),
    ShowIssues(bool),
    GoToIssue(usize),
}

impl Component for AppModel {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            editor: WeakComponentLink::default(),
            showing_issues: true,
            issues: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SourceCodeChanged(code) => {
                self.issues.clear();
                match syntxt_lang::parser::Parser::parse(&code) {
                    Ok(_) => {}
                    Err(err) => self.issues.push(Issue {
                        message: err.message,
                        location: err.pos.start,
                    }),
                }
                true
            }
            Msg::ShowIssues(show) => {
                if show != self.showing_issues {
                    self.showing_issues = show;
                    true
                } else {
                    false
                }
            }
            Msg::GoToIssue(index) => {
                if let Some(issue) = self.issues.get(index) {
                    self.editor.send_message(editor::Msg::GoTo {
                        line: issue.location.line as u32,
                        column: issue.location.column as u32,
                    });
                }
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let showing_issues = self.showing_issues;
        let issue_tab_class = if self.showing_issues {
            "tab-active"
        } else {
            "tab-inactive"
        };
        html! {
            <section style="height: 100vh; display: flex; flex-direction: column">
                <header style="flex: 0 0 0px; background-color: black">
                </header>
                <div style="flex: 1 1 0px; min-height: 0; min-width: 0;">
                    <Editor
                        weak_link=&self.editor
                        on_content_changed=self.link.callback(|code| Msg::SourceCodeChanged(code))
                        />
                </div>
                <div class=classes!("tab", issue_tab_class) style="flex: 0 0 20%; min-height: 0; overflow: auto">
                    // TODO: make this more efficient by not cloning the issues
                    <List<Issue>
                        items=self.issues.clone()
                        empty_text="No issues detected"
                        onclick=self.link.callback(|index| Msg::GoToIssue(index))
                        />
                </div>
                <footer class=classes!("footer") style="flex: 0 0 24px;">
                    <button
                        class=classes!("button-flat")
                        style="height: 100%;"
                        onclick=self.link.callback(move |_| Msg::ShowIssues(!showing_issues))
                        >{ format!("ⓧ {}", self.issues.len()) }</button>
                </footer>
            </section>
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
struct Issue {
    message: String,
    location: Pos,
}

impl ListItem for Issue {
    fn view(&self) -> Html {
        html! {
            <>
                <span style="color:red; font-weight: bold; margin-right: 5px">{"ⓧ"}</span>
                <span>{&self.message}</span>
                <span style="color:gray; margin-left: 5px">{self.location.line}{":"}{self.location.column}</span>
            </>
        }
    }
}
