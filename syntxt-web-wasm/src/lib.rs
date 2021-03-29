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

use std::vec;

use syntxt_lang::{
    ast::{self, Walk},
    line_map::Pos,
};
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
    tree::TreeNode,
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
    ast: ast::Node<ast::Root>,
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
            showing_issues: true,
            ast: ast::Node {
                span: 0..0,
                pos: Pos::origin()..Pos::origin(),
                data: ast::Root { objects: vec![] },
            },
            issues: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SourceCodeChanged(code) => {
                self.issues.clear();
                match syntxt_lang::parser::Parser::parse(&code) {
                    Ok(ast) => {
                        self.ast = ast;
                    }
                    Err((partial_ast, errors)) => {
                        self.ast = partial_ast;
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
        let issue_tab_class = if self.showing_issues {
            ""
        } else {
            "tab-inactive"
        };
        html! {
            <section style="height: 100vh; display: flex; flex-direction: column">
                <header style="flex: 0 0 0px; background-color: black">
                </header>
                <div style="flex: 1 1 0px; min-height: 0; min-width: 0; display: flex; flex-direction: row;">
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
                    <section class="sidebar-right" style="flex: 0 0 20%; min-width: 0; height: 100%; display: flex; flex-direction: column">
                        <header class="header" style="flex: 0 0;">
                            { "Outline" }
                        </header>
                        <div style="margin: 5px; flex: 1 1 0; min-height: 0; overflow-y: auto">
                            { AstTreeVisitor::view(&self.ast, self.link.clone()) }
                        </div>
                    </section>
                </div>
                <div class=classes!("tab", issue_tab_class) style="flex: 0 0 20%; min-height: 0; overflow: auto">
                    <List<Issue>
                        items=self.issues.clone()
                        empty_text="No issues detected"
                        onaction=self.link.callback(|index| Msg::GoToIssue(index))
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
}"#.to_string();
            self.editor.send_message(editor::Msg::Load { text: demo_song.clone() });
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

struct AstTreeVisitor {
    children: Vec<Html>,
    stack: Vec<usize>,
    link: ComponentLink<AppModel>,
}

impl AstTreeVisitor {
    pub fn view(ast: &ast::Node<ast::Root>, link: ComponentLink<AppModel>) -> Html {
        let mut visitor = AstTreeVisitor {
            children: vec![],
            stack: vec![],
            link: link.clone(),
        };
        ast.walk(&mut visitor);
        visitor.children.drain(..).collect::<Html>()
    }

    fn begin(&mut self) {
        self.stack.push(self.children.len());
    }

    fn finish<S: AsRef<str>>(&mut self, label: S, pos: Pos) {
        let scope_start = self.stack.pop().unwrap();
        let onaction = self.link.callback(move |()| Msg::JumpToEditor {
            line: pos.line as u32,
            column: pos.column as u32,
        });
        let node = if scope_start < self.children.len() {
            html! {
                <TreeNode
                    label=label.as_ref()
                    onaction=onaction
                    >
                    { self.children.drain(scope_start..).collect::<Html>() }
                </TreeNode>
            }
        } else {
            html! {
                <TreeNode label=label.as_ref() onaction=onaction />
            }
        };
        self.children.push(node);
    }

    fn leaf<S: AsRef<str>, T>(&mut self, label: S, node: &ast::Node<T>) {
        self.begin();
        self.finish(label, node.pos.start)
    }

    fn nested<S: AsRef<str>, T: Walk>(&mut self, label: S, node: &ast::Node<T>) {
        self.begin();
        node.walk(self);
        self.finish(label, node.pos.start);
    }
}

impl ast::Visitor for AstTreeVisitor {
    fn root(&mut self, node: &ast::Node<ast::Root>) {
        self.nested("Root", node);
    }

    fn object(&mut self, node: &ast::Node<ast::Object>) {
        self.nested(format!("Object: {}", node.data.name.data), node);
    }

    fn attribute(&mut self, node: &ast::Node<ast::Attribute>) {
        self.nested(format!("{}:", node.data.name.data), node);
    }

    fn expression(&mut self, node: &ast::Node<ast::Expr>) {
        match &node.data {
            // leaf nodes
            ast::Expr::String(x) => self.leaf(format!("{:?}", x), node),
            ast::Expr::Int(x) => self.leaf(format!("{}", x), node),
            ast::Expr::Ratio(x) => self.leaf(format!("{}", x), node),
            ast::Expr::Float(x) => self.leaf(format!("{}", x), node),
            ast::Expr::Bool(x) => self.leaf(format!("{:?}", x), node),
            ast::Expr::Var(x) => self.leaf(format!("{}", x), node),
            // nested expressions
            ast::Expr::Unary { operator, .. } => self.nested(format!("{:?}", operator.data), node),
            ast::Expr::Binary { operator, .. } => self.nested(format!("{:?}", operator.data), node),
            ast::Expr::Paren { .. } => self.nested("()", node),
            ast::Expr::Accessor { attribute, .. } => {
                self.nested(format!(".{}", attribute.data), node)
            }
            ast::Expr::Call { .. } => self.nested("Call", node),
            // nested, but not an expression, hide expression node
            ast::Expr::Object(obj) => obj.walk(self),
        }
    }
}
