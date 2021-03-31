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

use std::{sync::Arc, vec};

use syntxt_lang::{ast, line_map::Pos};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub mod components;
pub mod console;

use components::{
    ast_view::AstView,
    editor::{MarkerSeverity, ModelMarker},
    list::List,
    song_view::SongView,
    splitter::{Orientation, SplitContainer, SplitPane},
    Size,
};
use components::{
    editor::{self, Editor},
    list::ListItem,
    WeakComponentLink,
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
    ast: ast::NodePtr<ast::Root>,
    issues: Vec<Issue>,
}

enum Msg {
    SourceCodeChanged(String),
    ShowIssues(bool),
    GoToIssue(usize),
    JumpToEditor { line: u32, column: u32 },
}

impl Component for AppModel {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            editor: WeakComponentLink::default(),
            showing_issues: false,
            ast: Arc::new(ast::Node {
                span: 0..0,
                pos: Pos::origin()..Pos::origin(),
                data: ast::Root { objects: vec![] },
            }),
            issues: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SourceCodeChanged(code) => {
                self.issues.clear();
                match syntxt_lang::parser::Parser::parse(&code) {
                    Ok(ast) => {
                        self.ast = Arc::new(ast);
                    }
                    Err((partial_ast, errors)) => {
                        self.ast = Arc::new(partial_ast);
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
            Msg::JumpToEditor { line, column } => {
                self.editor.send_message(editor::Msg::GoTo { line, column });
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let showing_issues = self.showing_issues;
        html! {
            <SplitContainer orientation=Orientation::Vertical style="height: 100vh">
                <SplitPane weight=1.0 base=Size::Pixels(0.0)>
                    <SplitContainer orientation=Orientation::Horizontal style="height: 100%; width: 100%">
                        <SplitPane weight=1. base=Size::Pixels(0.)>
                            <SplitContainer orientation=Orientation::Vertical style="height: 100%">
                                <SplitPane weight=1.0 base=Size::Pixels(0.0)>
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
                                </SplitPane>
                                <SplitPane weight=0. base=Size::Percent(20.) collapsed=!showing_issues class=classes!("tab")>
                                    <List<Issue>
                                        items=self.issues.clone()
                                        empty_text="No issues detected"
                                        onaction=self.link.callback(|index| Msg::GoToIssue(index))
                                        />
                                </SplitPane>
                            </SplitContainer>
                        </SplitPane>

                        <SplitPane weight=0. base=Size::Percent(20.) class=classes!("sidebar-right")>
                            <SplitContainer orientation=Orientation::Vertical style="height: 100%; overflow-y: auto">
                                <SplitPane weight=0.0 base=Size::Auto class=classes!("header")>
                                    { "Outline" }
                                </SplitPane>
                                <SplitPane weight=1.0 base=Size::Pixels(0.0) style="margin: 5px;">
                                    <AstView
                                        ast=self.ast.clone()
                                        onjump=self.link.callback(|pos: Pos| Msg::JumpToEditor {
                                            line: pos.line as u32, column: pos.column as u32
                                        })
                                        />
                                </SplitPane>
                            </SplitContainer>
                        </SplitPane>
                    </SplitContainer>
                </SplitPane>
                <SplitPane weight=0. base=Size::Percent(20.) class=classes!("tab")>
                    <SongView
                        ast=self.ast.clone()
                        onjump=self.link.callback(|pos: Pos| Msg::JumpToEditor {
                            line: pos.line as u32, column: pos.column as u32
                        })
                        />
                </SplitPane>
                <SplitPane weight=0. base=Size::Pixels(24.0) class=classes!("footer")>
                    <button
                        class=classes!("button-flat")
                        style="height: 100%;"
                        onclick=self.link.callback(move |_| Msg::ShowIssues(!showing_issues))
                        >{ format!("ⓧ {}", self.issues.len()) }</button>
                </SplitPane>
            </SplitContainer>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            // Show demo song
            let demo_song = r#"Song {
    bpm: 120
    sampleRate: 44_100
    meta: Meta {
        name: "Example Song"
        author: "John Doe"
        year: 2021
        description: "Simply.\nAwesome."
        awesome: true and not false
    }
    Track {
      name: "Lead"

      Sequence {
        start: 8/4
      }
    }
    // Test for comments
    Track {
      name: "Drums"
    }
}"#
            .to_string();
            self.editor.send_message(editor::Msg::Load {
                text: demo_song.clone(),
            });
            self.link.send_message(Msg::SourceCodeChanged(demo_song));
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
