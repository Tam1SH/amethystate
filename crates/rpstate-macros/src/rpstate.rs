use darling::util::SpannedValue;
use darling::{ast::NestedMeta, FromField, FromMeta};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Expr, Fields, Ident, Type, Visibility,
};

#[derive(Debug, FromMeta)]
struct MacroArgs {
    #[darling(default)]
    prefix: Option<String>,
    #[darling(default)]
    version: Option<u32>,
    #[darling(default)]
    migrations: bool,
}

#[derive(Debug, FromField)]
#[darling(attributes(state, setting))]
struct StoreFieldEntry {
    ident: Option<Ident>,
    vis: Visibility,
    ty: Type,
    #[darling(default)]
    key: Option<String>,
    #[darling(default)]
    default: Option<Expr>,
    #[darling(default)]
    nested: bool,
    #[darling(default)]
    lookup: Option<SpannedValue<String>>,
    #[darling(default)]
    lookup_node: Option<SpannedValue<String>>,
    #[darling(default)]
    parent: Option<Expr>,
    #[darling(default)]
    export_mut: bool,
    #[darling(default)]
    volatile: bool,
}

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

        if !entry.nested
            && entry.lookup.is_none()
            && entry.lookup_node.is_none()
            && entry.default.is_none()
        {
            return darling::Error::custom(
                "Field must have a default value, be nested, or be a lookup",
            )
            .with_span(&field.ident)
            .write_errors()
            .into();
        }

        entries.push(entry);
    }

    let expanded = generate_code(
        struct_vis,
        struct_name,
        attrs,
        macro_args.prefix.clone(),
        &entries,
        macro_args,
    );

    proc_macro::TokenStream::from(expanded)
}

