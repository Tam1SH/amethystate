mod accessors;
mod data;
mod init;

use crate::rpstate::model::{MacroArgs, StoreFieldEntry};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Ident, Visibility};

pub(crate) fn generate_code(
    vis: &Visibility,
    name: &Ident,
    attrs: &[Attribute],
    prefix: Option<String>,
    entries: &[StoreFieldEntry],
    macro_args: MacroArgs,
) -> TokenStream2 {
    let is_root = prefix.is_some();
    let schema_methods = accessors::schema_methods(entries);
    let fields_impl = data::data_impl(name, prefix.clone(), entries, &macro_args);
    let migrations_registry = data::migrations_registry(name, entries, &macro_args);
    let struct_fields = accessors::struct_fields(entries);
    let init_fields = init::init_fields(entries, is_root);
    let node_impl = accessors::node_impl(name, is_root);
    let methods = accessors::methods(entries);
    let scope = accessors::scope(name, prefix);
    let constructor = accessors::constructor(is_root, &init_fields);

    quote! {
        #[derive(Clone)] #(#attrs)* #vis struct #name { #(#struct_fields,)* }
        #scope
        impl #name { #constructor #(#schema_methods)* #(#methods)* }
        #node_impl
        #fields_impl
        #migrations_registry
    }
}
