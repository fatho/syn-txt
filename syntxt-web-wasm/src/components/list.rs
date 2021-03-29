use yew::prelude::*;

#[derive(Clone)]
pub struct List<Item: ListItem + 'static> {
    link: ComponentLink<Self>,
    items: Vec<Item>,
    onclick: Callback<usize>,
    empty_text: String,
}

pub trait ListItem: PartialEq + Clone {
    fn view(&self) -> Html;
}

#[derive(Properties, Clone)]
pub struct Props<Item: Clone> {
    #[prop_or_default]
    pub onclick: Callback<usize>,
    #[prop_or_default]
    pub items: Vec<Item>,
    #[prop_or("Empty".into())]
    pub empty_text: String,
}

impl<Item: ListItem + 'static> Component for List<Item> {
    type Message = ();
    type Properties = Props<Item>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            items: props.items,
            onclick: props.onclick,
            empty_text: props.empty_text,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let mut changed = false;
        if props.items != self.items {
            self.items = props.items;
            changed = true;
        }
        if props.onclick != self.onclick {
            self.onclick = props.onclick;
            changed = true;
        }
        if props.empty_text != self.empty_text {
            self.empty_text = props.empty_text;
            changed = true;
        }
        changed
    }

    fn view(&self) -> Html {
        html! {
            <div class=classes!("list-flat") style="overflow-x: hidden; overflow-y: auto">
            {
                if self.items.is_empty() {
                    html! {
                        <div class=classes!("list-item-flat") disabled=true>
                            <span style="color:red; font-weight: bold; visibility: hidden; margin-right: 5px">{"ⓧ"}</span>
                            <span>{&self.empty_text}</span>
                        </div>
                    }
                } else {
                    self.items.iter()
                        .enumerate()
                        .map(|(index, item)| self.render_item(index, item))
                        .collect::<Html>()
                }
            }
            </div>
        }
    }
}

impl<Item: ListItem + 'static> List<Item> {
    fn render_item(&self, index: usize, item: &Item) -> Html {
        let onclick = self.onclick.reform(move |e: MouseEvent| {
            e.stop_propagation();
            index
        });
        html! {
            <div class=classes!("list-item-flat") tabindex={index} onclick=onclick>
                { item.view() }
            </div>
        }
    }
}
