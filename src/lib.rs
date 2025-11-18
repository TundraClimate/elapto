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
}

impl Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = match self.id {
            Some(ref id) => format!(" id={id}"),
            None => "".to_string(),
        };

        let class = match self.class {
            Some(ref class) => format!(" class={class}"),
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
                    Component::new(Embed::new(arr))
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
            Component::new(Embed::new_from_iter(self))
        }
    }
}

impl Expand for (Component, Component) {
    fn expand(self) -> Component {
        Component::new(Embed::new_from_iter([self.0, self.1]))
    }
}

impl Expand for (Component, Component, Component) {
    fn expand(self) -> Component {
        Component::new(Embed::new_from_iter([self.0, self.1, self.2]))
    }
}

impl_expand_to_string!(
    String, &str, usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128, bool, f32, f64,
    char
);
impl_expand_iter!(Vec<Component>);

#[widget]
#[derive(Default, Hash)]
struct Fragment;

impl Widget for Fragment {}

#[widget]
#[derive(Hash)]
struct Embed {
    inner: Vec<Component>,
}

impl Widget for Embed {}

impl Embed {
    fn new(inner: Vec<Component>) -> Self {
        Self { inner }
    }

    fn new_from_iter<I: IntoIterator<Item = Component>>(inner: I) -> Self {
        Self::new(inner.into_iter().collect::<Vec<_>>())
    }
}

#[widget]
#[derive(Hash)]
struct Text {
    value: String,
}

impl Widget for Text {}

impl Text {
    fn new<S: ToString>(value: S) -> Self {
        Self {
            value: value.to_string(),
        }
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

    let tag =
        mk!(<Foo name="John" expr={ 12 + 8 } bacte>"Hello" { [mk!(""), mk!(",")] } "World"</Foo>);

    eprintln!("{:?}", tag);

    panic!();
}