fn generate_code(
    vis: &Visibility,
    name: &Ident,
    attrs: &[syn::Attribute],
    prefix: Option<String>,
    entries: &[StoreFieldEntry],
    macro_args: MacroArgs,
) -> TokenStream2 {
    let is_root = prefix.is_some();

    let schema_methods = entries.iter().map(|e| {
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
    });

    let leaf_fields: Vec<_> = entries
        .iter()
        .filter(|e| !e.nested && e.lookup.is_none() && e.lookup_node.is_none() && !e.volatile)
        .collect();

    let data_struct_name = format_ident!("{}_Data", name);
    let data_fields = leaf_fields.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let ty = &e.ty;
        quote! { pub #fname: #ty }
    });

    let version_val = macro_args.version.unwrap_or(0);

    let field_descriptors = leaf_fields.iter().map(|e| {
        let name = e.ident.as_ref().unwrap().to_string();
        let ty = &e.ty;
        quote! {
            ::rpstate::store::migration::fields::FieldDescriptor {
                name: #name,
                type_hash: <#ty as ::rpstate::store::migration::types::RpType>::TYPE_HASH,
            }
        }
    });

    let load_fields = leaf_fields.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let key = e.key.clone().unwrap_or_else(|| fname.to_string());
        let ty = &e.ty;
        quote! {
        #fname: ctx.get::<#ty>(#key)?.ok_or_else(|| {
            ::rpstate::store::error::Error::Serialization(format!("Field {} missing during migration", #key))
        })?
    }
    });

    let save_fields = leaf_fields.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let key = e.key.clone().unwrap_or_else(|| fname.to_string());
        quote! { ctx.set(#key, &self.#fname)?; }
    });

    let deps = entries
        .iter()
        .filter_map(|e| e.parent.as_ref())
        .map(|p| quote! { <#p as ::rpstate::StateScope>::PREFIX })
        .collect::<Vec<_>>();

    let prefix_expr = prefix.clone().unwrap_or_default();
    let fields_impl = quote! {
        #[derive(::rpstate::serde::Serialize, ::rpstate::serde::Deserialize, Default, Clone, Debug)]
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub struct #data_struct_name {
            #(#data_fields,)*
        }

        impl ::rpstate::store::migration::fields::RpStateFields for #data_struct_name {
            const FIELDS: &'static [::rpstate::store::migration::fields::FieldDescriptor] = &[
                #(#field_descriptors),*
            ];
            const VERSION: u32 = #version_val;
            const PARENT_PREFIX: &'static str = #prefix_expr;
            const MIGRATION_DEPS: &'static [&'static str] = &[ #(#deps),* ];

            fn load_struct(ctx: &::rpstate::store::migration::MigrationContext) -> ::rpstate::store::Result<Self> {
                Ok(Self { #(#load_fields,)* })
            }

            fn save_struct(&self, ctx: &mut ::rpstate::store::migration::MigrationContext) -> ::rpstate::store::Result<()> {
                #(#save_fields)*
                Ok(())
            }
        }

        impl ::rpstate::store::shared::RpState for #name {
            type Data = #data_struct_name;
        }
    };

    let migrations_registry = if macro_args.migrations {
        quote! {
            impl ::rpstate::store::migration::registry::HasMigrations for #name {
                const MIGRATION_DEPS: &'static [&'static str] = &[ #(#deps),* ];
                fn migrations() -> ::rpstate::store::migration::Migrator {
                    build_migrations()
                }
            }
            ::rpstate::register_migrations!(#name);
        }
    } else {
        quote!()
    };

    let struct_fields = entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let fvis = &e.vis;
        let ty = &e.ty;
        if e.nested || e.lookup_node.is_some() {
            quote! { #fvis #fname: ::std::sync::Arc<#ty> }
        } else {
            let mode = if e.lookup.is_some() {
                if e.export_mut {
                    quote!(::rpstate::store::shared::WritableMode)
                } else {
                    quote!(::rpstate::store::shared::ReadOnlyMode)
                }
            } else {
                quote!(::rpstate::store::shared::WritableMode)
            };
            quote! { #fvis #fname: ::rpstate::Field<#ty, #mode> }
        }
    });

    let init_fields = entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let ty = &e.ty;
        let key = e.key.clone().unwrap_or_else(|| fname.to_string());

        let make_chain = |target: &SpannedValue<String>, parent: &Expr| {
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
        };

        if let Some(target) = &e.lookup_node {
            let parent = e.parent.as_ref().expect("lookup_node requires parent");
            let chain = make_chain(target, parent);
            let target_span = target.span();
            let target_str = target.to_string();
            quote_spanned! {target_span=>
            #fname: {
                const _: fn() = || {
                    fn assert_node_type<T>(_: ::rpstate::store::shared::ReadOnly<T>) {}
                    let _ = || assert_node_type(#chain);
                    let _ = #chain;
                };
                let path = format!("{}.{}", <#parent as ::rpstate::StateScope>::PREFIX, #target_str);
                ::std::sync::Arc::new(<#ty as ::rpstate::store::shared::RpStateNode>::new_node(store, &path)?)
            }
        }
        } else if let Some(target) = &e.lookup {
            let parent = e.parent.as_ref().expect("lookup requires parent");
            let chain = make_chain(target, parent);
            let target_span = target.span();
            let target_str = target.to_string();
            let def = e.default.as_ref().map(|d| quote!(#d)).unwrap_or_else(|| quote!(::std::default::Default::default()));

            let mode = if e.export_mut { quote!(::rpstate::store::shared::WritableMode) } else { quote!(::rpstate::store::shared::ReadOnlyMode) };

            let write_check = if e.export_mut {
                quote_spanned! { target.span() =>
                    fn assert_writable<T>(_: ::rpstate::store::shared::Writable<T>) {}
                    assert_writable(#chain);
                }
            } else {
                quote!()
            };


            quote_spanned! { target_span =>
            #fname: {
                const _: fn() = || {
                    #write_check
                    trait TypeCheck<T> {}
                    impl<T> TypeCheck<T> for ::rpstate::store::shared::ReadOnly<T> {}
                    impl<T> TypeCheck<T> for ::rpstate::store::shared::Writable<T> {}
                    fn assert_field_type_matches_lookup<T, M: TypeCheck<T>>(_: M) {}
                    assert_field_type_matches_lookup::<#ty, _>(#chain);
                };
                let path = format!("{}.{}", <#parent as ::rpstate::StateScope>::PREFIX, #target_str);
                ::rpstate::store::field_with_path::<#ty, _, #mode>(store, ::std::sync::Arc::from(path), #def)?
            }
        }
        } else if e.nested {
            if is_root {
                quote! { #fname: ::std::sync::Arc::new(#ty::new(store, #key)?) }
            } else {
                quote! { #fname: ::std::sync::Arc::new(#ty::new(store, &format!("{}.{}", namespace, #key))?) }
            }
        } else {
            let def = e.default.as_ref().expect("Default required");
            let path_expr = if is_root {
                quote! { #key.to_string() }
            } else {
                quote! { format!("{}.{}", namespace, #key) }
            };

            if e.volatile {
                quote! { #fname: ::rpstate::Field::new_volatile(::std::sync::Arc::from(#path_expr), #def) }
            } else if is_root {
                quote! { #fname: ::rpstate::store::field::<Self, #ty, ::rpstate::DefaultStore>(store, #key, #def)? }
            } else {
                quote! { #fname: ::rpstate::store::field_with_path(store, ::std::sync::Arc::from(#path_expr), #def)? }
            }
        }
    });

    let node_impl = if is_root {
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
    };

    let methods = entries.iter().map(|e| {
        let fname = e.ident.as_ref().unwrap();
        let ty = &e.ty;

        if e.nested || e.lookup_node.is_some() {
            quote! { pub fn #fname(&self) -> ::std::sync::Arc<#ty> { self.#fname.clone() } }
        } else {
            let mode = if e.lookup.is_some() {
                if e.export_mut {
                    quote!(::rpstate::store::shared::WritableMode)
                } else {
                    quote!(::rpstate::store::shared::ReadOnlyMode)
                }
            } else {
                quote!(::rpstate::store::shared::WritableMode)
            };

            quote! {
                pub fn #fname(&self) -> ::rpstate::Field<#ty, #mode> {
                    self.#fname.clone()
                }
            }
        }
    });

    let scope = prefix.map(
        |p| quote! { impl ::rpstate::StateScope for #name { const PREFIX: &'static str = #p; } },
    );
    let constructor = if is_root {
        quote! { pub fn new(store: &::std::sync::Arc<::rpstate::DefaultStore>) -> ::rpstate::store::Result<Self> { Ok(Self { #(#init_fields,)* }) } }
    } else {
        quote! { pub fn new(store: &::std::sync::Arc<::rpstate::DefaultStore>, namespace: &str) -> ::rpstate::store::Result<Self> { Ok(Self { #(#init_fields,)* }) } }
    };

    quote! {
        #[derive(Clone)] #(#attrs)* #vis struct #name { #(#struct_fields,)* }
        #scope
        impl #name { #constructor #(#schema_methods)* #(#methods)* }
        #node_impl
        #fields_impl
        #migrations_registry
    }
}
