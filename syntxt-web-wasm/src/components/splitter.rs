use yew::prelude::*;

use super::Size;

#[derive(Debug, Clone)]
pub struct SplitContainer {
    link: ComponentLink<Self>,
    props: SplitContainerProps,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct SplitContainerProps {
    pub orientation: Orientation,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: String,
    #[prop_or_default]
    pub children: ChildrenWithProps<SplitPane>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Component for SplitContainer {
    type Message = ();
    type Properties = SplitContainerProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let direction = match self.props.orientation {
            Orientation::Horizontal => "row",
            Orientation::Vertical => "column",
        };
        let style = format!(
            "display: flex; flex-direction: {}; {}",
            direction, self.props.style
        );
        html! {
            <div style=style class=self.props.class.clone()>
                { for self.props.children.iter() }
            </div>
        }
    }
}

pub struct SplitPane {
    link: ComponentLink<Self>,
    props: SplitPaneProperties,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct SplitPaneProperties {
    #[prop_or(1.0)]
    pub weight: f64,
    #[prop_or(Size::Auto)]
    pub base: Size,
    #[prop_or(false)]
    pub collapsed: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: String,
    #[prop_or_default]
    pub children: Children,
}

impl Component for SplitPane {
    type Message = ();
    type Properties = SplitPaneProperties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let display = if self.props.collapsed { "display: none;" } else {""};
        let style = format!(
            "flex: {} {} {}; min-width: 0; min-height: 0; {}{}",
            self.props.weight, self.props.weight, self.props.base, display, self.props.style
        );
        html! {
            <div style=style class=self.props.class.clone()>
                { for self.props.children.iter() }
            </div>
        }
    }
}
