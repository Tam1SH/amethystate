use proc_macro::TokenStream;

mod rpstate;

/// Generates a reactive state wrapper for a struct.
///
/// This macro creates accessors that return reactive `Field<T>` handles or `Arc<T>`
/// for nested structures. It handles persistence, signals, and cross-references.
///
/// # Struct Attributes
///
/// * `#[rpstate(prefix = "path")]` - Defines a **Root** struct.
///   * Acts as a top-level entry point in the store.
///   * Generates `pub fn new(store: &Arc<DefaultStore>) -> Result<Self>`.
/// * `#[rpstate]` - Defines a **Nested** struct.
///   * Used as a component within other structures.
///   * Generates `pub fn new(store: &Arc<DefaultStore>, namespace: &str) -> Result<Self>`.
///
/// # Field Attributes (`#[state(...)]`)
///
/// | Option | Type | Description |
/// | :--- | :--- | :--- |
/// | `default` | `Expr` | Initial value if not present in store. Required for leaf fields. |
/// | `nested` | `bool` | Marks field as another `#[rpstate]` struct. |
/// | `volatile` | `bool` | In-memory only. Never saved to or loaded from disk. |
/// | `export_mut` | `bool` | Allows this field to be mutated via `lookup` from other structs. |
/// | `key` | `String` | Overrides the storage key (defaults to field name). |
/// | `lookup` | `String` | Links to a leaf field in a `parent` struct. Supports dot-notation. |
/// | `lookup_node` | `String` | Links to a nested struct node in a `parent` struct. |
/// | `parent` | `Type` | The source `rpstate` struct for `lookup` or `lookup_node`. |
///
/// # Examples
///
/// ### Basic Root and Nesting
/// ```rust,ignore
/// #[rpstate]
/// pub struct NetworkConfig {
///     #[state(default = "localhost".to_string())]
///     pub host: String,
/// }
///
/// #[rpstate(prefix = "settings")]
/// pub struct AppSettings {
///     #[state(nested)]
///     pub net: NetworkConfig,
///
///     #[state(default = false, volatile)]
///     pub debug_mode: bool,
/// }
/// ```
///
/// ### Lookups and Permissions
/// ```rust,ignore
/// #[rpstate(prefix = "database")]
/// pub struct DatabaseState {
///     #[state(default = 10, export_mut)]
///     pub pool_size: u32,
/// }
///
/// #[rpstate(prefix = "ui")]
/// pub struct Dashboard {
///     // Links to DatabaseState.pool_size (read-only by default)
///     #[state(lookup = "pool_size", parent = DatabaseState)]
///     pub view_limit: u32,
///
///     // Links to DatabaseState.pool_size (writable)
///     #[state(lookup = "pool_size", parent = DatabaseState, export_mut)]
///     pub edit_limit: u32,
/// }
/// ```
#[proc_macro_attribute]
pub fn rpstate(args: TokenStream, input: TokenStream) -> TokenStream {
    rpstate::rpstate_impl(args, input)
}

#[proc_macro_derive(RpType)]
pub fn rp_type_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;
    let expanded = quote::quote! {
        impl ::rpstate::migration::types::RpType for #name {
            const TYPE_HASH: u64 = ::rpstate::migration::types::fnv1a(stringify!(#name).as_bytes());
            const TYPE_NAME: &'static str = stringify!(#name);
        }
    };
    TokenStream::from(expanded)
}
