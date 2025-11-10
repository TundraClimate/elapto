#![warn(missing_docs)]
#![allow(unused)]

//! The layer based TUI rendering library.
//!
//! **THIS CRATE IS CURRENTLY BETA VERSION**  
//! README and this docs.rs is BETA ver, Information will update always.  

mod style;
mod tui;

pub use elapto_macros::mk;

#[test]
fn test() {
    struct Foo;

    let tag = mk!(<Foo a b="12" c={ 182 + 2 }>{ "Hello, World!" }</Foo>);

    assert_eq!(tag, "".to_string())
}
