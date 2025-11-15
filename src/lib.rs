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

type Identifier = String;
type Class = String;

trait Widget: WidgetInfo {
    fn render(&self) -> Component {
        unimplemented!()
    }
}

trait WidgetInfo {
    fn type_name(&self) -> &'static str;

    fn properties(&self) -> Vec<(&str, String)>;

    fn gen_hash(&self) -> HashCell;
}

struct Component {
    id: Option<Identifier>,
    class: Option<Class>,
    widget: Box<dyn Widget>,
    childrens: Vec<Component>,
}

impl Component {
    fn new<W: Widget + 'static>(widget: W) -> Self {
        Self {
            id: None,
            class: None,
            widget: Box::new(widget),
            childrens: vec![],
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

    fn with_children(mut self, children: Component) -> Self {
        self.childrens.push(children);

        self
    }

    fn gen_hash(&self) -> HashCell {
        self.widget
            .gen_hash()
            .combine(&self.id)
            .combine(&self.class)
    }
}

impl Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let props = self
            .widget
            .properties()
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ");

        let childrens = self
            .childrens
            .iter()
            .map(|cpnt| format!("\t{cpnt:?}\n"))
            .collect::<String>();

        let childrens = if childrens.is_empty() {
            childrens
        } else {
            format!("\n{childrens}")
        };

        write!(
            f,
            "<{} id={:?} class={:?} {}>{}</>",
            self.widget.type_name(),
            self.id,
            self.class,
            props,
            childrens,
        )
    }
}

trait Expand {
    fn expand(self) -> String;
}

impl<T: ToString> Expand for T {
    fn expand(self) -> String {
        self.to_string()
    }
}

#[widget]
#[derive(Default, Hash)]
struct Fragment;

impl Widget for Fragment {}

#[widget]
#[derive(Hash)]
struct Embed {
    expanded: String,
}

impl Widget for Embed {}

impl Embed {
    fn new(expanded: String) -> Self {
        Self { expanded }
    }
}

#[widget]
#[derive(Hash)]
struct Text {
    value: &'static str,
}

impl Widget for Text {}

impl Text {
    fn new(value: &'static str) -> Self {
        Self { value }
    }
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

    impl Widget for Foo {}

    let tag = mk!(<Foo name="John" expr={ 12 + 8 } bacte>"Hello" { "," } "World"</Foo>);

    eprintln!("{:?}", tag);

    panic!();
}
