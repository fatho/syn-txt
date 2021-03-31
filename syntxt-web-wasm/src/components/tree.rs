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

#[allow(unused_imports)]
use crate::console_log;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct TreeNodeProps {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub label: String,
    #[prop_or_default]
    pub onaction: Callback<()>,
}

pub struct TreeNode {
    link: ComponentLink<Self>,
    props: TreeNodeProps,
    expanded: bool,
}

pub enum Msg {
    Toggle,
}

impl Component for TreeNode {
    type Properties = TreeNodeProps;
    type Message = Msg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            expanded: true,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Toggle => {
                if !self.props.children.is_empty() {
                    self.expanded = !self.expanded;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        // what type of node is this? Leaf node or inner node
        let type_class = self
            .props
            .children
            .is_empty()
            .then(|| "")
            .unwrap_or("tree-node-inner");
        let expanded_class = self.expanded.then(|| "tree-node-expanded").unwrap_or("");
        let container_visible = self.expanded && !self.props.children.is_empty();
        let container_class = container_visible.then(|| "").unwrap_or("invisible");

        let ondblclick = self.props.onaction.reform(|e: MouseEvent| {
            e.stop_propagation();
        });

        html! {
            <div>
                <div
                    class=classes!("tree-node-label")
                    tabindex=0
                    ondblclick=ondblclick
                    >
                    <span
                        class=classes!(type_class, expanded_class)
                        onclick=self.link.callback(|e: MouseEvent| { e.stop_propagation(); Msg::Toggle })
                        />
                    <span>
                        {&self.props.label}
                    </span>
                </div>
                <div class=classes!("tree-node-children", container_class)>
                    { for self.props.children.iter() }
                </div>
            </div>
        }
    }
}
