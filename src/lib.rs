#![warn(missing_docs)]
#![allow(unused)]

//! The layer based TUI rendering library.
//!
//! **THIS CRATE IS CURRENTLY BETA VERSION**  
//! README and this docs.rs is BETA ver, Information will update always.  

mod style;
mod tui;

use std::fmt::Debug;

pub use elapto_macros::{mk, widget};

type Identifier = String;
type Class = String;

trait Widget {}

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
        write!(f, "<id={:?} class={:?}>", self.id, self.class)
    }
}

trait Expand {
    fn expand(self) -> String;
}

#[widget]
#[derive(Default)]
struct Fragment;

#[widget]
#[derive(Default)]
struct Embed {
    expanded: String,
}

#[widget]
#[derive(Default)]
struct Text {
    value: &'static str,
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

    assert_eq!(format!("{:?}", tag), "".to_string())
}
