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

use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct TreeNodeProps {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub label: String,
    #[prop_or(true)]
    pub expanded: bool,
}

pub struct TreeNode {
    link: ComponentLink<Self>,
    props: TreeNodeProps,
}

pub enum Msg {
    Toggle
}

impl Component for TreeNode {
    type Properties = TreeNodeProps;
    type Message = Msg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Toggle => {
                if ! self.props.children.is_empty() {
                    self.props.expanded = !self.props.expanded;
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
        let type_class = self.props.children.is_empty().then(|| "").unwrap_or("tree-node-inner");
        let expanded_class = self.props.expanded.then(|| "tree-node-expanded").unwrap_or("");

        html! {
            <div>
                <div
                    class=classes!("tree-node-label", type_class, expanded_class)
                    tabindex=0
                    onclick=self.link.callback(|e: MouseEvent| { e.stop_propagation(); Msg::Toggle })
                    >
                    {&self.props.label}
                </div>
                {
                    if ! self.props.children.is_empty() && self.props.expanded {
                        html! {
                            <div class="tree-node-children">
                                { for self.props.children.iter() }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }

}
