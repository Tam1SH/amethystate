mod generate;
mod model;

use darling::{FromField, FromMeta, ast::NestedMeta};
use generate::generate_code;
use model::{MacroArgs, StoreFieldEntry};
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;
use syn::__private::TokenStream2;
use syn::{Data, DataStruct, DeriveInput, Fields, parse_macro_input};

pub fn rpstate_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => return darling::Error::from(e).write_errors().into(),
    };

    let macro_args = match MacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let struct_vis = &input.vis;
    let attrs = &input.attrs;
    let rpstate = rpstate_crate_path();

    let named_fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(f),
            ..
        }) => &f.named,
        _ => {
            return darling::Error::custom("rpstate can only be used on structs with named fields")
                .with_span(struct_name)
                .write_errors()
                .into();
        }
    };

    let mut entries = Vec::new();
    for field in named_fields {
        let entry = match StoreFieldEntry::from_field(field) {
            Ok(v) => v,
            Err(e) => return e.write_errors().into(),
        };

        entries.push(entry);
    }

    let expanded = generate_code(
        rpstate,
        struct_vis,
        struct_name,
        attrs,
        macro_args.prefix.clone(),
        &entries,
        macro_args,
    );

    proc_macro::TokenStream::from(expanded)
}

pub fn rpstate_crate_path() -> TokenStream2 {
    match crate_name("rpstate") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        _ => quote!(::rpstate),
    }
}
