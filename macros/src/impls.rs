use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, ExprLit, Fields, Ident, ItemStruct, Lit, LitBool, LitStr};

mod parse;
mod to_tokens;

enum Node {
    Tag(Tag),
    Inline(Inline),
    Text(Text),
}

enum Tag {
    WidgetTemplate {
        name: Ident,
        id: Option<(Ident, Expr)>,
        class: Option<(Ident, Expr)>,
        properties: Vec<(Ident, Property)>,
        children: Vec<Node>,
    },
    FragmentTemplate {
        constant_name: Ident,
        dummy_props: Vec<(Ident, Property)>,
        children: Vec<Node>,
    },
}

struct Inline {
    inner: Expr,
}

struct Text {
    inner: LitStr,
}

enum Property {
    Text(LitStr),
    Bool,
    Expr(Expr),
}

impl Tag {
    fn name(&self) -> &Ident {
        match self {
            Self::WidgetTemplate { name, .. } => name,
            Self::FragmentTemplate { constant_name, .. } => constant_name,
        }
    }

    fn id(&self) -> Option<&(Ident, Expr)> {
        match self {
            Self::WidgetTemplate { id, .. } => id.as_ref(),
            Self::FragmentTemplate { .. } => None,
        }
    }

    fn class(&self) -> Option<&(Ident, Expr)> {
        match self {
            Self::WidgetTemplate { class, .. } => class.as_ref(),
            Self::FragmentTemplate { .. } => None,
        }
    }

    fn properties(&self) -> &Vec<(Ident, Property)> {
        match self {
            Self::WidgetTemplate { properties, .. } => properties,
            Self::FragmentTemplate { dummy_props, .. } => dummy_props,
        }
    }

    fn children(&self) -> &Vec<Node> {
        match self {
            Self::WidgetTemplate { children, .. } => children,
            Self::FragmentTemplate { children, .. } => children,
        }
    }

    fn widget(
        name: Ident,
        id: Option<(Ident, Expr)>,
        class: Option<(Ident, Expr)>,
        properties: Vec<(Ident, Property)>,
        children: Vec<Node>,
    ) -> Self {
        Self::WidgetTemplate {
            name,
            id,
            class,
            properties,
            children,
        }
    }

    fn fragment(children: Vec<Node>) -> Self {
        Self::FragmentTemplate {
            constant_name: Ident::new("Fragment", Span::call_site()),
            dummy_props: vec![],
            children,
        }
    }
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

pub(crate) fn parse_tag(tokens: TokenStream) -> syn::Result<TokenStream> {
    let node = syn::parse2::<Node>(tokens)?;

    let cpnt = match node {
        Node::Tag(ref tag) => {
            let name = tag.name();
            let props = tag.properties();
            let id = tag
                .id()
                .map(|(id_ident, id)| quote! { .set_id(elapto::Identifier { #id_ident: From::from(#id) }) });
            let class = tag.class().map(
                |(class_ident, class)| quote! { .set_class(elapto::Class { #class_ident: From::from(#class) }) },
            );

            let props = props
                .iter()
                .map(|(k, v)| {
                    let v = match v {
                        Property::Text(ls) => quote! { #ls },
                        Property::Bool => quote! { true },
                        Property::Expr(expr) => quote! { #expr },
                    };

                    quote! { p.#k = #v; }
                })
                .collect::<Vec<_>>();

            let children = tag.children();

            let fragment_patch = if name == "Fragment" {
                Some(quote! { p.children = vec![ #(elapto::mk!(#children)),* ]; })
            } else {
                None
            };

            let children = match fragment_patch {
                Some(_) => vec![],
                None => children
                    .iter()
                    .map(|child| quote! { .with_child(elapto::mk!(#child)) })
                    .collect::<Vec<_>>(),
            };

            let name = match fragment_patch {
                Some(_) => quote! { elapto::Fragment },
                None => quote! { #name },
            };

            quote! {
                elapto::Component::new({
                    let mut p = #name::default();

                    #fragment_patch
                    #(#props)*

                    p
                })
                .with_sub_props(|p| {
                    p
                    #id
                    #class
                    #(#children)*
                })
            }
        }
        Node::Text(ref text) => {
            let text = &text.inner;

            quote! { elapto::Component::new(elapto::Text::new(#text)) }
        }
        Node::Inline(ref expr) => {
            let expr = &expr.inner;

            quote! { elapto::Expand::expand(#expr) }
        }
    };

    Ok(cpnt)
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
