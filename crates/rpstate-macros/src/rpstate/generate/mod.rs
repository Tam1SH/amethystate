mod accessors;
mod data;
mod init;

use crate::rpstate::model::{MacroArgs, StoreFieldEntry};
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, Ident, Token, Visibility};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RpMode {
    Reactive,
    Persistent,
    Both,
}

pub(crate) fn generate_code(
    vis: &Visibility,
    name: &Ident,
    attrs: &[Attribute],
    prefix: Option<String>,
    entries: &[StoreFieldEntry],
    macro_args: MacroArgs,
) -> TokenStream2 {
    let rp_mode = match macro_args.mode.as_deref() {
        None | Some("reactive") => RpMode::Reactive,
        Some("persistent") => RpMode::Persistent,
        Some("both") => RpMode::Both,
        Some(other) => {
            let err = format!(
                "invalid rpstate mode: \"{}\". Expected one of: \"reactive\", \"persistent\", \"both\"",
                other
            );
            return syn::Error::new(proc_macro2::Span::call_site(), err).to_compile_error();
        }
    };

    let is_root = prefix.is_some();
    let schema_methods = accessors::schema_methods(entries);
    let fields_impl = data::data_impl(
        vis,
        name,
        attrs,
        prefix.clone(),
        entries,
        &macro_args,
        rp_mode,
    );
    let struct_fields = accessors::struct_fields(entries);
    let init_fields = init::init_fields(entries, is_root);
    let node_impl = accessors::node_impl(name, is_root);
    let methods = accessors::methods(entries);
    let scope = accessors::scope(name, prefix);
    let constructor = accessors::constructor(is_root, &init_fields);

    match rp_mode {
        RpMode::Reactive => {
            quote! {
                #[derive(Clone)] #(#attrs)* #vis struct #name { #(#struct_fields,)* }
                #scope
                impl #name { #constructor #(#schema_methods)* #(#methods)* }
                #node_impl
                #fields_impl
            }
        }
        RpMode::Persistent => {
            quote! {
                #scope
                #fields_impl
            }
        }
        RpMode::Both => {
            quote! {
                #[derive(Clone)] #(#attrs)* #vis struct #name { #(#struct_fields,)* }
                #scope
                impl #name { #constructor #(#schema_methods)* #(#methods)* }
                #node_impl
                #fields_impl
            }
        }
    }
}

struct MapEntry {
    key: Expr,
    _colon: Token![:],
    value: Expr,
}

impl Parse for MapEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MapEntry {
            key: input.parse()?,
            _colon: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub(crate) fn parse_default(tokens: &TokenStream2) -> TokenStream2 {
    let mut iter = tokens.clone().into_iter();

    match iter.next() {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
            let content = g.stream();
            quote! { vec![#content] }
        }
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
            let content = g.stream();

            if content.is_empty() {
                return quote! { ::std::collections::HashMap::default() };
            }

            let parser = Punctuated::<MapEntry, Token![,]>::parse_terminated;
            if let Ok(pairs) = parser.parse2(content)
                && !pairs.is_empty()
            {
                let inserts = pairs.iter().map(|pair| {
                    let k = &pair.key;
                    let v = &pair.value;
                    quote! { __map.insert(::std::convert::Into::into(#k), #v); }
                });

                return quote! {
                    {
                        let mut __map = ::std::collections::HashMap::default();
                        #( #inserts )*
                        __map
                    }
                };
            }

            tokens.clone()
        }
        _ => tokens.clone(),
    }
}
