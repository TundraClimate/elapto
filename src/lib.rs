/* #![warn(missing_docs)] */
#![allow(unused)]

//! The layer based TUI rendering library.
//!
//! **THIS CRATE IS CURRENTLY BETA VERSION**  
//! README and this docs.rs is BETA ver, Information will update always.  

mod hash_cell;
mod style;
mod tui;

use std::fmt::{Debug, Write};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

pub use elapto_macros::{mk, widget};
pub use hash_cell::HashCell;

#[derive(Hash, Clone)]
pub struct Identifier {
    pub id: String,
}

#[derive(Hash, Clone)]
pub struct Class {
    pub class: String,
}

pub trait Widget: WidgetInfo {
    fn render(&self, children: Vec<Component>) -> Component;

    fn to_dom(&self) -> Option<DomNode> {
        None
    }
}

pub trait WidgetInfo {
    fn type_name(&self) -> &'static str;

    fn properties(&self) -> Vec<(&str, String)>;

    fn gen_hash(&self) -> HashCell;
}

#[derive(Clone, Hash)]
pub struct SubProperties {
    id: Option<Identifier>,
    class: Option<Class>,
    children: Vec<Component>,
}

#[derive(Clone)]
pub struct Component {
    widget: Arc<dyn Widget>,
    sub_props: SubProperties,
}

impl SubProperties {
    pub fn with_id<T: Into<Identifier>>(mut self, id: T) -> Self {
        self.id = Some(id.into());

        self
    }

    pub fn with_class<T: Into<Class>>(mut self, class: T) -> Self {
        self.class = Some(class.into());

        self
    }

    pub fn with_children(mut self, children: Vec<Component>) -> Self {
        self.children = children;

        self
    }
}

impl Component {
    pub fn new<W: Widget + 'static>(widget: W) -> Self {
        Self {
            widget: Arc::new(widget),
            sub_props: SubProperties {
                id: None,
                class: None,
                children: vec![],
            },
        }
    }

    pub fn with_sub_props<F: FnOnce(SubProperties) -> SubProperties>(mut self, f: F) -> Self {
        self.sub_props = f(self.sub_props);

        self
    }

    fn gen_hash(&self) -> HashCell {
        self.widget.gen_hash().combine(&self.sub_props)
    }

    fn to_dom_node(&self) -> Option<DomNode> {
        self.widget.to_dom()
    }

    fn render(&self) -> Component {
        self.widget.render(self.sub_props.children.clone())
    }
}

impl Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = match self.sub_props.id {
            Some(Identifier { ref id }) => format!(" id={id}"),
            None => "".to_string(),
        };

        let class = match self.sub_props.class {
            Some(Class { ref class }) => format!(" class={class}"),
            None => "".to_string(),
        };

        let props = self
            .widget
            .properties()
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ");

        let children = self
            .sub_props
            .children
            .iter()
            .map(|cpnt| format!("\t{cpnt:?}\n"))
            .collect::<String>();

        let (children, close) = if children.is_empty() {
            (children, "")
        } else {
            (format!("\n{children}"), "</>")
        };

        write!(
            f,
            "<{}{}{} {}>{}{}",
            self.widget.type_name(),
            id,
            class,
            props,
            children,
            close,
        )
    }
}

impl Hash for Component {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.gen_hash().hash(state);
    }
}

impl PartialEq for Component {
    fn eq(&self, other: &Self) -> bool {
        self.gen_hash() == other.gen_hash()
    }
}

pub trait Expand {
    fn expand(self) -> Component;
}

macro_rules! impl_expand_to_string {
    ($($ty:ty),*) => {
        $(impl Expand for $ty {
            fn expand(self) -> Component {
                Component::new(Text::new(self))
            }
        })*
    };
}

macro_rules! impl_expand_iter {
    ($($ty:ty),*) => {
        $(impl Expand for $ty {
            fn expand(self) -> Component {
                let arr = self.into_iter().collect::<Vec<Component>>();

                if arr.len() == 1 {
                    arr[0].clone()
                } else {
                    Component::new(Fragment::new(arr))
                }
            }
        })*
    };
}

impl Expand for Component {
    fn expand(self) -> Component {
        self
    }
}

impl<const N: usize> Expand for [Component; N] {
    fn expand(self) -> Component {
        if N == 1 {
            self[0].clone()
        } else {
            Component::new(Fragment::new_from_iter(self))
        }
    }
}

impl Expand for (Component, Component) {
    fn expand(self) -> Component {
        Component::new(Fragment::new_from_iter([self.0, self.1]))
    }
}

impl Expand for (Component, Component, Component) {
    fn expand(self) -> Component {
        Component::new(Fragment::new_from_iter([self.0, self.1, self.2]))
    }
}

impl_expand_to_string!(
    String, &str, usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128, bool, f32, f64,
    char
);
impl_expand_iter!(Vec<Component>);

#[widget(default)]
pub struct Fragment {
    pub children: Vec<Component>,
}

impl Widget for Fragment {
    fn render(&self, children: Vec<Component>) -> Component {
        panic!("Cannot call a 'render' method in the Fragment widget")
    }

    fn to_dom(&self) -> Option<DomNode> {
        if self.children.is_empty() {
            Some(DomNode::Ignore)
        } else {
            Some(DomNode::Vector(
                self.children
                    .iter()
                    .map(|cpnt| parse_component(cpnt.clone()))
                    .collect::<Vec<_>>(),
            ))
        }
    }
}

impl Fragment {
    pub fn new(children: Vec<Component>) -> Self {
        Self { children }
    }

    fn new_from_iter<I: IntoIterator<Item = Component>>(children: I) -> Self {
        Self::new(children.into_iter().collect::<Vec<_>>())
    }
}

#[widget(default)]
pub struct Text {
    pub value: String,
}

impl Widget for Text {
    fn render(&self, children: Vec<Component>) -> Component {
        panic!("Cannot call a 'render' method in the Text widget")
    }

    fn to_dom(&self) -> Option<DomNode> {
        Some(DomNode::Text(self.value.clone()))
    }
}

impl Text {
    pub fn new<S: ToString>(value: S) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
struct DomContainer(DomAst);

#[derive(Debug, PartialEq)]
pub struct DomAst(HashCell, DomNode);

#[derive(Debug, PartialEq)]
pub enum DomNode {
    Layer(Box<DomAst>),
    Vector(Vec<DomAst>),
    Text(String),
    NewLine,
    Ignore,
}

impl DomAst {
    fn new(cell: HashCell, node: DomNode) -> Self {
        Self(cell, node)
    }
}

fn parse_layer(original_component: Component) -> DomContainer {
    let root_hash = original_component.gen_hash();
    let expanded_root = original_component.render();

    let ast = DomAst::new(
        root_hash,
        DomNode::Layer(Box::new(parse_component(expanded_root))),
    );

    DomContainer(ast)
}

fn parse_component(cpnt: Component) -> DomAst {
    let cell = cpnt.gen_hash();

    if let Some(dom) = cpnt.to_dom_node() {
        return DomAst::new(cell, dom);
    }

    DomAst::new(
        cell,
        DomNode::Layer(Box::new(parse_component(cpnt.render()))),
    )
}
