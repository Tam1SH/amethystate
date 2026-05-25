use proc_macro::TokenStream;

mod rpstate;

/// Generates a persistent state wrapper for a struct.
///
/// This macro creates structures that manage persistence, reactive subscribers,
/// and migrations. Depending on the selected `mode`, it generates either reactive
/// `Field<T>` accessors or a flat persistent-only model.
///
/// # Struct Attributes (`#[rpstate(...)]`)
///
/// * `#[rpstate(prefix = "path", version = 1, mode = "reactive")]` - Defines a **Root** struct.
///   * `prefix` (optional String): Sets the top-level namespace path in the store.
///     Generates `pub fn new(store: &Arc<DefaultStore>) -> Result<Self>`.
///   * `version` (optional u32): Schema version for migrations (defaults to 0).
///   * `mode` (optional String): Controls the generated code paradigm. One of:
///     * `"reactive"` (default): Generates fine-grained reactive `Field<T>` accessors.
///     * `"persistent"`: Generates a flat struct with plain-type fields and synchronous `.save()` / `.save_lazy()` methods.
///     * `"both"`: Generates both reactive accessors on `#name` and a separate `#name_Persistent` flat struct.
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
/// ### Reactive Mode (Default)
/// ```rust,ignore
/// #[rpstate(prefix = "settings")]
/// pub struct AppSettings {
///     #[state(default = "localhost".to_string())]
///     pub host: String,
///
///     #[state(default = false, volatile)]
///     pub debug_mode: bool,
/// }
///
/// // Usage:
/// // let settings = AppSettings::new(&store)?;
/// // let _sub = settings.host().subscribe(|val| println!("Host: {val}"));
/// // settings.host().set("10.0.0.1".to_string())?;
/// ```
///
/// ### Persistent-only Mode
/// ```rust,ignore
/// #[rpstate(prefix = "network", mode = "persistent")]
/// pub struct NetworkConfig {
///     #[state(default = "localhost".to_string())]
///     pub host: String,
///     #[state(default = 8080)]
///     pub port: u16,
/// }
///
/// // Usage:
/// // let mut cfg = NetworkConfig::load(&store)?;
/// // cfg.host = "10.0.0.1".to_string(); // Direct field mutation (plain types)
/// // cfg.save_lazy()?;                  // RAM-buffer write (debounced/background)
/// // cfg.save()?;                       // Immediate synchronous flush to disk
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
