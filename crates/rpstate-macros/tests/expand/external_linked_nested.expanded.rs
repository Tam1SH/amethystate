use rpstate_macros::rpstate;
pub struct ConnectionPool {
    pub max_connections: ::rpstate::Field<u32, ::rpstate::store::shared::WritableMode>,
    pub timeout_secs: ::rpstate::Field<u32, ::rpstate::store::shared::WritableMode>,
}
#[automatically_derived]
impl ::core::clone::Clone for ConnectionPool {
    #[inline]
    fn clone(&self) -> ConnectionPool {
        ConnectionPool {
            max_connections: ::core::clone::Clone::clone(&self.max_connections),
            timeout_secs: ::core::clone::Clone::clone(&self.timeout_secs),
        }
    }
}
impl ConnectionPool {
    pub fn new(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        namespace: &str,
    ) -> ::rpstate::store::Result<Self> {
        Ok(Self {
            max_connections: ::rpstate::store::field_with_path(
                store,
                ::std::sync::Arc::from(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("{0}.{1}", namespace, "max_connections"),
                        )
                    }),
                ),
                10,
            )?,
            timeout_secs: ::rpstate::store::field_with_path(
                store,
                ::std::sync::Arc::from(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("{0}.{1}", namespace, "timeout_secs"),
                        )
                    }),
                ),
                30,
            )?,
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_max_connections() -> ::rpstate::store::shared::ReadOnly<u32> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    #[doc(hidden)]
    pub fn __schema_field_timeout_secs() -> ::rpstate::store::shared::ReadOnly<u32> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn max_connections(
        &self,
    ) -> ::rpstate::Field<u32, ::rpstate::store::shared::WritableMode> {
        self.max_connections.clone()
    }
    pub fn timeout_secs(
        &self,
    ) -> ::rpstate::Field<u32, ::rpstate::store::shared::WritableMode> {
        self.timeout_secs.clone()
    }
}
impl ::rpstate::store::shared::RpStateNode for ConnectionPool {
    fn new_node(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        path: &str,
    ) -> ::rpstate::store::Result<Self> {
        Self::new(store, path)
    }
}
pub struct DatabaseState {
    pub pool: ::std::sync::Arc<ConnectionPool>,
}
#[automatically_derived]
impl ::core::clone::Clone for DatabaseState {
    #[inline]
    fn clone(&self) -> DatabaseState {
        DatabaseState {
            pool: ::core::clone::Clone::clone(&self.pool),
        }
    }
}
impl ::rpstate::StateScope for DatabaseState {
    const PREFIX: &'static str = "sys.database";
}
impl DatabaseState {
    pub fn new(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
    ) -> ::rpstate::store::Result<Self> {
        Ok(Self {
            pool: ::std::sync::Arc::new(ConnectionPool::new(store, "pool")?),
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_pool() -> ::rpstate::store::shared::ReadOnly<ConnectionPool> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn pool(&self) -> ::std::sync::Arc<ConnectionPool> {
        self.pool.clone()
    }
}
impl ::rpstate::store::shared::RpStateNode for DatabaseState {
    fn new_node(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        _path: &str,
    ) -> ::rpstate::store::Result<Self> {
        Self::new(store)
    }
}
pub struct InspectorState {
    pub db_pool_view: ::std::sync::Arc<ConnectionPool>,
}
#[automatically_derived]
impl ::core::clone::Clone for InspectorState {
    #[inline]
    fn clone(&self) -> InspectorState {
        InspectorState {
            db_pool_view: ::core::clone::Clone::clone(&self.db_pool_view),
        }
    }
}
impl ::rpstate::StateScope for InspectorState {
    const PREFIX: &'static str = "ui.inspector";
}
impl InspectorState {
    pub fn new(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
    ) -> ::rpstate::store::Result<Self> {
        Ok(Self {
            db_pool_view: {
                const _: fn() = || {
                    fn assert_node_type<T>(_: ::rpstate::store::shared::ReadOnly<T>) {}
                    let _ = || assert_node_type(DatabaseState::__schema_field_pool());
                    let _ = DatabaseState::__schema_field_pool();
                };
                let path = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0}.{1}", < DatabaseState as ::rpstate::StateScope
                            >::PREFIX, "pool",
                        ),
                    )
                });
                ::std::sync::Arc::new(
                    <ConnectionPool as ::rpstate::store::shared::RpStateNode>::new_node(
                        store,
                        &path,
                    )?,
                )
            },
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_db_pool_view() -> ::rpstate::store::shared::ReadOnly<
        ConnectionPool,
    > {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn db_pool_view(&self) -> ::std::sync::Arc<ConnectionPool> {
        self.db_pool_view.clone()
    }
}
impl ::rpstate::store::shared::RpStateNode for InspectorState {
    fn new_node(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        _path: &str,
    ) -> ::rpstate::store::Result<Self> {
        Self::new(store)
    }
}
fn main() {}
