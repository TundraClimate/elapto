#![warn(missing_docs)]
#![allow(unused)]

//! The layer based TUI rendering library.
//!
//! **THIS CRATE IS CURRENTLY BETA VERSION**  
//! README and this docs.rs is BETA ver, Information will update always.  

mod style;
mod tui;

use std::fmt::{Debug, Write};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

pub use elapto_macros::{mk, widget};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct HashCell(u64);

impl HashCell {
    fn new<H: Hash>(obj: H) -> Self {
        let mut hasher = DefaultHasher::new();

        obj.hash(&mut hasher);

        Self(hasher.finish())
    }

    fn combine<H: Hash>(self, other: H) -> Self {
        let mut hasher = DefaultHasher::new();

        self.0.hash(&mut hasher);
        other.hash(&mut hasher);

        Self(hasher.finish())
    }
}

impl PartialEq<u64> for HashCell {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<HashCell> for u64 {
    fn eq(&self, other: &HashCell) -> bool {
        *self == other.0
    }
}

impl Debug for HashCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Hash, Clone)]
struct Identifier {
    id: String,
}

#[derive(Hash, Clone)]
struct Class {
    class: String,
}

trait Widget: WidgetInfo {
    fn render(&self, children: Vec<Component>) -> Component;

    fn to_dom(&self) -> Option<DomNode> {
        None
    }
}

trait WidgetInfo {
    fn type_name(&self) -> &'static str;

    fn properties(&self) -> Vec<(&str, String)>;

    fn gen_hash(&self) -> HashCell;
}

#[derive(Clone)]
struct Component {
    id: Option<Identifier>,
    class: Option<Class>,
    widget: Arc<dyn Widget>,
    children: Vec<Component>,
}

impl Component {
    fn new<W: Widget + 'static>(widget: W) -> Self {
        Self {
            id: None,
            class: None,
            widget: Arc::new(widget),
            children: vec![],
        }
    }

    fn set_id<T: Into<Identifier>>(mut self, id: T) -> Self {
        self.id = Some(id.into());

        self
    }

    fn set_class<T: Into<Class>>(mut self, class: T) -> Self {
        self.class = Some(class.into());

        self
    }

    fn with_child(mut self, children: Component) -> Self {
        self.children.push(children);

        self
    }

    fn gen_hash(&self) -> HashCell {
        self.children
            .iter()
            .fold(self.widget.gen_hash(), |acc, cpnt| acc.combine(cpnt))
            .combine(&self.id)
            .combine(&self.class)
    }

    fn to_dom_node(&self) -> Option<DomNode> {
        self.widget.to_dom()
    }

    fn render(&self) -> Component {
        self.widget.render(self.children.clone())
    }
}

impl Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = match self.id {
            Some(Identifier { ref id }) => format!(" id={id}"),
            None => "".to_string(),
        };

        let class = match self.class {
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

trait Expand {
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

#[widget]
#[derive(Default, Hash)]
struct Fragment {
    children: Vec<Component>,
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
    fn new(children: Vec<Component>) -> Self {
        Self { children }
    }

    fn new_from_iter<I: IntoIterator<Item = Component>>(children: I) -> Self {
        Self::new(children.into_iter().collect::<Vec<_>>())
    }
}

#[widget]
#[derive(Hash)]
struct Text {
    value: String,
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
    fn new<S: ToString>(value: S) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
struct DomContainer(DomAst);

#[derive(Debug, PartialEq)]
struct DomAst(HashCell, DomNode);

#[derive(Debug, PartialEq)]
enum DomNode {
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

#[test]
fn test() {
    #[widget]
    #[derive(Default, Hash)]
    struct Foo {
        name: &'static str,
        expr: usize,
        bacte: bool,
    }

    impl Widget for Foo {
        fn render(&self, children: Vec<Component>) -> Component {
            mk!(<>"Foo: " { children } </>)
        }
    }

    let tag =
        mk!(<Foo name="John" expr={ 12 + 8 } bacte>"Hello" { [mk!(""), mk!(",")] } "World"</Foo>);

    eprintln!("{:?}", parse_layer(tag));

    panic!();
}
