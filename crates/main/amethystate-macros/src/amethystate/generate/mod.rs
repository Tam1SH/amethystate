mod accessors;
mod data;
mod init;
mod wasm;

use amethystate_macros_core::{MacroArgs, StoreFieldEntry, get_type_ident_str};
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

pub fn generate_code(
    crate_name: TokenStream2,
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
                "invalid amethystate mode: \"{}\". Expected one of: \"reactive\", \"persistent\", \"both\"",
                other
            );
            return syn::Error::new(proc_macro2::Span::call_site(), err).to_compile_error();
        }
    };

    if macro_args.target.as_deref() == Some("tauri-wasm") {
        return wasm::generate_wasm_code(crate_name, vis, name, attrs, prefix, entries, macro_args);
    }

    let is_root = prefix.is_some();
    let schema_methods = accessors::schema_methods(&crate_name, entries);
    let fields_impl = data::data_impl(
        &crate_name,
        vis,
        name,
        attrs,
        prefix.clone(),
        entries,
        &macro_args,
        rp_mode,
    );

    let struct_fields = accessors::struct_fields(&crate_name, entries);
    let init_fields = init::init_fields(&crate_name, entries, is_root);
    let node_impl = accessors::node_impl(&crate_name, name, is_root);
    let methods = accessors::methods(&crate_name, entries);
    let scope = accessors::scope(&crate_name, name, prefix.clone());
    let constructor = accessors::constructor(&crate_name, is_root, &init_fields);

    let schema_export = generate_schema_export(&crate_name, name, &prefix, entries);

    let inherent_subs = if matches!(rp_mode, RpMode::Reactive | RpMode::Both) {
        let sub_all_fields = entries.iter().map(|e| {
            let fname = e.ident.as_ref().unwrap();
            if e.nested || e.lookup_node.is_some() {
                quote! {
                    {
                        let cb_clone = cb.clone();
                        scope.watch_scope(self.#fname.subscribe_all(move || cb_clone()));
                    }
                }
            } else if e.get_map_types().is_some() {
                quote! {
                    {
                        let cb_clone = cb.clone();
                        scope.watch(self.#fname.subscribe_any(move |_| cb_clone()));
                    }
                }
            } else {
                quote! {
                    {
                        let cb_clone = cb.clone();
                        scope.watch(self.#fname.subscribe(move |_| cb_clone()));
                    }
                }
            }
        });

        let sub_all_fields_ext = entries.iter().map(|e| {
            let fname = e.ident.as_ref().unwrap();
            if e.nested || e.lookup_node.is_some() {
                quote! {
                    {
                        let cb_clone = cb.clone();
                        scope.watch_scope(self.#fname.subscribe_all_external(move || cb_clone()));
                    }
                }
            } else if e.get_map_types().is_some() {
                quote! {
                    {
                        let cb_clone = cb.clone();
                        scope.watch(self.#fname.subscribe_any_external(move |_| cb_clone()));
                    }
                }
            } else {
                quote! {
                    {
                        let cb_clone = cb.clone();
                        scope.watch(self.#fname.subscribe_external(move |_| cb_clone()));
                    }
                }
            }
        });

        quote! {
            pub fn subscribe_all<F>(&self, callback: F) -> #crate_name::ReactiveScope
            where
                F: Fn() + Send + Sync + 'static,
            {
                let cb = ::std::sync::Arc::new(callback);
                let mut scope = #crate_name::ReactiveScope::new();

                #(#sub_all_fields)*

                scope
            }

            pub fn subscribe_all_external<F>(&self, callback: F) -> #crate_name::ReactiveScope
            where
                F: Fn() + Send + Sync + 'static,
            {
                let cb = ::std::sync::Arc::new(callback);
                let mut scope = #crate_name::ReactiveScope::new();

                #(#sub_all_fields_ext)*

                scope
            }
        }
    } else {
        quote! {}
    };

    let slice_impl = if is_root {
        let load_fn = match rp_mode {
            RpMode::Persistent => quote! { Self::load_with(store) },
            _ => quote! { Self::new_with(store) },
        };

        let slice_subs = if matches!(rp_mode, RpMode::Reactive | RpMode::Both) {
            quote! {
                fn subscribe_all<F>(&self, callback: F) -> #crate_name::ReactiveScope
                where
                    F: Fn() + Send + Sync + 'static,
                {
                    self.subscribe_all(callback)
                }

                fn subscribe_all_external<F>(&self, callback: F) -> #crate_name::ReactiveScope
                where
                    F: Fn() + Send + Sync + 'static,
                {
                    self.subscribe_all_external(callback)
                }
            }
        } else {
            quote! {
                fn subscribe_all<F>(&self, _callback: F) -> #crate_name::ReactiveScope
                where
                    F: Fn() + Send + Sync + 'static,
                {
                    #crate_name::ReactiveScope::new()
                }

                fn subscribe_all_external<F>(&self, _callback: F) -> #crate_name::ReactiveScope
                where
                    F: Fn() + Send + Sync + 'static,
                {
                    #crate_name::ReactiveScope::new()
                }
            }
        };

        quote! {
            impl<S: #crate_name::Store> #crate_name::AmeStateSlice<S> for #name<S> {
                fn load_slice(store: &S) -> #crate_name::Result<Self> {
                    #load_fn
                }

                #slice_subs
            }
        }
    } else {
        quote! {}
    };

    let global_new_impl = if is_root {
        quote! {
            impl #name<#crate_name::DefaultStore> {
                pub fn new() -> #crate_name::Result<Self> {
                    let store = #crate_name::global_store();
                    Self::new_with(&store)
                }
            }
        }
    } else {
        quote! {}
    };

    let fork_fields = entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        if e.nested || e.lookup_node.is_some() {
            quote! { #fname: ::std::sync::Arc::new(self.#fname.fork_with_id(new_id)) }
        } else {
            quote! { #fname: self.#fname.fork_with_id(new_id) }
        }
    });

    let fork_impl = if matches!(rp_mode, RpMode::Reactive | RpMode::Both) {
        quote! {
            pub fn fork(&self) -> Self {
                self.fork_with_id(#crate_name::uuid::Uuid::new_v4())
            }

            #[doc(hidden)]
            pub fn fork_with_id(&self, new_id: #crate_name::uuid::Uuid) -> Self {
                Self {
                    __amethystate_instance_id: new_id,
                    #(#fork_fields,)*
                }
            }
        }
    } else {
        quote! {}
    };

    match rp_mode {
        RpMode::Reactive | RpMode::Both => {
            quote! {
                #[derive(Clone)]
                #(#attrs)* #vis struct #name<S: #crate_name::Store = #crate_name::DefaultStore> {
                    __amethystate_instance_id: #crate_name::uuid::Uuid,
                    #(#struct_fields,)*
                }
                #scope
                impl<S: #crate_name::Store> #name<S> {
                    #constructor #(#schema_methods)* #(#methods)*
                    #fork_impl
                    #inherent_subs
                }
                #global_new_impl
                #node_impl
                #fields_impl
                #schema_export
                #slice_impl
            }
        }
        RpMode::Persistent => {
            quote! {
                #scope
                #fields_impl
                #schema_export
                #slice_impl
            }
        }
    }
}

