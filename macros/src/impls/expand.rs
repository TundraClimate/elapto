use crate::impls::{
    Fragment, Inline, Named, Object, Property, Tag, Text, WidgetInitializer, WidgetStruct,
};

use proc_macro_crate::FoundCrate;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, ExprLit, Ident, Lit};

fn crate_ident() -> TokenStream {
    match proc_macro_crate::crate_name("elapto").unwrap() {
        FoundCrate::Itself => quote! { crate },
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
    }
}

trait Expand {
    fn expand(self) -> TokenStream;
}

impl Expand for Named {
    fn expand(self) -> TokenStream {
        let crate_ident = crate_ident();

        let name = &self.name;
        let props = self
            .properties
            .into_iter()
            .map(|(key, value)| {
                let expr = value.expand();

                quote! { p.#key = #expr; }
            })
            .collect::<Vec<_>>();
        let (id_ident, id) = (self.id_ident, self.id).clone();
        let (class_ident, class) = (self.class_ident, self.class).clone();

        let id = id
            .map(|id| quote! { .with_id(#crate_ident::Identifier { #id_ident: #id.to_string() }) });
        let class =
            class.map(|class| quote! { .with_class(#crate_ident::Class { #class_ident: #class.to_string() }) });

        let children = self
            .children
            .0
            .into_iter()
            .map(expand_object)
            .collect::<Vec<_>>();
        let children = quote! { vec![ #(#children),* ] };

        let fragment_patch = (name == "Fragment").then_some(quote! {
            p.children = #children;
        });

        let children = fragment_patch
            .is_none()
            .then_some(quote! { .with_children(#children) });

        quote! {
            #crate_ident::Component::new({
                use #crate_ident::Fragment;

                let mut p = #name::default();

                #fragment_patch
                #(#props)*

                p
            })
            .with_sub_props(|p| {
                p
                    #id
                    #class
                    #children
            })
        }
    }
}

impl Expand for Fragment {
    fn expand(self) -> TokenStream {
        let crate_ident = crate_ident();

        let children = self.0.0.into_iter().map(expand_object).collect::<Vec<_>>();

        quote! { #crate_ident::Component::new(#crate_ident::Fragment::new(vec![ #(#children),* ])) }
    }
}

impl Expand for Inline {
    fn expand(self) -> TokenStream {
        let crate_ident = crate_ident();

        let expr = self.0;

        quote! { #crate_ident::Expand::expand({ #expr }) }
    }
}

impl Expand for Text {
    fn expand(self) -> TokenStream {
        let crate_ident = crate_ident();

        let text = self.0;

        quote! { #crate_ident::Component::new(#crate_ident::Text::new(#text)) }
    }
}

impl Expand for Property {
    fn expand(self) -> TokenStream {
        match self {
            Self::Text(text) => quote! { #text.to_string() },
            Self::Bool => quote! { true },
            Self::Expr(Expr::Lit(ExprLit {
                lit: Lit::Str(lit), ..
            })) => quote! { #lit.to_string() },
            Self::Expr(expr) => quote! { { #expr } },
        }
    }
}

pub(crate) fn expand_object(obj: Object) -> TokenStream {
    match obj {
        Object::Tag(Tag::Named(named)) => named.expand(),
        Object::Tag(Tag::Fragment(fragm)) => fragm.expand(),
        Object::Inline(expr) => expr.expand(),
        Object::Text(text) => text.expand(),
    }
}

impl Expand for WidgetInitializer {
    fn expand(self) -> TokenStream {
        let derive_default = self.default;
        let derive_hash = !self.custom_hash;

        let default = derive_default.then_some(quote! { Default });
        let custom_hash = derive_hash.then_some(quote! { Hash });

        let comma = (derive_default && derive_hash).then_some(quote! { , });

        quote! {
            #[derive(#default #comma #custom_hash)]
        }
    }
}

impl Expand for WidgetStruct {
    fn expand(self) -> TokenStream {
        let crate_ident = crate_ident();

        let attrs = self.attrs;
        let name = self.name;
        let vis = self.vis;
        let fields = self.fields;

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

        quote! {
            #(#attrs)*
            #vis struct #name {
                #(#expand_fields)*
            }

            impl #crate_ident::WidgetInfo for #name {
                fn type_name(&self) -> &'static str {
                    ::std::any::type_name::<Self>()
                }

                fn properties(&self) -> Vec<(&str, String)> {
                    vec![ #(#props)* ]
                }

                fn gen_hash(&self) -> #crate_ident::HashCell {
                    #crate_ident::HashCell::new(&self)
                }
            }
        }
    }
}

pub(crate) fn expand_widget_struct(init: WidgetInitializer, wd: WidgetStruct) -> TokenStream {
    let init = init.expand();
    let item = wd.expand();

    quote! {
        #[allow(missing_docs)]
        #init
        #item
    }
}
