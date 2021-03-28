use syntxt_lang::line_map::Pos;
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
    showing_issues: bool,
    issues: Vec<Issue>,
}

enum Msg {
    SourceCodeChanged(String),
    ShowIssues(bool),
}

impl Component for AppModel {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
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
                    Err(err) => {
                        self.issues.push(Issue {
                            message: err.message,
                            location: err.pos.start,
                        })
                    }
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
                    <Editor on_content_changed=self.link.callback(|code| Msg::SourceCodeChanged(code))/>
                </div>
                <div class=classes!("tab", issue_tab_class) style="flex: 0 0 20%; min-height: 0; overflow: auto">
                    <div class=classes!("list-flat") style="overflow-x: hidden; overflow-y: auto">
                    {
                        if self.issues.is_empty() {
                            html! {
                                <div class=classes!("list-item-flat") disabled=true>
                                    <span style="color:red; font-weight: bold; visibility: hidden; margin-right: 5px">{"ⓧ"}</span>
                                    <span>{"No issues detected"}</span>
                                </div>
                            }
                        } else {
                            self.issues.iter()
                                .enumerate()
                                .map(|(index, issue)| render_issue(index, issue))
                                .collect::<Html>()
                        }
                    }
                    </div>
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

struct Issue {
    message: String,
    location: Pos,
}

fn render_issue(index: usize, issue: &Issue) -> Html {
    html! {
        <div class=classes!("list-item-flat") tabindex={index}>
            <span style="color:red; font-weight: bold; margin-right: 5px">{"ⓧ"}</span>
            <span>{&issue.message}</span>
            <span style="color:gray; margin-left: 5px">{issue.location.line}{":"}{issue.location.column}</span>
        </div>
    }
}
