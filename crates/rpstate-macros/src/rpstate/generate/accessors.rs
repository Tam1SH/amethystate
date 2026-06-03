use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use rpstate_macros_core::StoreFieldEntry;
use syn::{Ident, Visibility};

pub(crate) fn schema_methods<'a>(
    crate_name: &'a TokenStream2,
    entries: &'a [StoreFieldEntry],
) -> impl Iterator<Item = TokenStream2> + 'a {
    entries.iter().map(move |e| {
        let fname = e.ident.as_ref().unwrap();
        let mname = format_ident!("__schema_field_{}", fname, span = fname.span());
        let ty = &e.ty;
        let wrapper = if e.export_mut {
            quote!(#crate_name::Writable)
        } else {
            quote!(#crate_name::ReadOnly)
        };
        quote_spanned! { fname.span() =>
            #[doc(hidden)]
            pub fn #mname(&self) -> #wrapper<#ty> { ::std::unreachable!() }
        }
    })
}

pub(crate) fn struct_fields<'a>(
    crate_name: &'a TokenStream2,
    entries: &'a [StoreFieldEntry],
) -> impl Iterator<Item = TokenStream2> + 'a {
    entries.iter().map(move |e| {
        let fname = e.ident.as_ref().unwrap();
        let fvis = &e.vis;
        let ty = &e.ty;

        if e.nested || e.lookup_node.is_some() {
            quote! { #fvis #fname: ::std::sync::Arc<#ty> }
        } else if let Some((k, v)) = e.get_map_types() {
            let mode = field_mode(crate_name, e);
            quote! { #fvis #fname: #crate_name::ReactiveMap<#k, #v, #crate_name::DefaultStore, #mode> }
        } else {
            let mode = field_mode(crate_name, e);
            quote! { #fvis #fname: #crate_name::Field<#ty, #crate_name::DefaultStore, #mode> }
        }
    })
}

pub(crate) fn methods<'a>(
    crate_name: &'a TokenStream2,
    entries: &'a [StoreFieldEntry],
) -> impl Iterator<Item = TokenStream2> + 'a {
    entries.iter().map(move |e| {
        let fname = e.ident.as_ref().unwrap();
        let ty = &e.ty;

        if e.nested || e.lookup_node.is_some() {
            quote! { pub fn #fname(&self) -> ::std::sync::Arc<#ty> { self.#fname.clone() } }
        } else if let Some((k, v)) = e.get_map_types() {
            let mode = field_mode(crate_name, e);
            quote! {
                pub fn #fname(&self) -> #crate_name::ReactiveMap<#k, #v, #crate_name::DefaultStore, #mode> {
                    self.#fname.clone()
                }
            }
        } else {
            let mode = field_mode(crate_name, e);
            quote! {
                pub fn #fname(&self) -> #crate_name::Field<#ty, #crate_name::DefaultStore, #mode> {
                    self.#fname.clone()
                }
            }
        }
    })
}

pub(crate) fn node_impl(crate_name: &TokenStream2, name: &Ident, is_root: bool) -> TokenStream2 {
    if is_root {
        quote! {
            impl #crate_name::RpStateNode for #name {
                fn new_node(store: &::std::sync::Arc<#crate_name::DefaultStore>, _path: &str) -> #crate_name::Result<Self> {
                    Self::new(store)
                }
            }
        }
    } else {
        quote! {
            impl #crate_name::RpStateNode for #name {
                fn new_node(store: &::std::sync::Arc<#crate_name::DefaultStore>, path: &str) -> #crate_name::Result<Self> {
                    Self::new(store, path)
                }
            }
        }
    }
}

pub(crate) fn scope(
    crate_name: &TokenStream2,
    name: &Ident,
    prefix: Option<String>,
) -> Option<TokenStream2> {
    prefix.map(
        |p| quote! { impl #crate_name::StateScope for #name { const PREFIX: &'static str = #p; } },
    )
}

pub(crate) fn constructor(
    crate_name: &TokenStream2,
    is_root: bool,
    init_fields: &[TokenStream2],
) -> TokenStream2 {
    if is_root {
        quote! {
            pub fn new(store: &::std::sync::Arc<#crate_name::DefaultStore>) -> #crate_name::Result<Self> {
                use #crate_name::Store;
                let result = Self { #(#init_fields,)* };
                store.mark_initialized(<Self as #crate_name::StateScope>::PREFIX)?;
                Ok(result)
            }
        }
    } else {
        quote! {
            pub fn new(store: &::std::sync::Arc<#crate_name::DefaultStore>, namespace: &str) -> #crate_name::Result<Self> {
                use #crate_name::Store;
                let result = Self { #(#init_fields,)* };
                store.mark_initialized(namespace)?;
                Ok(result)
            }
        }
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

pub(crate) fn field_mode(crate_name: &TokenStream2, e: &StoreFieldEntry) -> TokenStream2 {
    if e.lookup.is_some() {
        if e.export_mut {
            quote!(#crate_name::WritableMode)
        } else {
            quote!(#crate_name::ReadOnlyMode)
        }
    } else {
        quote!(#crate_name::WritableMode)
    }
}

#[allow(dead_code)]
pub(crate) fn _keep_visibility_type(_: &Visibility) {}
