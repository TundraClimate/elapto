use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, TokenStreamExt, quote};
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{Expr, ExprLit, Fields, Ident, ItemStruct, Lit, LitBool, LitStr, Token, braced};

enum Node {
    Tag(Tag),
    Inline(Inline),
    Text(Text),
}

enum Tag {
    WidgetTemplate {
        name: Ident,
        id: Option<Expr>,
        class: Option<Expr>,
        properties: Vec<(Ident, Property)>,
        childrens: Vec<Node>,
    },
    FragmentTemplate {
        constant_name: Ident,
        dummy_props: Vec<(Ident, Property)>,
        childrens: Vec<Node>,
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

    fn id(&self) -> Option<&Expr> {
        match self {
            Self::WidgetTemplate { id, .. } => id.as_ref(),
            Self::FragmentTemplate { .. } => None,
        }
    }

    fn class(&self) -> Option<&Expr> {
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

    fn childrens(&self) -> &Vec<Node> {
        match self {
            Self::WidgetTemplate { childrens, .. } => childrens,
            Self::FragmentTemplate { childrens, .. } => childrens,
        }
    }

    fn widget(
        name: Ident,
        id: Option<Expr>,
        class: Option<Expr>,
        properties: Vec<(Ident, Property)>,
        childrens: Vec<Node>,
    ) -> Self {
        Self::WidgetTemplate {
            name,
            id,
            class,
            properties,
            childrens,
        }
    }

    fn fragment(childrens: Vec<Node>) -> Self {
        Self::FragmentTemplate {
            constant_name: Ident::new("Fragment", Span::call_site()),
            dummy_props: vec![],
            childrens,
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

impl Parse for Tag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;

        if input.peek(Token![>]) {
            input.parse::<Token![>]>()?;

            let childrens = parse_childrens(input)?;

            input.parse::<Token![<]>()?;
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;

            return Ok(Self::fragment(childrens));
        }

        let name: Ident = input.parse()?;
        let mut id: Option<Expr> = None;
        let mut class: Option<Expr> = None;
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
                id = Some(v.into());

                continue;
            }

            if k == "class" {
                class = Some(v.into());

                continue;
            }

            properties.push((k, v));
        }

        input.parse::<Token![>]>()?;

        let childrens = parse_childrens(input)?;

        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;

        let close_name: Ident = input.parse()?;

        if name != close_name {
            return Err(syn::Error::new_spanned(close_name, "closing tag mismatch"));
        }

        input.parse::<Token![>]>()?;

        Ok(Self::widget(name, id, class, properties, childrens))
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

fn parse_childrens(input: ParseStream) -> syn::Result<Vec<Node>> {
    let mut childrens = vec![];

    while !input.is_empty() {
        if input.peek(Token![<]) && input.peek2(Token![/]) {
            break;
        }

        let node: Node = input.parse()?;

        childrens.push(node);
    }

    Ok(childrens)
}

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
        let id = self.id().map(|id| quote! { id=#id });
        let class = self.class().map(|class| quote! { class=#class });
        let props = self
            .properties()
            .iter()
            .map(|(k, v)| {
                let v = match v {
                    Property::Text(ls) => quote! { #ls },
                    Property::Bool => quote! { true },
                    Property::Expr(expr) => quote! { #expr },
                };

                quote! { #k=#v }
            })
            .collect::<Vec<_>>();
        let childrens = self
            .childrens()
            .iter()
            .map(|node| quote! { #node })
            .collect::<Vec<_>>();

        let toks = quote! {
            <#name #id #class #(#props)*>#(#childrens)*</#name>
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

pub(crate) fn parse_tag(tokens: TokenStream) -> syn::Result<TokenStream> {
    let node = syn::parse2::<Node>(tokens)?;

    let id = match node {
        Node::Tag(ref tag) => {
            let id = tag.id();

            id.map(|id| quote! { .set_id(#id) })
        }
        _ => None,
    };

    let class = match node {
        Node::Tag(ref tag) => {
            let class = tag.class();

            class.map(|class| quote! { .set_class(#class) })
        }
        _ => None,
    };

    let widget = match node {
        Node::Tag(ref tag) => {
            let name = tag.name();
            let props = tag.properties();

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

            quote! {
                {
                    let mut p = #name::default();

                    #(#props)*

                    p
                }
            }
        }
        Node::Text(ref text) => {
            let text = &text.inner;

            quote! { crate::Text::new(#text) }
        }
        Node::Inline(ref expr) => {
            let expr = &expr.inner;

            quote! { crate::Embed::new(crate::Expand::expand(#expr)) }
        }
    };

    let childrens = match node {
        Node::Tag(ref tag) => {
            let childrens = tag.childrens();

            childrens
                .iter()
                .map(|children| quote! { .with_children(crate::mk!(#children)) })
                .collect::<Vec<_>>()
        }
        _ => vec![],
    };

    Ok(quote! {
        crate::Component::new(#widget)
            #id
            #class
            #(#childrens)*
    })
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

        impl crate::WidgetInfo for #name {
            fn type_name(&self) -> &'static str {
                ::std::any::type_name::<Self>()
            }

            fn properties(&self) -> Vec<(&str, String)> {
                vec![ #(#props)* ]
            }
        }
    })
}
