use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};

use crate::impls::{Inline, Node, Property, Tag, Text};

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Tag(tag) => tag.to_tokens(tokens),
            Self::Inline(expr) => expr.to_tokens(tokens),
            Self::Text(text) => text.to_tokens(tokens),
        }
    }
}

impl ToTokens for Tag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name();
        let id = self.id().map(|(_, id)| quote! { id=#id });
        let class = self.class().map(|(_, class)| quote! { class=#class });
        let props = self
            .properties()
            .iter()
            .map(|(k, v)| {
                let v = match v {
                    Property::Text(ls) => quote! { #ls },
                    Property::Bool => quote! { true },
                    Property::Expr(expr) => quote! { { #expr } },
                };

                quote! { #k=#v }
            })
            .collect::<Vec<_>>();
        let children = self
            .children()
            .iter()
            .map(|node| quote! { #node })
            .collect::<Vec<_>>();

        let toks = quote! {
            <#name #id #class #(#props)*>#(#children)*</#name>
        };

        tokens.append_all(toks);
    }
}

impl ToTokens for Inline {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.inner;

        let toks = quote! {
            { #expr }
        };

        tokens.append_all(toks);
    }
}

impl ToTokens for Text {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let text = &self.inner;

        let toks = quote! {
            #text
        };

        tokens.append_all(toks);
    }
}