fn generate_schema_export(
    crate_name: &TokenStream2,
    name: &Ident,
    prefix: &Option<String>,
    entries: &[StoreFieldEntry],
) -> TokenStream2 {
    let struct_name_str = name.to_string();
    let prefix_tokens = match prefix {
        Some(p) => quote! { Some(#p) },
        None => quote! { None },
    };

    let field_metas = entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let fname_str = fname.to_string();
        let (ts_type, full_ts_type) = map_type_to_ts(e.ty.clone());

        let ty = &e.ty;
        let rust_type_str = quote!(#ty).to_string();

        let kind_tokens = if e.volatile {
            quote! { #crate_name::tauri::FieldKind::Volatile }
        } else if e.nested {
            let sname = get_type_ident_str(&e.ty);
            quote! { #crate_name::tauri::FieldKind::Nested { struct_name: #sname } }
        } else if let Some(target) = &e.lookup {
            let target_str = target.to_string();
            let mutable = e.export_mut;
            quote! { #crate_name::tauri::FieldKind::Lookup { target_key: #target_str, mutable: #mutable } }
        } else if let Some(target) = &e.lookup_node {
            let target_str = target.to_string();
            let sname = get_type_ident_str(&e.ty);
            quote! { #crate_name::tauri::FieldKind::LookupNode { target_prefix: #target_str, struct_name: #sname } }
        } else if let Some((k, v)) = e.get_map_types() {
            let k_ts = map_type_to_ts(k.clone()).1;
            let v_ts = map_type_to_ts(v.clone()).1;
            let k_rust = quote!(#k).to_string();
            let v_rust = quote!(#v).to_string();
            quote! {
                #crate_name::tauri::FieldKind::ReactiveMap {
                    key_type: #k_ts,
                    value_type: #v_ts,
                    key_rust_type: #k_rust,
                    value_rust_type: #v_rust,
                }
            }
        } else {
            quote! { #crate_name::tauri::FieldKind::Plain }
        };

        quote! {
            #crate_name::tauri::FieldExportMeta {
                name: #fname_str,
                ts_type: #ts_type,
                full_ts_type: #full_ts_type,
                rust_type: #rust_type_str,
                kind: #kind_tokens,
            }
        }
    });

    if cfg!(feature = "tauri") {
        quote! {
            #[cfg(not(target_arch = "wasm32"))]
            #crate_name::inventory::submit! {
                #crate_name::tauri::SchemaExportEntry {
                    prefix: #prefix_tokens,
                    struct_name: #struct_name_str,
                    fields: &[
                        #(#field_metas),*
                    ],
                }
            }
        }
    } else {
        quote!()
    }
}

fn map_type_to_ts(ty: syn::Type) -> (String, String) {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let ident_str = segment.ident.to_string();
                match ident_str.as_str() {
                    "String" => ("string".to_string(), "string".to_string()),
                    "bool" => ("boolean".to_string(), "boolean".to_string()),
                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32"
                    | "i64" | "i128" | "isize" | "f32" | "f64" => {
                        ("number".to_string(), "number".to_string())
                    }
                    "Vec" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                        {
                            let (inner_base, inner_full) = map_type_to_ts(inner_ty.clone());
                            return (inner_base, format!("{}[]", inner_full));
                        }
                        ("any".to_string(), "any[]".to_string())
                    }
                    "Option" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                        {
                            let (inner_base, inner_full) = map_type_to_ts(inner_ty.clone());
                            return (inner_base, format!("{} | null", inner_full));
                        }
                        ("any".to_string(), "any | null".to_string())
                    }
                    other => (other.to_string(), other.to_string()),
                }
            } else {
                ("any".to_string(), "any".to_string())
            }
        }
        _ => ("any".to_string(), "any".to_string()),
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
