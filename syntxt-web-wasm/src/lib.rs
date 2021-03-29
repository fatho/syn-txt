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

use syntxt_lang::line_map::Pos;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub mod components;
pub mod console;

use components::{
    editor::{self, Editor},
    list::ListItem,
    WeakComponentLink,
};
use components::{
    editor::{MarkerSeverity, ModelMarker},
    list::List,
};

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    App::<AppModel>::new().mount_to_body();
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
                    Err((_partial_ast, errors)) => {
                        for err in errors {
                            self.issues.push(Issue {
                                message: err.message,
                                start: err.pos.start,
                                end: err.pos.end,
                            })
                        }
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
            Msg::GoToIssue(index) => {
                if let Some(issue) = self.issues.get(index) {
                    self.editor.send_message(editor::Msg::GoTo {
                        line: issue.start.line as u32,
                        column: issue.start.column as u32,
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
                        markers=self.issues.iter().map(|issue| {
                            ModelMarker {
                                start_line_number: issue.start.line as u32,
                                start_column: issue.start.column as u32,
                                end_line_number: issue.end.line as u32,
                                end_column: issue.end.column as u32,
                                message: issue.message.clone(),
                                severity: MarkerSeverity::Error,
                            }
                        }).collect::<Vec<_>>()
                        on_content_changed=self.link.callback(|code| Msg::SourceCodeChanged(code))
                        />
                </div>
                <div class=classes!("tab", issue_tab_class) style="flex: 0 0 20%; min-height: 0; overflow: auto">
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
    start: Pos,
    end: Pos,
}

impl ListItem for Issue {
    fn view(&self) -> Html {
        html! {
            <>
                <span style="color:red; font-weight: bold; margin-right: 5px">{"ⓧ"}</span>
                <span>{&self.message}</span>
                <span style="color:gray; margin-left: 5px">{self.start.line}{":"}{self.start.column}</span>
            </>
        }
    }
}
