use darling::FromField;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use rpstate_macros_core::StoreFieldEntry;
use syn::{Data, DataStruct, DeriveInput, Fields, parse_macro_input};

fn rpstate_dioxus_crate_path() -> TokenStream2 {
    match crate_name("rpstate-dioxus") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        _ => quote!(::rpstate_dioxus),
    }
}

#[proc_macro_attribute]
pub fn rpstate_dioxus(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let struct_vis = &input.vis;

    let rpstate_dioxus = rpstate_dioxus_crate_path();

    let named_fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(f),
            ..
        }) => &f.named,
        _ => {
            return syn::Error::new_spanned(struct_name, "rpstate_dioxus only works on structs")
                .to_compile_error()
                .into();
        }
    };

    let mut entries = Vec::new();
    for field in named_fields {
        match StoreFieldEntry::from_field(field) {
            Ok(v) => entries.push(v),
            Err(e) => return e.write_errors().into(),
        }
    }

    let handle_struct_name = format_ident!("{}Handle", struct_name);

    let handle_fields = entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let fvis = &e.vis;
        let ty = &e.ty;

        if e.nested || e.lookup_node.is_some() {
            let nested_handle = format_ident!("{}Handle", get_type_ident_str(ty));
            quote! { #fvis #fname: #nested_handle }
        } else if let Some((k, v)) = e.get_map_types() {
            let mode = if e.lookup.is_some() && !e.export_mut {
                quote!(#rpstate_dioxus::ReadOnlyMode)
            } else {
                quote!(#rpstate_dioxus::WritableMode)
            };
            quote! { #fvis #fname: #rpstate_dioxus::MapHandle<#k, #v, #rpstate_dioxus::DefaultStore, #mode> }
        } else {
            let mode = if e.lookup.is_some() && !e.export_mut {
                quote!(#rpstate_dioxus::ReadOnlyMode)
            } else {
                quote!(#rpstate_dioxus::WritableMode)
            };
            quote! { #fvis #fname: #rpstate_dioxus::FieldHandle<#ty, #rpstate_dioxus::DefaultStore, #mode> }
        }
    });

    let register_fields = entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        if e.nested || e.lookup_node.is_some() {
            quote! { #fname: self.#fname.register_dioxus(arena) }
        } else if e.get_map_types().is_some() {
            quote! { #fname: arena.register_map(self.#fname()) }
        } else {
            quote! { #fname: arena.register_field(self.#fname()) }
        }
    });

    let expanded = quote! {
        #input

        #[doc(hidden)]
        #[derive(Copy, Clone, PartialEq)]
        #struct_vis struct #handle_struct_name {
            #(#handle_fields,)*
        }

        impl #struct_name {
            #struct_vis fn register_dioxus(&self, arena: &#rpstate_dioxus::Arena) -> #handle_struct_name {
                #handle_struct_name {
                    #(#register_fields,)*
                }
            }
        }

        impl #rpstate_dioxus::RpStateDioxusNested for #struct_name {
            type Handle = #handle_struct_name;

            fn register_dioxus(&self, arena: &#rpstate_dioxus::Arena) -> Self::Handle {
                self.register_dioxus(arena)
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_type_ident_str(ty: &syn::Type) -> String {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident.to_string();
    }
    "any".to_string()
}
