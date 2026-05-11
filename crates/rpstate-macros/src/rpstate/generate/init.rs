use crate::rpstate::generate::accessors::{field_mode, lookup_chain};
use crate::rpstate::model::StoreFieldEntry;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};

pub(crate) fn init_fields(entries: &[StoreFieldEntry], is_root: bool) -> Vec<TokenStream2> {
    entries
        .iter()
        .map(|e| init_field(e, is_root))
        .collect::<Vec<_>>()
}

fn init_field(e: &StoreFieldEntry, is_root: bool) -> TokenStream2 {
    let fname = e.ident.as_ref().unwrap();
    let ty = &e.ty;
    let key = e.key.clone().unwrap_or_else(|| fname.to_string());

    if let Some(target) = &e.lookup_node {
        let parent = e.parent.as_ref().expect("lookup_node requires parent");
        let chain = lookup_chain(target, parent);
        let target_span = target.span();
        let target_str = target.to_string();
        quote_spanned! {target_span=>
            #fname: {
                const _: fn() = || {
                    fn assert_node_type<T>(_: ::rpstate::store::access::ReadOnly<T>) {}
                    let _ = || assert_node_type(#chain);
                    let _ = #chain;
                };
                let path = format!("{}.{}", <#parent as ::rpstate::StateScope>::PREFIX, #target_str);
                ::std::sync::Arc::new(<#ty as ::rpstate::store::node::RpStateNode>::new_node(store, &path)?)
            }
        }
    } else if let Some(target) = &e.lookup {
        let parent = e.parent.as_ref().expect("lookup requires parent");
        let chain = lookup_chain(target, parent);
        let target_span = target.span();
        let target_str = target.to_string();
        let def = e
            .default
            .as_ref()
            .map(|d| quote!(#d))
            .unwrap_or_else(|| quote!(::std::default::Default::default()));

        let mode = field_mode(e);
        let write_check = if e.export_mut {
            quote_spanned! { target.span() =>
                fn assert_writable<T>(_: ::rpstate::store::access::Writable<T>) {}
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
                    impl<T> TypeCheck<T> for ::rpstate::store::access::ReadOnly<T> {}
                    impl<T> TypeCheck<T> for ::rpstate::store::access::Writable<T> {}
                    fn assert_field_type_matches_lookup<T, M: TypeCheck<T>>(_: M) {}
                    assert_field_type_matches_lookup::<#ty, _>(#chain);
                };
                let path = format!("{}.{}", <#parent as ::rpstate::StateScope>::PREFIX, #target_str);
                ::rpstate::store::field_with_path::<#ty, _, #mode>(store, ::std::sync::Arc::from(path), #def)?
            }
        }
    } else if e.nested {
        if is_root {
            quote! {
                #fname: ::std::sync::Arc::new(#ty::new(
                    store,
                    &format!("{}.{}", <Self as ::rpstate::StateScope>::PREFIX, #key)
                )?)
            }
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
            let path_expr = if is_root {
                quote! { format!("{}.{}", <Self as ::rpstate::StateScope>::PREFIX, #key) }
            } else {
                quote! { format!("{}.{}", namespace, #key) }
            };
            quote! { #fname: ::rpstate::Field::new_volatile(::std::sync::Arc::from(#path_expr), #def) }
        } else if is_root {
            quote! { #fname: ::rpstate::store::field::<Self, #ty, ::rpstate::DefaultStore>(store, #key, #def)? }
        } else {
            quote! { #fname: ::rpstate::store::field_with_path(store, ::std::sync::Arc::from(#path_expr), #def)? }
        }
    }
}
