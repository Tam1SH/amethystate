use rpstate_macros::rpstate;
pub struct DatabaseConfig {
    pub host: ::rpstate::Field<String>,
}
#[automatically_derived]
impl ::core::clone::Clone for DatabaseConfig {
    #[inline]
    fn clone(&self) -> DatabaseConfig {
        DatabaseConfig {
            host: ::core::clone::Clone::clone(&self.host),
        }
    }
}
impl DatabaseConfig {
    pub fn new(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        namespace: &str,
    ) -> ::rpstate::store::Result<Self> {
        Ok(Self {
            host: ::rpstate::store::field_with_path(
                store,
                ::std::sync::Arc::from(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(format_args!("{0}.{1}", namespace, "host"))
                    }),
                ),
                "localhost".to_string(),
            )?,
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_host() -> ::rpstate::store::shared::ReadOnly<String> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn host(&self) -> ::rpstate::Field<String> {
        self.host.clone()
    }
    pub fn set_host(&self, val: String) -> ::rpstate::store::Result<()> {
        self.host.set(val)
    }
}
pub struct SystemSettings {
    pub db: ::std::sync::Arc<DatabaseConfig>,
}
#[automatically_derived]
impl ::core::clone::Clone for SystemSettings {
    #[inline]
    fn clone(&self) -> SystemSettings {
        SystemSettings {
            db: ::core::clone::Clone::clone(&self.db),
        }
    }
}
impl ::rpstate::StateScope for SystemSettings {
    const PREFIX: &'static str = "sys";
}
impl SystemSettings {
    pub fn new(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
    ) -> ::rpstate::store::Result<Self> {
        Ok(Self {
            db: ::std::sync::Arc::new(DatabaseConfig::new(store, "db")?),
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_db() -> ::rpstate::store::shared::ReadOnly<DatabaseConfig> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn db(&self) -> ::std::sync::Arc<DatabaseConfig> {
        self.db.clone()
    }
}
fn main() {}
