use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{Expr, Ident, LitStr, Token, braced};

use crate::impls::{Inline, Node, Property, Tag, Text};

impl Parse for Node {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![<]) {
            Ok(Node::Tag(input.parse()?))
        } else if input.peek(LitStr) {
            Ok(Node::Text(input.parse()?))
        } else if input.peek(Brace) {
            let content;
            braced!(content in input);

            Ok(Node::Inline(content.parse()?))
        } else {
            Err(syn::Error::new(Span::call_site(), "unexpected token"))
        }
    }
}

impl Parse for Tag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;

        if input.peek(Token![>]) {
            input.parse::<Token![>]>()?;

            let children = parse_children(input)?;

            input.parse::<Token![<]>()?;
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;

            let children = children
                .into_iter()
                .map(|node| quote! { crate::mk!(#node) })
                .collect::<Vec<_>>();
            let children = syn::parse2::<Expr>(quote! { vec![ #(#children),* ] })?;
            let child_props = vec![(
                Ident::new("children", Span::call_site()),
                Property::Expr(children),
            )];

            return Ok(Self::fragment(child_props));
        }

        let name: Ident = input.parse()?;
        let mut id: Option<(Ident, Expr)> = None;
        let mut class: Option<(Ident, Expr)> = None;
        let mut properties = vec![];

        while !input.is_empty() {
            if input.peek(Token![/]) && input.peek2(Token![>]) {
                input.parse::<Token![/]>()?;
                input.parse::<Token![>]>()?;

                return Ok(Self::widget(name, id, class, properties, vec![]));
            }

            if input.peek(Token![>]) {
                break;
            }

            let k: Ident = input.parse()?;

            let v = if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;

                match input {
                    input if input.peek(LitStr) => Property::Text(input.parse()?),
                    input if input.peek(Brace) => {
                        let content;
                        braced!(content in input);

                        Property::Expr(content.parse()?)
                    }
                    _ => return Err(syn::Error::new(Span::call_site(), "unexpected tokens")),
                }
            } else {
                Property::Bool
            };

            if k == "id" {
                id = Some((k, v.into()));

                continue;
            }

            if k == "class" {
                class = Some((k, v.into()));

                continue;
            }

            properties.push((k, v));
        }

        input.parse::<Token![>]>()?;

        let children = parse_children(input)?;

        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;

        let close_name: Ident = input.parse()?;

        if name != close_name {
            return Err(syn::Error::new_spanned(close_name, "closing tag mismatch"));
        }

        input.parse::<Token![>]>()?;

        Ok(Self::widget(name, id, class, properties, children))
    }
}

impl Parse for Inline {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            inner: input.parse()?,
        })
    }
}

impl Parse for Text {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            inner: input.parse()?,
        })
    }
}

fn parse_children(input: ParseStream) -> syn::Result<Vec<Node>> {
    let mut children = vec![];

    while !input.is_empty() {
        if input.peek(Token![<]) && input.peek2(Token![/]) {
            break;
        }

        let node: Node = input.parse()?;

        children.push(node);
    }

    Ok(children)
}
