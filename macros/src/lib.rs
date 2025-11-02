extern crate proc_macro;

use proc_macro::TokenStream;

mod impls;

#[proc_macro]
pub fn tag_macro(tokens: TokenStream) -> TokenStream {
    impls::parse_tag(tokens.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
