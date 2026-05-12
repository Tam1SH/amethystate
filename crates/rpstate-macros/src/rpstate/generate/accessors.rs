use crate::rpstate::model::StoreFieldEntry;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::{Ident, Visibility};

pub(crate) fn schema_methods(
    entries: &[StoreFieldEntry],
) -> impl Iterator<Item = TokenStream2> + '_ {
    entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let mname = format_ident!("__schema_field_{}", fname, span = fname.span());
        let ty = &e.ty;
        let wrapper = if e.export_mut {
            quote!(::rpstate::Writable)
        } else {
            quote!(::rpstate::ReadOnly)
        };
        quote_spanned! { fname.span() =>
            #[doc(hidden)]
            pub fn #mname(&self) -> #wrapper<#ty> { ::std::unreachable!() }
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
            quote! { #fvis #fname: ::rpstate::Field<#ty, ::rpstate::DefaultStore, #mode> }
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
                pub fn #fname(&self) -> ::rpstate::Field<#ty, ::rpstate::DefaultStore, #mode> {
                    self.#fname.clone()
                }
            }
        }
    })
}

pub(crate) fn node_impl(name: &Ident, is_root: bool) -> TokenStream2 {
    if is_root {
        quote! {
            impl ::rpstate::RpStateNode for #name {
                fn new_node(store: &::std::sync::Arc<::rpstate::DefaultStore>, _path: &str) -> ::rpstate::Result<Self> {
                    Self::new(store)
                }
            }
        }
    } else {
        quote! {
            impl ::rpstate::RpStateNode for #name {
                fn new_node(store: &::std::sync::Arc<::rpstate::DefaultStore>, path: &str) -> ::rpstate::Result<Self> {
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
        quote! { pub fn new(store: &::std::sync::Arc<::rpstate::DefaultStore>) -> ::rpstate::Result<Self> { Ok(Self { #(#init_fields,)* }) } }
    } else {
        quote! { pub fn new(store: &::std::sync::Arc<::rpstate::DefaultStore>, namespace: &str) -> ::rpstate::Result<Self> { Ok(Self { #(#init_fields,)* }) } }
    }
}

pub(crate) fn lookup_chain(
    target: &darling::util::SpannedValue<String>,
    parent: &syn::Expr,
) -> TokenStream2 {
    let target_str = target.to_string();
    let parts: Vec<&str> = target_str.split('.').collect();

    let mut chain = quote! { unsafe { (&*::core::ptr::null::<#parent>()) } };

    for p in parts {
        let m = format_ident!("__schema_field_{}", p);
        chain = quote! { #chain.#m() };
    }
    chain
}

pub(crate) fn field_mode(e: &StoreFieldEntry) -> TokenStream2 {
    if e.lookup.is_some() {
        if e.export_mut {
            quote!(::rpstate::WritableMode)
        } else {
            quote!(::rpstate::ReadOnlyMode)
        }
    } else {
        quote!(::rpstate::WritableMode)
    }
}

#[allow(dead_code)]
pub(crate) fn _keep_visibility_type(_: &Visibility) {}
