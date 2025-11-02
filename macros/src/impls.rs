use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{Ident, Token};

struct WidgetTemplate {
    name: Ident,
}

impl Parse for WidgetTemplate {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        let ident: Ident = input.parse()?;
        input.parse::<Token![>]>()?;

        Ok(WidgetTemplate { name: ident })
    }
}

pub(crate) fn parse_tag(tokens: TokenStream) -> syn::Result<TokenStream> {
    let tag = syn::parse2::<WidgetTemplate>(tokens)?;
    let name = tag.name;

    Ok(quote! {
        format!("{}", stringify!(#name))
    })
}
