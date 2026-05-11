use crate::rpstate::model::{MacroArgs, StoreFieldEntry};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::Ident;

pub(crate) fn persistent_fields(entries: &[StoreFieldEntry]) -> Vec<&StoreFieldEntry> {
    entries
        .iter()
        .filter(|e| e.lookup.is_none() && e.lookup_node.is_none() && !e.volatile)
        .collect()
}

pub(crate) fn data_impl(
    name: &Ident,
    prefix: Option<String>,
    entries: &[StoreFieldEntry],
    macro_args: &MacroArgs,
) -> TokenStream2 {
    let p_fields = persistent_fields(entries);
    let data_struct_name = format_ident!("{}_Data", name);

    let data_fields = p_fields.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let ty = &e.ty;
        if e.nested {
            quote! { pub #fname: <#ty as ::rpstate::store::node::RpState>::Data }
        } else {
            quote! { pub #fname: #ty }
        }
    });

    let version_val = macro_args.version.unwrap_or(0);

    let field_descriptors = p_fields.iter().map(|e| {
        let name = e.ident.as_ref().unwrap().to_string();
        let ty = &e.ty;
        if e.nested {
            quote! {
                ::rpstate::store::migration::fields::FieldDescriptor {
                    name: #name,
                    type_hash: 0xDEADBEEF ^ < <#ty as ::rpstate::store::node::RpState>::Data as ::rpstate::store::migration::types::RpType>::TYPE_HASH,
                }
            }
        } else {
            quote! {
                ::rpstate::store::migration::fields::FieldDescriptor {
                    name: #name,
                    type_hash: <#ty as ::rpstate::store::migration::types::RpType>::TYPE_HASH,
                }
            }
        }
    });

    let load_fields = p_fields.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let key = e.key.clone().unwrap_or_else(|| fname.to_string());
        let ty = &e.ty;

        if e.nested {
            quote! {
                #fname: {
                    let mut sub_ctx = ctx.scoped(#key);
                    < <#ty as ::rpstate::store::node::RpState>::Data as ::rpstate::store::migration::fields::RpStateFields>::load_struct(&mut sub_ctx)?
                }
            }
        } else {
            let fallback = e.default.as_ref()
                .map(|d| quote! { #d })
                .unwrap_or_else(|| quote! { <#ty as ::std::default::Default>::default() });
            quote! {
                #fname: ctx.get::<#ty>(#key)?.unwrap_or_else(|| #fallback)
            }
        }
    });

    let save_fields = p_fields.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let key = e.key.clone().unwrap_or_else(|| fname.to_string());

        if e.nested {
            quote! {
                {
                    let mut sub_ctx = ctx.scoped(#key);
                    self.#fname.save_struct(&mut sub_ctx)?;
                }
            }
        } else {
            quote! { ctx.set(#key, &self.#fname)?; }
        }
    });

    let deps = migration_deps(entries);
    let prefix_expr = prefix.unwrap_or_default();

    quote! {
        #[derive(::rpstate::serde::Serialize, ::rpstate::serde::Deserialize, Default, Clone, Debug)]
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub struct #data_struct_name {
            #(#data_fields,)*
        }

        impl ::rpstate::store::migration::types::RpType for #data_struct_name {
            const TYPE_HASH: u64 = ::rpstate::store::migration::types::fnv1a(stringify!(#data_struct_name).as_bytes());
        }

        impl ::rpstate::store::migration::fields::RpStateFields for #data_struct_name {
            const FIELDS: &'static [::rpstate::store::migration::fields::FieldDescriptor] = &[
                #(#field_descriptors),*
            ];
            const VERSION: u32 = #version_val;
            const PARENT_PREFIX: &'static str = #prefix_expr;
            const MIGRATION_DEPS: &'static [&'static str] = &[ #(#deps),* ];

            fn load_struct(ctx: &mut ::rpstate::store::migration::MigrationContext) -> ::rpstate::store::Result<Self> {
                Ok(Self { #(#load_fields,)* })
            }

            fn save_struct(&self, ctx: &mut ::rpstate::store::migration::MigrationContext) -> ::rpstate::store::Result<()> {
                #(#save_fields)*
                Ok(())
            }
        }

        impl ::rpstate::store::node::RpState for #name {
            type Data = #data_struct_name;
        }
    }
}

pub(crate) fn migrations_registry(
    name: &Ident,
    entries: &[StoreFieldEntry],
    macro_args: &MacroArgs,
) -> TokenStream2 {
    if !macro_args.migrations {
        return quote!();
    }

    let deps = migration_deps(entries);
    quote! {
        impl ::rpstate::store::migration::registry::HasMigrations for #name {
            const MIGRATION_DEPS: &'static [&'static str] = &[ #(#deps),* ];
            fn migrations() -> ::rpstate::store::migration::Migrator {
                build_migrations()
            }
        }
        ::rpstate::register_migrations!(#name);
    }
}

fn migration_deps(entries: &[StoreFieldEntry]) -> Vec<TokenStream2> {
    entries
        .iter()
        .filter_map(|e| e.parent.as_ref())
        .map(|p| quote! { <#p as ::rpstate::StateScope>::PREFIX })
        .collect::<Vec<_>>()
}
