use proc_macro2::{Span, TokenStream, TokenTree};
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{Fields, Ident, ItemStruct, LitStr, Token, braced};

use crate::impls::{
    Fragment, Inline, Named, Object, ObjectArray, Property, Tag, Text, WidgetStruct,
};

impl Parse for Object {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![<]) {
            Ok(Object::Tag(input.parse()?))
        } else if input.peek(LitStr) {
            Ok(Object::Text(Text(input.parse()?)))
        } else if input.peek(Brace) {
            let content;
            braced!(content in input);

            Ok(Object::Inline(Inline(content.parse()?)))
        } else {
            Err(syn::Error::new(Span::call_site(), "unexpected token"))
        }
    }
}

impl Parse for Tag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if !input.peek(Token![<]) {
            Err(syn::Error::new(Span::call_site(), "expected '<'"))
        } else if input.peek2(Ident) {
            Ok(Tag::Named(input.parse()?))
        } else if input.peek2(Token![>]) {
            Ok(Tag::Fragment(input.parse()?))
        } else {
            Err(syn::Error::new(Span::call_site(), "unexpected token"))
        }
    }
}

impl Parse for Named {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;

        let name: Ident = input.parse()?;

        let mut id = None;
        let mut class = None;
        let mut properties = vec![];

        while !input.is_empty() {
            if input.peek(Token![/]) || input.peek(Token![>]) {
                break;
            }

            let key: Ident = input.parse()?;

            let prop = if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;

                if input.peek(LitStr) {
                    Property::Text(input.parse()?)
                } else if input.peek(Brace) {
                    let content;
                    braced!(content in input);

                    Property::Expr(content.parse()?)
                } else {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "expected value (e.g. \"\", { expr })",
                    ));
                }
            } else {
                Property::Bool
            };

            if key == "id" {
                id = Some((key, prop.into()));

                continue;
            }

            if key == "class" {
                class = Some((key, prop.into()));

                continue;
            }

            properties.push((key, prop));
        }

        let (id_ident, id) = id.unzip();
        let (class_ident, class) = class.unzip();

        if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;

            let children = ObjectArray(vec![]);

            return Ok(Self {
                name,
                id_ident,
                id,
                class,
                class_ident,
                properties,
                children,
            });
        }

        input.parse::<Token![>]>()?;

        let children: ObjectArray = syn::parse2(in_tag(input)?)?;

        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;

        let close_name: Ident = input.parse()?;

        if name != close_name {
            return Err(syn::Error::new(Span::call_site(), "mismatch close tag"));
        }

        input.parse::<Token![>]>()?;

        Ok(Self {
            name,
            id_ident,
            id,
            class_ident,
            class,
            properties,
            children,
        })
    }
}

impl Parse for Fragment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        input.parse::<Token![>]>()?;

        let children: ObjectArray = syn::parse2(in_tag(input)?)?;

        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        input.parse::<Token![>]>()?;

        Ok(Self(children))
    }
}

impl Parse for ObjectArray {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut children = vec![];

        while !input.is_empty() {
            let obj: Object = input.parse()?;

            children.push(obj);
        }

        Ok(Self(children))
    }
}

fn in_tag(input: ParseStream) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();

    let mut depth = 0usize;

    while !input.is_empty() {
        if input.peek(Token![<]) {
            if input.peek2(Token![/]) {
                if depth == 0 {
                    break;
                } else {
                    depth -= 1;
                }
            } else {
                depth += 1;
            }
        }

        let tt: TokenTree = input.parse()?;

        tokens.extend(Some(tt));
    }

    if depth != 0 {
        return Err(syn::Error::new(Span::call_site(), "unclosed tag"));
    }

    Ok(tokens)
}

impl Parse for WidgetStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: ItemStruct = input.parse()?;

        if matches!(item.fields, Fields::Unnamed(_)) {
            return Err(syn::Error::new(
                Span::call_site(),
                "expected a named struct",
            ));
        }

        let attrs = item.attrs;
        let vis = item.vis;
        let name = item.ident;
        let fields = item.fields;

        Ok(WidgetStruct {
            attrs,
            vis,
            name,
            fields,
        })
    }
}
