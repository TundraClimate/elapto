#![warn(missing_docs)]
#![allow(unused)]

//! The layer based TUI rendering library.
//!
//! **THIS CRATE IS CURRENTLY BETA VERSION**  
//! README and this docs.rs is BETA ver, Information will update always.  

mod style;
mod tui;

use std::fmt::Debug;

pub use elapto_macros::mk;

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

#[test]
fn test() {
    #[derive(Default)]
    struct Foo {
        pub name: &'static str,
        pub expr: usize,
        pub bacte: bool,
    }

    impl Widget for Foo {}

    let tag = mk!(<Foo name="John" expr={ 12 + 8 } bacte><Foo /></Foo>);

    assert_eq!(format!("{:?}", tag), "".to_string())
}
