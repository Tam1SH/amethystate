use rpstate_macros::rpstate;
pub struct NetworkState {
    pub port: ::rpstate::Field<u16>,
    pub host: ::rpstate::Field<String>,
}
#[automatically_derived]
impl ::core::clone::Clone for NetworkState {
    #[inline]
    fn clone(&self) -> NetworkState {
        NetworkState {
            port: ::core::clone::Clone::clone(&self.port),
            host: ::core::clone::Clone::clone(&self.host),
        }
    }
}
impl ::rpstate::StateScope for NetworkState {
    const PREFIX: &'static str = "net";
}
impl NetworkState {
    pub fn new(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
    ) -> ::rpstate::store::Result<Self> {
        Ok(Self {
            port: ::rpstate::store::field::<
                Self,
                u16,
                ::rpstate::DefaultStore,
            >(store, "port", 8080)?,
            host: ::rpstate::store::field::<
                Self,
                String,
                ::rpstate::DefaultStore,
            >(store, "host", "127.0.0.1".to_string())?,
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_port() -> ::rpstate::store::shared::Writable<u16> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    #[doc(hidden)]
    pub fn __schema_field_host() -> ::rpstate::store::shared::ReadOnly<String> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn port(&self) -> ::rpstate::Field<u16> {
        self.port.clone()
    }
    pub fn set_port(&self, val: u16) -> ::rpstate::store::Result<()> {
        self.port.set(val)
    }
    pub fn host(&self) -> ::rpstate::Field<String> {
        self.host.clone()
    }
    pub fn set_host(&self, val: String) -> ::rpstate::store::Result<()> {
        self.host.set(val)
    }
}
pub struct UiState {
    pub proxy_port: ::rpstate::Field<u16>,
    pub proxy_host: ::rpstate::Field<String>,
}
#[automatically_derived]
impl ::core::clone::Clone for UiState {
    #[inline]
    fn clone(&self) -> UiState {
        UiState {
            proxy_port: ::core::clone::Clone::clone(&self.proxy_port),
            proxy_host: ::core::clone::Clone::clone(&self.proxy_host),
        }
    }
}
impl ::rpstate::StateScope for UiState {
    const PREFIX: &'static str = "ui";
}
impl UiState {
    pub fn new(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
    ) -> ::rpstate::store::Result<Self> {
        Ok(Self {
            proxy_port: {
                const _: fn() = || {
                    trait TypeCheck<T> {}
                    impl<T> TypeCheck<T> for ::rpstate::store::shared::ReadOnly<T> {}
                    impl<T> TypeCheck<T> for ::rpstate::store::shared::Writable<T> {}
                    fn assert_field_type_matches_lookup<T, M: TypeCheck<T>>(_: M) {}
                    assert_field_type_matches_lookup::<
                        u16,
                        _,
                    >(NetworkState::__schema_field_port());
                };
                let path = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0}.{1}", < NetworkState as ::rpstate::StateScope >::PREFIX,
                            "port",
                        ),
                    )
                });
                ::rpstate::store::field_with_path(
                    store,
                    ::std::sync::Arc::from(path),
                    ::std::default::Default::default(),
                )?
            },
            proxy_host: {
                const _: fn() = || {
                    trait TypeCheck<T> {}
                    impl<T> TypeCheck<T> for ::rpstate::store::shared::ReadOnly<T> {}
                    impl<T> TypeCheck<T> for ::rpstate::store::shared::Writable<T> {}
                    fn assert_field_type_matches_lookup<T, M: TypeCheck<T>>(_: M) {}
                    assert_field_type_matches_lookup::<
                        String,
                        _,
                    >(NetworkState::__schema_field_host());
                };
                let path = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0}.{1}", < NetworkState as ::rpstate::StateScope >::PREFIX,
                            "host",
                        ),
                    )
                });
                ::rpstate::store::field_with_path(
                    store,
                    ::std::sync::Arc::from(path),
                    ::std::default::Default::default(),
                )?
            },
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_proxy_port() -> ::rpstate::store::shared::ReadOnly<u16> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    #[doc(hidden)]
    pub fn __schema_field_proxy_host() -> ::rpstate::store::shared::ReadOnly<String> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn proxy_port(&self) -> ::rpstate::Field<u16> {
        self.proxy_port.clone()
    }
    pub fn proxy_host(&self) -> ::rpstate::Field<String> {
        self.proxy_host.clone()
    }
}
fn main() {}
