use syntxt_lang::{
    ast::{self, Walk},
    line_map::Pos,
};
use yew::prelude::*;

use crate::components::tree::TreeNode;

/// A component for displaying the syntxt AST as a tree.
pub struct AstView {
    props: AstViewProps,
}

#[derive(Debug, Clone, Properties)]
pub struct AstViewProps {
    pub ast: ast::NodePtr<ast::Root>,
    /// Callback for jumping to the corresponding source location when the user triggers an AST node
    #[prop_or_default]
    pub onjump: Callback<Pos>,
}

impl Component for AstView {
    type Message = ();
    type Properties = AstViewProps;

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
        AstTreeVisitor::view(&self.props.ast, self.props.onjump.clone())
    }
}

/// Visitor constructing a HTML tree view while traversing the AST.
struct AstTreeVisitor {
    children: Vec<Html>,
    /// Stores the number of elements that were in children before descending into the tree.
    /// When ascending back, any extra `children` on top of the number popped the stack
    /// are the direct children of the element that was created.
    stack: Vec<usize>,
    onjump: Callback<Pos>,
}

impl AstTreeVisitor {
    pub fn view(ast: &ast::Node<ast::Root>, onjump: Callback<Pos>) -> Html {
        let mut visitor = AstTreeVisitor {
            children: vec![],
            stack: vec![],
            onjump,
        };
        ast.walk(&mut visitor);
        visitor.children.drain(..).collect::<Html>()
    }

    fn begin(&mut self) {
        self.stack.push(self.children.len());
    }

    fn finish<S: AsRef<str>>(&mut self, label: S, pos: Pos) {
        let scope_start = self.stack.pop().unwrap();
        let onaction = self.onjump.reform(move |()| pos);
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
            // TODO: Implement AST walking for notes
            ast::Expr::Sequence(seq) => self.leaf(format!("{:?}", seq), node),
        }
    }
}
