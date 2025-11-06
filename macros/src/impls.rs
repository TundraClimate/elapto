use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::fmt::Debug;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitStr, Token, token::Brace};

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
        properties: Vec<(Ident, Expr)>,
        childrens: Vec<Node>,
    },
    FragmentTemplate {
        constant_name: Ident,
        dummy_props: Vec<(Ident, Expr)>,
        childrens: Vec<Node>,
    },
}

struct Inline {
    inner: Expr,
}

struct Text {
    inner: LitStr,
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

    fn properties(&self) -> &Vec<(Ident, Expr)> {
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
}

impl Tag {
    fn widget(
        name: Ident,
        id: Option<Expr>,
        class: Option<Expr>,
        properties: Vec<(Ident, Expr)>,
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

            input.parse::<Token![=]>()?;

            let v: Expr = input.parse()?;

            if k == "id" {
                id = Some(v);

                continue;
            }

            if k == "class" {
                class = Some(v);

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
            syn::braced!(content in input);

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

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tag(tag) => write!(f, "Node::tag, {:?}", tag.childrens()),
            Self::Inline(expr) => write!(f, "Node::inline"),
            Self::Text(text) => write!(f, "Node::text, {:?}", text.inner.value()),
        }
    }
}

pub(crate) fn parse_tag(tokens: TokenStream) -> syn::Result<TokenStream> {
    let node = syn::parse2::<Node>(tokens)?;
    let node = format!("{:?}", node);

    Ok(quote! {
        format!("{}", stringify!(#node))
    })
}
