#![warn(missing_docs)]
#![allow(unused)]

//! The layer based TUI rendering library.
//!
//! **THIS CRATE IS CURRENTLY BETA VERSION**  
//! README and this docs.rs is BETA ver, Information will update always.  

mod style;
mod tui;

use std::fmt::{Debug, Write};

pub use elapto_macros::{mk, widget};

type Identifier = String;
type Class = String;

trait Widget {
    fn type_name(&self) -> &'static str;

    fn properties(&self) -> Vec<(&str, String)>;
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
#[derive(Default)]
struct Fragment;

#[widget]
struct Embed {
    expanded: String,
}

impl Embed {
    fn new(expanded: String) -> Self {
        Self { expanded }
    }
}

#[widget]
struct Text {
    value: &'static str,
}

impl Text {
    fn new(value: &'static str) -> Self {
        Self { value }
    }
}

#[test]
fn test() {
    #[widget]
    #[derive(Default)]
    struct Foo {
        name: &'static str,
        expr: usize,
        bacte: bool,
    }

    let tag = mk!(<Foo name="John" expr={ 12 + 8 } bacte>"Hello" { "," } "World"</Foo>);

    eprintln!("{:?}", tag);

    panic!();
}
