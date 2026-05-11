use crate::rpstate::model::StoreFieldEntry;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Expr, Ident, Visibility};

pub(crate) fn schema_methods(
    entries: &[StoreFieldEntry],
) -> impl Iterator<Item = TokenStream2> + '_ {
    entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let mname = format_ident!("__schema_field_{}", fname, span = fname.span());
        let ty = &e.ty;
        let wrapper = if e.export_mut {
            quote!(::rpstate::store::shared::Writable)
        } else {
            quote!(::rpstate::store::shared::ReadOnly)
        };
        quote_spanned! { fname.span() =>
            #[doc(hidden)]
            pub fn #mname() -> #wrapper<#ty> { ::std::unreachable!() }
        }
    })
}

pub(crate) fn struct_fields(
    entries: &[StoreFieldEntry],
) -> impl Iterator<Item = TokenStream2> + '_ {
    entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let fvis = &e.vis;
        let ty = &e.ty;
        if e.nested || e.lookup_node.is_some() {
            quote! { #fvis #fname: ::std::sync::Arc<#ty> }
        } else {
            let mode = field_mode(e);
            quote! { #fvis #fname: ::rpstate::Field<#ty, #mode> }
        }
    })
}

pub(crate) fn methods(entries: &[StoreFieldEntry]) -> impl Iterator<Item = TokenStream2> + '_ {
    entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let ty = &e.ty;

        if e.nested || e.lookup_node.is_some() {
            quote! { pub fn #fname(&self) -> ::std::sync::Arc<#ty> { self.#fname.clone() } }
        } else {
            let mode = field_mode(e);
            quote! {
                pub fn #fname(&self) -> ::rpstate::Field<#ty, #mode> {
                    self.#fname.clone()
                }
            }
        }
    })
}

pub(crate) fn node_impl(name: &Ident, is_root: bool) -> TokenStream2 {
    if is_root {
        quote! {
            impl ::rpstate::store::shared::RpStateNode for #name {
                fn new_node(store: &::std::sync::Arc<::rpstate::DefaultStore>, _path: &str) -> ::rpstate::store::Result<Self> {
                    Self::new(store)
                }
            }
        }
    } else {
        quote! {
            impl ::rpstate::store::shared::RpStateNode for #name {
                fn new_node(store: &::std::sync::Arc<::rpstate::DefaultStore>, path: &str) -> ::rpstate::store::Result<Self> {
                    Self::new(store, path)
                }
            }
        }
    }
}

pub(crate) fn scope(name: &Ident, prefix: Option<String>) -> Option<TokenStream2> {
    prefix.map(
        |p| quote! { impl ::rpstate::StateScope for #name { const PREFIX: &'static str = #p; } },
    )
}

pub(crate) fn constructor(is_root: bool, init_fields: &[TokenStream2]) -> TokenStream2 {
    if is_root {
        quote! { pub fn new(store: &::std::sync::Arc<::rpstate::DefaultStore>) -> ::rpstate::store::Result<Self> { Ok(Self { #(#init_fields,)* }) } }
    } else {
        quote! { pub fn new(store: &::std::sync::Arc<::rpstate::DefaultStore>, namespace: &str) -> ::rpstate::store::Result<Self> { Ok(Self { #(#init_fields,)* }) } }
    }
}

pub(crate) fn lookup_chain(
    target: &darling::util::SpannedValue<String>,
    parent: &Expr,
) -> TokenStream2 {
    let target_span = target.span();
    let target_str = target.to_string();
    let parts: Vec<&str> = target_str.split('.').collect();
    let mut chain = quote_spanned!(parent.span()=> #parent);
    for (i, p) in parts.iter().enumerate() {
        let m = format_ident!("__schema_field_{}", p, span = target_span);
        chain = if i == 0 {
            quote_spanned!(target_span=> #chain::#m())
        } else {
            quote_spanned!(target_span=> #chain.#m())
        };
    }
    chain
}

pub(crate) fn field_mode(e: &StoreFieldEntry) -> TokenStream2 {
    if e.lookup.is_some() {
        if e.export_mut {
            quote!(::rpstate::store::shared::WritableMode)
        } else {
            quote!(::rpstate::store::shared::ReadOnlyMode)
        }
    } else {
        quote!(::rpstate::store::shared::WritableMode)
    }
}

#[allow(dead_code)]
pub(crate) fn _keep_visibility_type(_: &Visibility) {}
