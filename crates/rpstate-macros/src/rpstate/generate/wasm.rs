use crate::rpstate::generate::parse_default;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rpstate_macros_core::{MacroArgs, StoreFieldEntry};
use syn::{Ident, Visibility};

pub fn generate_wasm_code(
    crate_name: TokenStream2,
    vis: &Visibility,
    name: &Ident,
    attrs: &[syn::Attribute],
    prefix: Option<String>,
    entries: &[StoreFieldEntry],
    _macro_args: MacroArgs,
) -> TokenStream2 {
    let is_root = prefix.is_some();
    let prefix_str = prefix.unwrap_or_default();

    let backend_ty = quote! { ::rpstate::tauri::TauriBackend };

    let struct_fields = entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let fvis = &e.vis;
        let ty = &e.ty;

        if e.nested || e.lookup_node.is_some() {
            let nested_type = get_type_ident(ty);
            quote! { #fvis #fname: #nested_type }
        } else if let Some((k, v)) = e.get_map_types() {
            quote! { #fvis #fname: #crate_name::client::ReactiveMap<#k, #v, #backend_ty> }
        } else {
            quote! { #fvis #fname: #crate_name::client::Field<#ty, #backend_ty> }
        }
    });

    let methods = entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let ty = &e.ty;

        if e.nested || e.lookup_node.is_some() {
            let nested_type = get_type_ident(ty);
            quote! { pub fn #fname(&self) -> #nested_type { self.#fname.clone() } }
        } else if let Some((k, v)) = e.get_map_types() {
            quote! { pub fn #fname(&self) -> #crate_name::client::ReactiveMap<#k, #v, #backend_ty> { self.#fname.clone() } }
        } else {
            quote! { pub fn #fname(&self) -> #crate_name::client::Field<#ty, #backend_ty> { self.#fname.clone() } }
        }
    });

    let init_fields = entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let key_suffix = if let Some(lookup) = &e.lookup { lookup.to_string() }
        else if let Some(lookup_node) = &e.lookup_node { lookup_node.to_string() }
        else { e.key.as_ref().map(|s| s.as_str()).unwrap_or(&fname.to_string()).to_string() };

        let full_key = if e.lookup.is_some() || e.lookup_node.is_some() { key_suffix }
        else { format!("{}.{}", prefix_str, key_suffix) };

        let ty = &e.ty;
        let fallback = e.default.as_ref().map(parse_default).unwrap_or_else(|| quote! { ::std::default::Default::default() });

        if e.nested || e.lookup_node.is_some() {
            let nested_type = get_type_ident(ty);
            quote! { #fname: #nested_type::new(#full_key, &initial, store) }
        } else if let Some((k, v)) = e.get_map_types() {
            quote! {
                #fname: {
                    let mut map_init = ::std::collections::HashMap::new();
                    let map_prefix = format!("{}.", #full_key);
                    for (k, v) in &initial {
                        if let Some(sub_key) = k.strip_prefix(&map_prefix) {
                            if let Ok(parsed_k) = <#k as ::std::str::FromStr>::from_str(sub_key) {
                                if let Ok(parsed_v) = store.decode::<#v>(v) {
                                    map_init.insert(parsed_k, parsed_v);
                                }
                            }
                        }
                    }
                    #crate_name::client::ReactiveMap::new_with_backend(#full_key, map_init, store.clone())
                }
            }
        } else {
            quote! {
                #fname: {
                    let val = initial.get(#full_key)
                        .and_then(|v| store.decode::<#ty>(v).ok())
                        .unwrap_or_else(|| #fallback);
                    #crate_name::client::Field::new_with_backend(#full_key, val, store.clone())
                }
            }
        }
    });

    let load_impl = if is_root {
        quote! {
            impl #crate_name::client::RpStateSliceAsync<#backend_ty> for #name {
                type Error = <#backend_ty as #crate_name::RpBackendAsync>::Error;

                async fn load_async(store: &#backend_ty) -> ::std::result::Result<Self, Self::Error> {
                    use #crate_name::RpBackendAsync;
                    let raw_entries = store.scan_prefix(#prefix_str).await?;
                    let mut initial = ::std::collections::HashMap::new();
                    for (k, v) in raw_entries {
                        initial.insert(k, v);
                    }

                    Ok(Self {
                        #(#init_fields,)*
                    })
                }
            }
        }
    } else {
        let nested_init_fields = entries.iter().map(|e| {
            let fname = e.ident.as_ref().unwrap();
            let key_str = e.key.as_ref().map(|s| s.as_str()).unwrap_or(&fname.to_string()).to_string();
            let ty = &e.ty;
            let fallback = e.default.as_ref().map(parse_default).unwrap_or_else(|| quote! { ::std::default::Default::default() });

            if e.nested || e.lookup_node.is_some() {
                let nested_type = get_type_ident(ty);
                quote! { #fname: #nested_type::new(&format!("{}.{}", prefix, #key_str), initial, store) }
            } else if let Some((k, v)) = e.get_map_types() {
                quote! {
                    #fname: {
                        let mut map_init = ::std::collections::HashMap::new();
                        let map_prefix = format!("{}.", prefix, #key_str);
                        for (k, v) in initial {
                            if let Some(sub_key) = k.strip_prefix(&map_prefix) {
                                if let Ok(parsed_k) = <#k as ::std::str::FromStr>::from_str(sub_key) {
                                    if let Ok(parsed_v) = store.decode::<#v>(v) {
                                        map_init.insert(parsed_k, parsed_v);
                                    }
                                }
                            }
                        }
                        #crate_name::client::ReactiveMap::new_with_backend(format!("{}.{}", prefix, #key_str), map_init, store.clone())
                    }
                }
            } else {
                quote! {
                    #fname: {
                        let full_key = format!("{}.{}", prefix, #key_str);
                        let val = initial.get(&full_key)
                            .and_then(|v| store.decode::<#ty>(v).ok())
                            .unwrap_or_else(|| #fallback);
                        #crate_name::client::Field::new_with_backend(full_key, val, store.clone())
                    }
                }
            }
        });

        quote! {
            impl #name {
                pub fn new(prefix: &str, initial: &::std::collections::HashMap<String, <#backend_ty as #crate_name::client::RpBackendAsync>::Raw>, store: &#backend_ty) -> Self {
                    Self { #(#nested_init_fields,)* }
                }
            }
        }
    };

    quote! {
        #[derive(Clone, Debug)]
        #(#attrs)* #vis struct #name {
            #(#struct_fields,)*
        }

        #load_impl

        impl #name {
            #(#methods)*
        }
    }
}

fn get_type_ident(ty: &syn::Type) -> proc_macro2::TokenStream {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = &segment.ident;
            return quote! { #ident };
        }
    }
    quote! { #ty }
}
