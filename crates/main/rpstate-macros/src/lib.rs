use proc_macro::TokenStream;

mod migrate;
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
///   * `prefix` (String): Sets the top-level namespace path in the store.
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

/// Transforms a function into a migration step between two state versions.
///
/// The macro derives source and target types from the function signature:
/// - **from**: the type of the first argument
/// - **to**: the inner type of `Result<T>` return type
///
/// The function name becomes the migration step description in the registry.
///
/// # Attributes
///
/// - `#[rename(old_field => new_field)]` — declares a field rename. Can be stacked.
///   Generates a compile-time check that both fields exist on the respective types.
///
/// # Examples
///
/// Simple rename, no context:
///
/// ```rust,ignore
/// mod v1 {
///     #[rpstate(prefix = "app", version = 1)]
///     pub struct Config {
///         #[state(default = "localhost".to_string())]
///         pub host: String,
///         #[state(default = 8080)]
///         pub port: u16,
///     }
/// }
///
/// #[rpstate(prefix = "app", version = 2)]
/// pub struct Config {
///     #[state(default = "localhost".to_string())]
///     pub address: String,
///     #[state(default = 8080)]
///     pub port: u16,
/// }
///
/// #[migrate]
/// #[rename(host => address)]
/// fn migrate_config_v1_to_v2(old: RpData<v1::Config>) -> rpstate::Result<RpData<Config>> {
///     Ok(RpData::<Config> { address: old.host, port: old.port })
/// }
/// ```
///
/// Manual key cleanup via `MigrationContext`:
///
/// ```rust,ignore
/// #[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, RpType)]
/// pub struct ProxyEndpoint {
///     pub url: String,
///     pub timeout_ms: u32,
/// }
///
/// mod v1 {
///     #[rpstate(prefix = "network", version = 1)]
///     pub struct ProxyConfig {
///         #[state(default = "default".into())]
///         pub name: String,
///         pub routes: ReactiveMap<String, String>,
///     }
/// }
///
/// #[rpstate(prefix = "network", version = 2)]
/// pub struct ProxyConfig {
///     #[state(default = "default".into())]
///     pub name: String,
///     pub endpoints: ReactiveMap<String, ProxyEndpoint>,
/// }
///
/// #[migrate]
/// fn migrate_proxy_config_v1_to_v2(
///     old: RpData<v1::ProxyConfig>,
///     ctx: &mut rpstate::migration::MigrationContext,
/// ) -> rpstate::Result<RpData<ProxyConfig>> {
///     for key in old.routes.keys() {
///         ctx.delete(&format!("routes.{}", key))?;
///     }
///     let endpoints = old.routes.into_iter()
///         .map(|(k, v)| (k, ProxyEndpoint { url: v, timeout_ms: 5000 }))
///         .collect();
///     Ok(RpData::<ProxyConfig> { name: old.name, endpoints })
/// }
/// ```
#[proc_macro_attribute]
pub fn migrate(args: TokenStream, input: TokenStream) -> TokenStream {
    migrate::migrate_impl(args, input)
}

#[proc_macro_derive(RpType)]
pub fn rp_type_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;
    let expanded = quote::quote! {
        impl ::rpstate::migration::types::RpType for #name {
            const TYPE_HASH: u32 = ::rpstate::migration::types::fnv1a(stringify!(#name).as_bytes());
            const TYPE_NAME: &'static str = stringify!(#name);
        }
    };
    TokenStream::from(expanded)
}
