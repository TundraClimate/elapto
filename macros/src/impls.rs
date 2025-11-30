use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, ExprLit, Fields, Ident, ItemStruct, Lit, LitBool, LitStr};

mod expand;
mod parse;

enum Object {
    Tag(Tag),
    Inline(Inline),
    Text(Text),
}

struct Inline(Expr);

struct Text(LitStr);

enum Tag {
    Named(Named),
    Fragment(Fragment),
}

struct Named {
    name: Ident,
    id_ident: Option<Ident>,
    id: Option<Expr>,
    class_ident: Option<Ident>,
    class: Option<Expr>,
    properties: Vec<(Ident, Property)>,
    children: ObjectArray,
}

struct Fragment(ObjectArray);

enum Property {
    Text(LitStr),
    Bool,
    Expr(Expr),
}

impl From<Property> for Expr {
    fn from(val: Property) -> Self {
        match val {
            Property::Text(ls) => Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Str(ls),
            }),
            Property::Bool => Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Bool(LitBool::new(true, Span::call_site())),
            }),
            Property::Expr(expr) => expr,
        }
    }
}

struct ObjectArray(Vec<Object>);

pub(crate) fn parse_object(tokens: TokenStream) -> syn::Result<TokenStream> {
    let obj = syn::parse2::<Object>(tokens)?;
    let toks = expand::expand_object(obj);

    Ok(toks)
}

pub(crate) fn parse_widget(tokens: TokenStream) -> syn::Result<TokenStream> {
    let item = syn::parse2::<ItemStruct>(tokens)?;

    let attrs = &item.attrs;
    let vis = &item.vis;
    let name = &item.ident;
    let fields = &item.fields;

    let is_tuple = matches!(fields, Fields::Unnamed(_));

    if is_tuple {
        return Err(syn::Error::new(
            Span::call_site(),
            "expected a named struct",
        ));
    }

    let (expand_fields, props) = fields
        .iter()
        .map(|field| {
            let attrs = &field.attrs;
            let ident = &field.ident;
            let ty = &field.ty;

            (
                quote! { #(#attrs)* pub #ident: #ty, },
                quote! { (stringify!(#ident), format!("{:?}", self.#ident)), },
            )
        })
        .collect::<(Vec<_>, Vec<_>)>();

    Ok(quote! {
        #(#attrs)*
        #vis struct #name {
            #(#expand_fields)*
        }

        impl elapto::WidgetInfo for #name {
            fn type_name(&self) -> &'static str {
                ::std::any::type_name::<Self>()
            }

            fn properties(&self) -> Vec<(&str, String)> {
                vec![ #(#props)* ]
            }

            fn gen_hash(&self) -> elapto::HashCell {
                elapto::HashCell::new(&self)
            }
        }
    })
}
