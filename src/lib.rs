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

struct Component {
    id: Option<Identifier>,
    class: Option<Class>,
}

impl Component {
    fn new() -> Self {
        Self {
            id: None,
            class: None,
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
}

impl Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<id={:?} class={:?}>", self.id, self.class)
    }
}

#[test]
fn test() {
    struct Foo;

    let tag = mk!(<Foo a b="12" c={ 182 + 2 }>{ "Hello, World!" }</Foo>);

    assert_eq!(format!("{:?}", tag), "".to_string())
}
