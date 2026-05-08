use rpstate_macros::rpstate;
pub struct AppConfig {
    pub port: ::rpstate::Field<u16, ::rpstate::store::shared::WritableMode>,
    pub session_id: ::rpstate::Field<String, ::rpstate::store::shared::WritableMode>,
}
#[automatically_derived]
impl ::core::clone::Clone for AppConfig {
    #[inline]
    fn clone(&self) -> AppConfig {
        AppConfig {
            port: ::core::clone::Clone::clone(&self.port),
            session_id: ::core::clone::Clone::clone(&self.session_id),
        }
    }
}
impl ::rpstate::StateScope for AppConfig {
    const PREFIX: &'static str = "app";
}
impl AppConfig {
    pub fn new(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
    ) -> ::rpstate::store::Result<Self> {
        Ok(Self {
            port: ::rpstate::store::field::<
                Self,
                u16,
                ::rpstate::DefaultStore,
            >(store, "port", 8080)?,
            session_id: ::rpstate::Field::new_volatile(
                ::std::sync::Arc::from("session_id".to_string()),
                "localhost".to_string(),
            ),
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_port() -> ::rpstate::store::shared::ReadOnly<u16> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    #[doc(hidden)]
    pub fn __schema_field_session_id() -> ::rpstate::store::shared::ReadOnly<String> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn port(&self) -> ::rpstate::Field<u16, ::rpstate::store::shared::WritableMode> {
        self.port.clone()
    }
    pub fn session_id(
        &self,
    ) -> ::rpstate::Field<String, ::rpstate::store::shared::WritableMode> {
        self.session_id.clone()
    }
}
impl ::rpstate::store::shared::RpStateNode for AppConfig {
    fn new_node(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        _path: &str,
    ) -> ::rpstate::store::Result<Self> {
        Self::new(store)
    }
}
fn main() {}
