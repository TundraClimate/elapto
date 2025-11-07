#![warn(missing_docs)]
#![allow(unused)]

//! The layer based TUI rendering library.
//!
//! **THIS CRATE IS CURRENTLY BETA VERSION**  
//! README and this docs.rs is BETA ver, Information will update always.  

mod style;
mod tui;

use elapto_macros::tag_macro;

#[test]
fn test() {
    struct Foo;

    let tag = tag_macro!(<Foo a b="12" c={ 182 }>{ "" }</Foo>);

    assert_eq!(tag, "".to_string())
}
