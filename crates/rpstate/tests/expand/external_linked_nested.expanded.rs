use rpstate_macros::rpstate;
pub struct ConnectionPool {
    pub max_connections: ::rpstate::Field<
        u32,
        ::rpstate::DefaultStore,
        ::rpstate::WritableMode,
    >,
    pub timeout_secs: ::rpstate::Field<
        u32,
        ::rpstate::DefaultStore,
        ::rpstate::WritableMode,
    >,
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
    ) -> ::rpstate::Result<Self> {
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
    pub fn __schema_field_max_connections(&self) -> ::rpstate::ReadOnly<u32> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    #[doc(hidden)]
    pub fn __schema_field_timeout_secs(&self) -> ::rpstate::ReadOnly<u32> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn max_connections(
        &self,
    ) -> ::rpstate::Field<u32, ::rpstate::DefaultStore, ::rpstate::WritableMode> {
        self.max_connections.clone()
    }
    pub fn timeout_secs(
        &self,
    ) -> ::rpstate::Field<u32, ::rpstate::DefaultStore, ::rpstate::WritableMode> {
        self.timeout_secs.clone()
    }
}
impl ::rpstate::RpStateNode for ConnectionPool {
    fn new_node(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        path: &str,
    ) -> ::rpstate::Result<Self> {
        Self::new(store, path)
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct ConnectionPool_Data {
    pub max_connections: u32,
    pub timeout_secs: u32,
}
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for ConnectionPool_Data {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private228::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "ConnectionPool_Data",
                false as usize + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "max_connections",
                &self.max_connections,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "timeout_secs",
                &self.timeout_secs,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for ConnectionPool_Data {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private228::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "field identifier",
                    )
                }
                fn visit_u64<__E>(
                    self,
                    __value: u64,
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private228::Ok(__Field::__field0),
                        1u64 => _serde::__private228::Ok(__Field::__field1),
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "max_connections" => _serde::__private228::Ok(__Field::__field0),
                        "timeout_secs" => _serde::__private228::Ok(__Field::__field1),
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"max_connections" => _serde::__private228::Ok(__Field::__field0),
                        b"timeout_secs" => _serde::__private228::Ok(__Field::__field1),
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private228::PhantomData<ConnectionPool_Data>,
                lifetime: _serde::__private228::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = ConnectionPool_Data;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "struct ConnectionPool_Data",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<
                        u32,
                    >(&mut __seq)? {
                        _serde::__private228::Some(__value) => __value,
                        _serde::__private228::None => {
                            return _serde::__private228::Err(
                                _serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct ConnectionPool_Data with 2 elements",
                                ),
                            );
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<
                        u32,
                    >(&mut __seq)? {
                        _serde::__private228::Some(__value) => __value,
                        _serde::__private228::None => {
                            return _serde::__private228::Err(
                                _serde::de::Error::invalid_length(
                                    1usize,
                                    &"struct ConnectionPool_Data with 2 elements",
                                ),
                            );
                        }
                    };
                    _serde::__private228::Ok(ConnectionPool_Data {
                        max_connections: __field0,
                        timeout_secs: __field1,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private228::Option<u32> = _serde::__private228::None;
                    let mut __field1: _serde::__private228::Option<u32> = _serde::__private228::None;
                    while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private228::Option::is_some(&__field0) {
                                    return _serde::__private228::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "max_connections",
                                        ),
                                    );
                                }
                                __field0 = _serde::__private228::Some(
                                    _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                );
                            }
                            __Field::__field1 => {
                                if _serde::__private228::Option::is_some(&__field1) {
                                    return _serde::__private228::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "timeout_secs",
                                        ),
                                    );
                                }
                                __field1 = _serde::__private228::Some(
                                    _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                );
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private228::Some(__field0) => __field0,
                        _serde::__private228::None => {
                            _serde::__private228::de::missing_field("max_connections")?
                        }
                    };
                    let __field1 = match __field1 {
                        _serde::__private228::Some(__field1) => __field1,
                        _serde::__private228::None => {
                            _serde::__private228::de::missing_field("timeout_secs")?
                        }
                    };
                    _serde::__private228::Ok(ConnectionPool_Data {
                        max_connections: __field0,
                        timeout_secs: __field1,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["max_connections", "timeout_secs"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "ConnectionPool_Data",
                FIELDS,
                __Visitor {
                    marker: _serde::__private228::PhantomData::<ConnectionPool_Data>,
                    lifetime: _serde::__private228::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::default::Default for ConnectionPool_Data {
    #[inline]
    fn default() -> ConnectionPool_Data {
        ConnectionPool_Data {
            max_connections: ::core::default::Default::default(),
            timeout_secs: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::clone::Clone for ConnectionPool_Data {
    #[inline]
    fn clone(&self) -> ConnectionPool_Data {
        ConnectionPool_Data {
            max_connections: ::core::clone::Clone::clone(&self.max_connections),
            timeout_secs: ::core::clone::Clone::clone(&self.timeout_secs),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::fmt::Debug for ConnectionPool_Data {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "ConnectionPool_Data",
            "max_connections",
            &self.max_connections,
            "timeout_secs",
            &&self.timeout_secs,
        )
    }
}
impl ConnectionPool_Data {
    #[doc(hidden)]
    pub fn __rpstate_load_from(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        prefix: &str,
    ) -> ::rpstate::Result<Self> {
        Ok(Self {
            max_connections: <::rpstate::DefaultStore as ::rpstate::Store>::get::<
                u32,
            >(&**store, &Self::__rpstate_path(prefix, "max_connections"))?
                .unwrap_or_else(|| 10),
            timeout_secs: <::rpstate::DefaultStore as ::rpstate::Store>::get::<
                u32,
            >(&**store, &Self::__rpstate_path(prefix, "timeout_secs"))?
                .unwrap_or_else(|| 30),
        })
    }
    #[doc(hidden)]
    pub fn __rpstate_save_to(
        &self,
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        prefix: &str,
    ) -> ::rpstate::Result<()> {
        <::rpstate::DefaultStore as ::rpstate::Store>::set(
            &**store,
            &Self::__rpstate_path(prefix, "max_connections"),
            &self.max_connections,
        )?;
        <::rpstate::DefaultStore as ::rpstate::Store>::set(
            &**store,
            &Self::__rpstate_path(prefix, "timeout_secs"),
            &self.timeout_secs,
        )?;
        Ok(())
    }
    fn __rpstate_path(prefix: &str, key: &str) -> ::std::string::String {
        if prefix.is_empty() {
            key.to_string()
        } else {
            ::alloc::__export::must_use({
                ::alloc::fmt::format(
                    format_args!(
                        "{0}.{1}", prefix.trim_end_matches('.'), key
                        .trim_start_matches('.'),
                    ),
                )
            })
        }
    }
}
impl ::rpstate::migration::types::RpType for ConnectionPool_Data {
    const TYPE_HASH: u64 = ::rpstate::migration::types::fnv1a(
        "ConnectionPool_Data".as_bytes(),
    );
    const TYPE_NAME: &'static str = "ConnectionPool_Data";
}
impl ::rpstate::migration::fields::RpStateFields for ConnectionPool_Data {
    const FIELDS: &'static [::rpstate::migration::fields::FieldDescriptor] = &[
        ::rpstate::migration::fields::FieldDescriptor {
            name: "max_connections",
            type_hash: <u32 as ::rpstate::migration::types::RpType>::TYPE_HASH,
            type_name: "stringify!(u32)",
        },
        ::rpstate::migration::fields::FieldDescriptor {
            name: "timeout_secs",
            type_hash: <u32 as ::rpstate::migration::types::RpType>::TYPE_HASH,
            type_name: "stringify!(u32)",
        },
    ];
    const VERSION: u32 = 0u32;
    const SCHEMA_HASH: u64 = ::rpstate::migration::types::schema_hash(Self::FIELDS);
    const PARENT_PREFIX: &'static str = "";
    const MIGRATION_DEPS: &'static [&'static str] = &[];
    fn load_struct(ctx: &mut ::rpstate::MigrationContext) -> ::rpstate::Result<Self> {
        Ok(Self {
            max_connections: ctx.get::<u32>("max_connections")?.unwrap_or_else(|| 10),
            timeout_secs: ctx.get::<u32>("timeout_secs")?.unwrap_or_else(|| 30),
        })
    }
    fn save_struct(
        &self,
        ctx: &mut ::rpstate::MigrationContext,
    ) -> ::rpstate::Result<()> {
        ctx.set("max_connections", &self.max_connections)?;
        ctx.set("timeout_secs", &self.timeout_secs)?;
        Ok(())
    }
}
impl ::rpstate::RpState for ConnectionPool {
    type Data = ConnectionPool_Data;
}
#[allow(non_upper_case_globals)]
const _: () = {
    static __INVENTORY: ::inventory::Node = ::inventory::Node {
        value: &{
            ::rpstate::tauri_codegen::SchemaExportEntry {
                prefix: None,
                struct_name: "ConnectionPool",
                fields: &[
                    ::rpstate::tauri_codegen::FieldExportMeta {
                        name: "max_connections",
                        ts_type: "number",
                        full_ts_type: "number",
                        kind: ::rpstate::tauri_codegen::FieldKind::Plain,
                    },
                    ::rpstate::tauri_codegen::FieldExportMeta {
                        name: "timeout_secs",
                        ts_type: "number",
                        full_ts_type: "number",
                        kind: ::rpstate::tauri_codegen::FieldKind::Plain,
                    },
                ],
            }
        },
        next: ::inventory::__private::UnsafeCell::new(
            ::inventory::__private::Option::None,
        ),
    };
    unsafe extern "C" fn __ctor() {
        unsafe { ::inventory::ErasedNode::submit(__INVENTORY.value, &__INVENTORY) }
    }
    #[used]
    #[link_section = ".CRT$XCU"]
    static __CTOR: unsafe extern "C" fn() = __ctor;
};
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
    ) -> ::rpstate::Result<Self> {
        Ok(Self {
            pool: ::std::sync::Arc::new(
                ConnectionPool::new(
                    store,
                    &::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "{0}.{1}", < Self as ::rpstate::StateScope >::PREFIX,
                                "pool",
                            ),
                        )
                    }),
                )?,
            ),
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_pool(&self) -> ::rpstate::ReadOnly<ConnectionPool> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn pool(&self) -> ::std::sync::Arc<ConnectionPool> {
        self.pool.clone()
    }
}
impl ::rpstate::RpStateNode for DatabaseState {
    fn new_node(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        _path: &str,
    ) -> ::rpstate::Result<Self> {
        Self::new(store)
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct DatabaseState_Data {
    pub pool: <ConnectionPool as ::rpstate::RpState>::Data,
}
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for DatabaseState_Data {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private228::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "DatabaseState_Data",
                false as usize + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "pool",
                &self.pool,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for DatabaseState_Data {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private228::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "field identifier",
                    )
                }
                fn visit_u64<__E>(
                    self,
                    __value: u64,
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private228::Ok(__Field::__field0),
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "pool" => _serde::__private228::Ok(__Field::__field0),
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"pool" => _serde::__private228::Ok(__Field::__field0),
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private228::PhantomData<DatabaseState_Data>,
                lifetime: _serde::__private228::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = DatabaseState_Data;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "struct DatabaseState_Data",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<
                        <ConnectionPool as ::rpstate::RpState>::Data,
                    >(&mut __seq)? {
                        _serde::__private228::Some(__value) => __value,
                        _serde::__private228::None => {
                            return _serde::__private228::Err(
                                _serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct DatabaseState_Data with 1 element",
                                ),
                            );
                        }
                    };
                    _serde::__private228::Ok(DatabaseState_Data {
                        pool: __field0,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private228::Option<
                        <ConnectionPool as ::rpstate::RpState>::Data,
                    > = _serde::__private228::None;
                    while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private228::Option::is_some(&__field0) {
                                    return _serde::__private228::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("pool"),
                                    );
                                }
                                __field0 = _serde::__private228::Some(
                                    _serde::de::MapAccess::next_value::<
                                        <ConnectionPool as ::rpstate::RpState>::Data,
                                    >(&mut __map)?,
                                );
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private228::Some(__field0) => __field0,
                        _serde::__private228::None => {
                            _serde::__private228::de::missing_field("pool")?
                        }
                    };
                    _serde::__private228::Ok(DatabaseState_Data {
                        pool: __field0,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["pool"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "DatabaseState_Data",
                FIELDS,
                __Visitor {
                    marker: _serde::__private228::PhantomData::<DatabaseState_Data>,
                    lifetime: _serde::__private228::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::default::Default for DatabaseState_Data {
    #[inline]
    fn default() -> DatabaseState_Data {
        DatabaseState_Data {
            pool: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::clone::Clone for DatabaseState_Data {
    #[inline]
    fn clone(&self) -> DatabaseState_Data {
        DatabaseState_Data {
            pool: ::core::clone::Clone::clone(&self.pool),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::fmt::Debug for DatabaseState_Data {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "DatabaseState_Data",
            "pool",
            &&self.pool,
        )
    }
}
impl DatabaseState_Data {
    #[doc(hidden)]
    pub fn __rpstate_load_from(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        prefix: &str,
    ) -> ::rpstate::Result<Self> {
        Ok(Self {
            pool: <ConnectionPool as ::rpstate::RpState>::Data::__rpstate_load_from(
                store,
                &Self::__rpstate_path(prefix, "pool"),
            )?,
        })
    }
    #[doc(hidden)]
    pub fn __rpstate_save_to(
        &self,
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        prefix: &str,
    ) -> ::rpstate::Result<()> {
        self.pool.__rpstate_save_to(store, &Self::__rpstate_path(prefix, "pool"))?;
        Ok(())
    }
    fn __rpstate_path(prefix: &str, key: &str) -> ::std::string::String {
        if prefix.is_empty() {
            key.to_string()
        } else {
            ::alloc::__export::must_use({
                ::alloc::fmt::format(
                    format_args!(
                        "{0}.{1}", prefix.trim_end_matches('.'), key
                        .trim_start_matches('.'),
                    ),
                )
            })
        }
    }
}
impl ::rpstate::migration::types::RpType for DatabaseState_Data {
    const TYPE_HASH: u64 = ::rpstate::migration::types::fnv1a(
        "DatabaseState_Data".as_bytes(),
    );
    const TYPE_NAME: &'static str = "DatabaseState_Data";
}
impl ::rpstate::migration::fields::RpStateFields for DatabaseState_Data {
    const FIELDS: &'static [::rpstate::migration::fields::FieldDescriptor] = &[
        ::rpstate::migration::fields::FieldDescriptor {
            name: "pool",
            type_hash: 0xDEADBEEF
                ^ <<ConnectionPool as ::rpstate::RpState>::Data as ::rpstate::migration::types::RpType>::TYPE_HASH,
            type_name: "stringify!(ConnectionPool)",
        },
    ];
    const VERSION: u32 = 0u32;
    const SCHEMA_HASH: u64 = ::rpstate::migration::types::schema_hash(Self::FIELDS);
    const PARENT_PREFIX: &'static str = "sys.database";
    const MIGRATION_DEPS: &'static [&'static str] = &[];
    fn load_struct(ctx: &mut ::rpstate::MigrationContext) -> ::rpstate::Result<Self> {
        Ok(Self {
            pool: {
                let mut sub_ctx = ctx.scoped("pool");
                <<ConnectionPool as ::rpstate::RpState>::Data as ::rpstate::migration::fields::RpStateFields>::load_struct(
                    &mut sub_ctx,
                )?
            },
        })
    }
    fn save_struct(
        &self,
        ctx: &mut ::rpstate::MigrationContext,
    ) -> ::rpstate::Result<()> {
        {
            let mut sub_ctx = ctx.scoped("pool");
            self.pool.save_struct(&mut sub_ctx)?;
        }
        Ok(())
    }
}
impl ::rpstate::RpState for DatabaseState {
    type Data = DatabaseState_Data;
}
#[allow(non_upper_case_globals)]
const _: () = {
    static __INVENTORY: ::inventory::Node = ::inventory::Node {
        value: &{
            ::rpstate::tauri_codegen::SchemaExportEntry {
                prefix: Some("sys.database"),
                struct_name: "DatabaseState",
                fields: &[
                    ::rpstate::tauri_codegen::FieldExportMeta {
                        name: "pool",
                        ts_type: "ConnectionPool",
                        full_ts_type: "ConnectionPool",
                        kind: ::rpstate::tauri_codegen::FieldKind::Nested {
                            struct_name: "ConnectionPool",
                        },
                    },
                ],
            }
        },
        next: ::inventory::__private::UnsafeCell::new(
            ::inventory::__private::Option::None,
        ),
    };
    unsafe extern "C" fn __ctor() {
        unsafe { ::inventory::ErasedNode::submit(__INVENTORY.value, &__INVENTORY) }
    }
    #[used]
    #[link_section = ".CRT$XCU"]
    static __CTOR: unsafe extern "C" fn() = __ctor;
};
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
    ) -> ::rpstate::Result<Self> {
        Ok(Self {
            db_pool_view: {
                const _: fn() = || {
                    fn assert_node_type<T>(_: ::rpstate::ReadOnly<T>) {}
                    let _ = || assert_node_type(
                        unsafe { (&*::core::ptr::null::<DatabaseState>()) }
                            .__schema_field_pool(),
                    );
                    let _ = unsafe { (&*::core::ptr::null::<DatabaseState>()) }
                        .__schema_field_pool();
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
                    <ConnectionPool as ::rpstate::RpStateNode>::new_node(store, &path)?,
                )
            },
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_db_pool_view(&self) -> ::rpstate::ReadOnly<ConnectionPool> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn db_pool_view(&self) -> ::std::sync::Arc<ConnectionPool> {
        self.db_pool_view.clone()
    }
}
impl ::rpstate::RpStateNode for InspectorState {
    fn new_node(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        _path: &str,
    ) -> ::rpstate::Result<Self> {
        Self::new(store)
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct InspectorState_Data {}
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for InspectorState_Data {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private228::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "InspectorState_Data",
                false as usize,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for InspectorState_Data {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private228::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "field identifier",
                    )
                }
                fn visit_u64<__E>(
                    self,
                    __value: u64,
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private228::PhantomData<InspectorState_Data>,
                lifetime: _serde::__private228::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = InspectorState_Data;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "struct InspectorState_Data",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    _: __A,
                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    _serde::__private228::Ok(InspectorState_Data {})
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)?;
                            }
                        }
                    }
                    _serde::__private228::Ok(InspectorState_Data {})
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &[];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "InspectorState_Data",
                FIELDS,
                __Visitor {
                    marker: _serde::__private228::PhantomData::<InspectorState_Data>,
                    lifetime: _serde::__private228::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::default::Default for InspectorState_Data {
    #[inline]
    fn default() -> InspectorState_Data {
        InspectorState_Data {}
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::clone::Clone for InspectorState_Data {
    #[inline]
    fn clone(&self) -> InspectorState_Data {
        InspectorState_Data {}
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::fmt::Debug for InspectorState_Data {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "InspectorState_Data")
    }
}
impl InspectorState_Data {
    #[doc(hidden)]
    pub fn __rpstate_load_from(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        prefix: &str,
    ) -> ::rpstate::Result<Self> {
        Ok(Self {})
    }
    #[doc(hidden)]
    pub fn __rpstate_save_to(
        &self,
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        prefix: &str,
    ) -> ::rpstate::Result<()> {
        Ok(())
    }
    fn __rpstate_path(prefix: &str, key: &str) -> ::std::string::String {
        if prefix.is_empty() {
            key.to_string()
        } else {
            ::alloc::__export::must_use({
                ::alloc::fmt::format(
                    format_args!(
                        "{0}.{1}", prefix.trim_end_matches('.'), key
                        .trim_start_matches('.'),
                    ),
                )
            })
        }
    }
}
impl ::rpstate::migration::types::RpType for InspectorState_Data {
    const TYPE_HASH: u64 = ::rpstate::migration::types::fnv1a(
        "InspectorState_Data".as_bytes(),
    );
    const TYPE_NAME: &'static str = "InspectorState_Data";
}
impl ::rpstate::migration::fields::RpStateFields for InspectorState_Data {
    const FIELDS: &'static [::rpstate::migration::fields::FieldDescriptor] = &[];
    const VERSION: u32 = 0u32;
    const SCHEMA_HASH: u64 = ::rpstate::migration::types::schema_hash(Self::FIELDS);
    const PARENT_PREFIX: &'static str = "ui.inspector";
    const MIGRATION_DEPS: &'static [&'static str] = &[
        <DatabaseState as ::rpstate::StateScope>::PREFIX,
    ];
    fn load_struct(ctx: &mut ::rpstate::MigrationContext) -> ::rpstate::Result<Self> {
        Ok(Self {})
    }
    fn save_struct(
        &self,
        ctx: &mut ::rpstate::MigrationContext,
    ) -> ::rpstate::Result<()> {
        Ok(())
    }
}
impl ::rpstate::RpState for InspectorState {
    type Data = InspectorState_Data;
}
#[allow(non_upper_case_globals)]
const _: () = {
    static __INVENTORY: ::inventory::Node = ::inventory::Node {
        value: &{
            ::rpstate::tauri_codegen::SchemaExportEntry {
                prefix: Some("ui.inspector"),
                struct_name: "InspectorState",
                fields: &[
                    ::rpstate::tauri_codegen::FieldExportMeta {
                        name: "db_pool_view",
                        ts_type: "ConnectionPool",
                        full_ts_type: "ConnectionPool",
                        kind: ::rpstate::tauri_codegen::FieldKind::LookupNode {
                            target_prefix: "pool",
                            struct_name: "ConnectionPool",
                        },
                    },
                ],
            }
        },
        next: ::inventory::__private::UnsafeCell::new(
            ::inventory::__private::Option::None,
        ),
    };
    unsafe extern "C" fn __ctor() {
        unsafe { ::inventory::ErasedNode::submit(__INVENTORY.value, &__INVENTORY) }
    }
    #[used]
    #[link_section = ".CRT$XCU"]
    static __CTOR: unsafe extern "C" fn() = __ctor;
};
fn main() {}
