use darling::util::SpannedValue;
use darling::{FromField, FromMeta};
use syn::{Expr, Ident, Type, Visibility};

#[derive(Debug, FromMeta)]
pub(crate) struct MacroArgs {
    #[darling(default)]
    pub(crate) prefix: Option<String>,
    #[darling(default)]
    pub(crate) version: Option<u32>,
    #[darling(default)]
    pub(crate) migrations: bool,
}

#[derive(Debug, FromField)]
#[darling(attributes(state, setting))]
pub(crate) struct StoreFieldEntry {
    pub(crate) ident: Option<Ident>,
    pub(crate) vis: Visibility,
    pub(crate) ty: Type,
    #[darling(default)]
    pub(crate) key: Option<String>,
    #[darling(default)]
    pub(crate) default: Option<Expr>,
    #[darling(default)]
    pub(crate) nested: bool,
    #[darling(default)]
    pub(crate) lookup: Option<SpannedValue<String>>,
    #[darling(default)]
    pub(crate) lookup_node: Option<SpannedValue<String>>,
    #[darling(default)]
    pub(crate) parent: Option<Expr>,
    #[darling(default)]
    pub(crate) export_mut: bool,
    #[darling(default)]
    pub(crate) volatile: bool,
}
