use proc_macro2::{Span, TokenStream};
use syn::{Attribute, Expr, ExprLit, Fields, Ident, Lit, LitBool, LitStr, Visibility};

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

struct WidgetInitializer {
    default: bool,
    custom_hash: bool,
}

struct WidgetStruct {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    fields: Fields,
}

pub(crate) fn parse_object(tokens: TokenStream) -> syn::Result<TokenStream> {
    let obj = syn::parse2::<Object>(tokens)?;
    let toks = expand::expand_object(obj);

    Ok(toks)
}

pub(crate) fn parse_widget(attr: TokenStream, tokens: TokenStream) -> syn::Result<TokenStream> {
    let initializer: WidgetInitializer = syn::parse2(attr)?;
    let item: WidgetStruct = syn::parse2(tokens)?;
    let toks = expand::expand_widget_struct(initializer, item);

    Ok(toks)
}
