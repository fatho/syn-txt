use syntxt_lang::{ast, line_map::Pos};
use yew::prelude::*;

/// A component for displaying the syntxt AST as a tree.
pub struct SongView {
    props: SongViewProps,
}

#[derive(Debug, Clone, Properties)]
pub struct SongViewProps {
    pub ast: ast::NodePtr<ast::Root>,
    /// Callback for jumping to the corresponding source location when the user triggers an AST node
    #[prop_or_default]
    pub onjump: Callback<Pos>,
}

impl Component for SongView {
    type Message = ();
    type Properties = SongViewProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        render(&self.props.ast, self.props.onjump.clone())
    }
}

fn render(ast: &ast::Node<ast::Root>, onjump: Callback<Pos>) -> Html {
    for child in &ast.data.objects {
        if child.data.name.data == "Song" {
            return view_song(child, onjump);
        }
    }
    html! { { "Not a song" }}
}

fn view_song(ast: &ast::Node<ast::Object>, onjump: Callback<Pos>) -> Html {
    html! {
        <div style="height: 100%; width: 100%; overflow-x: hidden; overflow-y: auto;">
        {
            for
            ast.data.children.iter()
            .enumerate()
            .filter(|(_, child)| child.data.name.data == "Track")
            .map(|(index, child)| view_track(child, index, onjump.clone()))
        }
        </div>
    }
}

fn view_track(ast: &ast::Node<ast::Object>, index: usize, onjump: Callback<Pos>) -> Html {
    let mut name = None;
    for attr in &ast.data.attrs {
        if attr.data.name.data == "name" {
            // TODO: do we need to evaluate the expression here?
            if let ast::Expr::String(str) = &attr.data.value.data {
                name = Some(str.to_string());
            }
        }
    }
    let pos = ast.pos.start;
    let onclick = onjump.reform(move |e: MouseEvent| {
        e.stop_propagation();
        pos
    });
    html! {
        <div class=classes!("song-view-track")>
            <div class=classes!("song-view-track-header") style="width: 200px;">
                <div class=classes!("song-view-track-name") onclick=onclick>
                    { name.unwrap_or_else(|| format!("Track {}", index + 1)) }
                </div>
            </div>
        </div>
    }
}
