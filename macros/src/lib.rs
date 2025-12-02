#![allow(clippy::large_enum_variant)]

extern crate proc_macro;

use proc_macro::TokenStream;

mod impls;

#[proc_macro]
pub fn mk(tokens: TokenStream) -> TokenStream {
    impls::parse_object(tokens.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn widget(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    impls::parse_widget(attr.into(), tokens.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
