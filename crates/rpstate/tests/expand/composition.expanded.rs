use rpstate_macros::rpstate;
pub struct NetworkState<S: ::rpstate::Store = ::rpstate::DefaultStore> {
    pub port: ::rpstate::Field<u16, S, ::rpstate::WritableMode>,
    pub host: ::rpstate::Field<String, S, ::rpstate::WritableMode>,
}
#[automatically_derived]
impl<S: ::core::clone::Clone + ::rpstate::Store> ::core::clone::Clone
for NetworkState<S> {
    #[inline]
    fn clone(&self) -> NetworkState<S> {
        NetworkState {
            port: ::core::clone::Clone::clone(&self.port),
            host: ::core::clone::Clone::clone(&self.host),
        }
    }
}
impl<S: ::rpstate::Store> ::rpstate::StateScope for NetworkState<S> {
    const PREFIX: &'static str = "net";
}
impl<S: ::rpstate::Store> NetworkState<S> {
    pub fn new_with(store: &S) -> ::rpstate::Result<Self> {
        use ::rpstate::Store;
        let result = Self {
            port: ::rpstate::store::field::<Self, u16, S>(store, "port", 8080)?,
            host: ::rpstate::store::field::<
                Self,
                String,
                S,
            >(store, "host", "127.0.0.1".to_string())?,
        };
        store.mark_initialized(<Self as ::rpstate::StateScope>::PREFIX)?;
        Ok(result)
    }
    #[doc(hidden)]
    pub fn __schema_field_port(&self) -> ::rpstate::Writable<u16> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    #[doc(hidden)]
    pub fn __schema_field_host(&self) -> ::rpstate::ReadOnly<String> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn port(&self) -> ::rpstate::Field<u16, S, ::rpstate::WritableMode> {
        self.port.clone()
    }
    pub fn host(&self) -> ::rpstate::Field<String, S, ::rpstate::WritableMode> {
        self.host.clone()
    }
}
impl NetworkState<::rpstate::DefaultStore> {
    pub fn new() -> ::rpstate::Result<Self> {
        let store = ::rpstate::global_store();
        Self::new_with(&store)
    }
}
impl<S: ::rpstate::Store> ::rpstate::RpStateNode<S> for NetworkState<S> {
    fn new_node(store: &S, _path: &str) -> ::rpstate::Result<Self> {
        Self::new_with(store)
    }
}
#[serde(crate = "::rpstate::serde")]
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct NetworkState_Data {
    pub host: String,
    pub port: u16,
}
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    use ::rpstate::serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for NetworkState_Data {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private228::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "NetworkState_Data",
                false as usize + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "host",
                &self.host,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "port",
                &self.port,
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
    use ::rpstate::serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for NetworkState_Data {
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
                        "host" => _serde::__private228::Ok(__Field::__field0),
                        "port" => _serde::__private228::Ok(__Field::__field1),
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
                        b"host" => _serde::__private228::Ok(__Field::__field0),
                        b"port" => _serde::__private228::Ok(__Field::__field1),
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
                marker: _serde::__private228::PhantomData<NetworkState_Data>,
                lifetime: _serde::__private228::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = NetworkState_Data;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "struct NetworkState_Data",
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
                        String,
                    >(&mut __seq)? {
                        _serde::__private228::Some(__value) => __value,
                        _serde::__private228::None => {
                            return _serde::__private228::Err(
                                _serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct NetworkState_Data with 2 elements",
                                ),
                            );
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<
                        u16,
                    >(&mut __seq)? {
                        _serde::__private228::Some(__value) => __value,
                        _serde::__private228::None => {
                            return _serde::__private228::Err(
                                _serde::de::Error::invalid_length(
                                    1usize,
                                    &"struct NetworkState_Data with 2 elements",
                                ),
                            );
                        }
                    };
                    _serde::__private228::Ok(NetworkState_Data {
                        host: __field0,
                        port: __field1,
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
                    let mut __field0: _serde::__private228::Option<String> = _serde::__private228::None;
                    let mut __field1: _serde::__private228::Option<u16> = _serde::__private228::None;
                    while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private228::Option::is_some(&__field0) {
                                    return _serde::__private228::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("host"),
                                    );
                                }
                                __field0 = _serde::__private228::Some(
                                    _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                );
                            }
                            __Field::__field1 => {
                                if _serde::__private228::Option::is_some(&__field1) {
                                    return _serde::__private228::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("port"),
                                    );
                                }
                                __field1 = _serde::__private228::Some(
                                    _serde::de::MapAccess::next_value::<u16>(&mut __map)?,
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
                            _serde::__private228::de::missing_field("host")?
                        }
                    };
                    let __field1 = match __field1 {
                        _serde::__private228::Some(__field1) => __field1,
                        _serde::__private228::None => {
                            _serde::__private228::de::missing_field("port")?
                        }
                    };
                    _serde::__private228::Ok(NetworkState_Data {
                        host: __field0,
                        port: __field1,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["host", "port"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "NetworkState_Data",
                FIELDS,
                __Visitor {
                    marker: _serde::__private228::PhantomData::<NetworkState_Data>,
                    lifetime: _serde::__private228::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::default::Default for NetworkState_Data {
    #[inline]
    fn default() -> NetworkState_Data {
        NetworkState_Data {
            host: ::core::default::Default::default(),
            port: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::clone::Clone for NetworkState_Data {
    #[inline]
    fn clone(&self) -> NetworkState_Data {
        NetworkState_Data {
            host: ::core::clone::Clone::clone(&self.host),
            port: ::core::clone::Clone::clone(&self.port),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::fmt::Debug for NetworkState_Data {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "NetworkState_Data",
            "host",
            &self.host,
            "port",
            &&self.port,
        )
    }
}
impl NetworkState_Data {}
impl ::rpstate::migration::types::RpType for NetworkState_Data {
    const TYPE_HASH: u64 = ::rpstate::migration::types::fnv1a(
        "NetworkState_Data".as_bytes(),
    );
    const TYPE_NAME: &'static str = "NetworkState_Data";
}
impl ::rpstate::migration::fields::RpStateFields for NetworkState_Data {
    const FIELDS: &'static [::rpstate::migration::fields::FieldDescriptor] = &[
        ::rpstate::migration::fields::FieldDescriptor {
            name: "host",
            type_hash: <String as ::rpstate::migration::types::RpType>::TYPE_HASH,
            type_name: "String",
        },
        ::rpstate::migration::fields::FieldDescriptor {
            name: "port",
            type_hash: <u16 as ::rpstate::migration::types::RpType>::TYPE_HASH,
            type_name: "u16",
        },
    ];
    const VERSION: u32 = 0u32;
    const SCHEMA_HASH: u64 = ::rpstate::migration::types::schema_hash(Self::FIELDS);
    const PARENT_PREFIX: &'static str = "net";
    const MIGRATION_DEPS: &'static [&'static str] = &[];
    fn load_struct(ctx: &mut ::rpstate::MigrationContext) -> ::rpstate::Result<Self> {
        Ok(Self {
            host: ctx.get::<String>("host")?.unwrap_or_else(|| "127.0.0.1".to_string()),
            port: ctx.get::<u16>("port")?.unwrap_or_else(|| 8080),
        })
    }
    fn save_struct(
        &self,
        ctx: &mut ::rpstate::MigrationContext,
    ) -> ::rpstate::Result<()> {
        ctx.set("host", &self.host)?;
        ctx.set("port", &self.port)?;
        Ok(())
    }
}
impl<S: ::rpstate::Store> ::rpstate::RpState for NetworkState<S> {
    type Data = NetworkState_Data;
}
impl<S: ::rpstate::Store> ::rpstate::RpStateSlice<S> for NetworkState<S> {
    fn load_slice(store: &S) -> ::rpstate::Result<Self> {
        Self::new_with(store)
    }
}
pub struct UiState<S: ::rpstate::Store = ::rpstate::DefaultStore> {
    pub proxy_port: ::rpstate::Field<u16, S, ::rpstate::ReadOnlyMode>,
    pub proxy_host: ::rpstate::Field<String, S, ::rpstate::ReadOnlyMode>,
}
#[automatically_derived]
impl<S: ::core::clone::Clone + ::rpstate::Store> ::core::clone::Clone for UiState<S> {
    #[inline]
    fn clone(&self) -> UiState<S> {
        UiState {
            proxy_port: ::core::clone::Clone::clone(&self.proxy_port),
            proxy_host: ::core::clone::Clone::clone(&self.proxy_host),
        }
    }
}
impl<S: ::rpstate::Store> ::rpstate::StateScope for UiState<S> {
    const PREFIX: &'static str = "ui";
}
impl<S: ::rpstate::Store> UiState<S> {
    pub fn new_with(store: &S) -> ::rpstate::Result<Self> {
        use ::rpstate::Store;
        let result = Self {
            proxy_port: {
                const _: fn() = || {
                    trait TypeCheck<T> {}
                    impl<T> TypeCheck<T> for ::rpstate::ReadOnly<T> {}
                    impl<T> TypeCheck<T> for ::rpstate::Writable<T> {}
                    fn assert_field_type_matches_lookup<T, M: TypeCheck<T>>(_: M) {}
                    assert_field_type_matches_lookup::<
                        u16,
                        _,
                    >(
                        unsafe { (&*::core::ptr::null::<NetworkState>()) }
                            .__schema_field_port(),
                    );
                };
                let path = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0}.{1}", < NetworkState as ::rpstate::StateScope >::PREFIX,
                            "port",
                        ),
                    )
                });
                ::rpstate::store::field_with_path::<
                    u16,
                    _,
                    ::rpstate::ReadOnlyMode,
                >(
                    store,
                    ::std::sync::Arc::from(path),
                    ::std::default::Default::default(),
                )?
            },
            proxy_host: {
                const _: fn() = || {
                    trait TypeCheck<T> {}
                    impl<T> TypeCheck<T> for ::rpstate::ReadOnly<T> {}
                    impl<T> TypeCheck<T> for ::rpstate::Writable<T> {}
                    fn assert_field_type_matches_lookup<T, M: TypeCheck<T>>(_: M) {}
                    assert_field_type_matches_lookup::<
                        String,
                        _,
                    >(
                        unsafe { (&*::core::ptr::null::<NetworkState>()) }
                            .__schema_field_host(),
                    );
                };
                let path = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0}.{1}", < NetworkState as ::rpstate::StateScope >::PREFIX,
                            "host",
                        ),
                    )
                });
                ::rpstate::store::field_with_path::<
                    String,
                    _,
                    ::rpstate::ReadOnlyMode,
                >(
                    store,
                    ::std::sync::Arc::from(path),
                    ::std::default::Default::default(),
                )?
            },
        };
        store.mark_initialized(<Self as ::rpstate::StateScope>::PREFIX)?;
        Ok(result)
    }
    #[doc(hidden)]
    pub fn __schema_field_proxy_port(&self) -> ::rpstate::ReadOnly<u16> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    #[doc(hidden)]
    pub fn __schema_field_proxy_host(&self) -> ::rpstate::ReadOnly<String> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn proxy_port(&self) -> ::rpstate::Field<u16, S, ::rpstate::ReadOnlyMode> {
        self.proxy_port.clone()
    }
    pub fn proxy_host(&self) -> ::rpstate::Field<String, S, ::rpstate::ReadOnlyMode> {
        self.proxy_host.clone()
    }
}
impl UiState<::rpstate::DefaultStore> {
    pub fn new() -> ::rpstate::Result<Self> {
        let store = ::rpstate::global_store();
        Self::new_with(&store)
    }
}
impl<S: ::rpstate::Store> ::rpstate::RpStateNode<S> for UiState<S> {
    fn new_node(store: &S, _path: &str) -> ::rpstate::Result<Self> {
        Self::new_with(store)
    }
}
#[serde(crate = "::rpstate::serde")]
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct UiState_Data {}
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    use ::rpstate::serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for UiState_Data {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private228::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "UiState_Data",
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
    use ::rpstate::serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for UiState_Data {
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
                marker: _serde::__private228::PhantomData<UiState_Data>,
                lifetime: _serde::__private228::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = UiState_Data;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "struct UiState_Data",
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
                    _serde::__private228::Ok(UiState_Data {})
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
                    _serde::__private228::Ok(UiState_Data {})
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &[];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "UiState_Data",
                FIELDS,
                __Visitor {
                    marker: _serde::__private228::PhantomData::<UiState_Data>,
                    lifetime: _serde::__private228::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::default::Default for UiState_Data {
    #[inline]
    fn default() -> UiState_Data {
        UiState_Data {}
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::clone::Clone for UiState_Data {
    #[inline]
    fn clone(&self) -> UiState_Data {
        UiState_Data {}
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::fmt::Debug for UiState_Data {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "UiState_Data")
    }
}
impl UiState_Data {}
impl ::rpstate::migration::types::RpType for UiState_Data {
    const TYPE_HASH: u64 = ::rpstate::migration::types::fnv1a("UiState_Data".as_bytes());
    const TYPE_NAME: &'static str = "UiState_Data";
}
impl ::rpstate::migration::fields::RpStateFields for UiState_Data {
    const FIELDS: &'static [::rpstate::migration::fields::FieldDescriptor] = &[];
    const VERSION: u32 = 0u32;
    const SCHEMA_HASH: u64 = ::rpstate::migration::types::schema_hash(Self::FIELDS);
    const PARENT_PREFIX: &'static str = "ui";
    const MIGRATION_DEPS: &'static [&'static str] = &[
        <NetworkState as ::rpstate::StateScope>::PREFIX,
        <NetworkState as ::rpstate::StateScope>::PREFIX,
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
impl<S: ::rpstate::Store> ::rpstate::RpState for UiState<S> {
    type Data = UiState_Data;
}
impl<S: ::rpstate::Store> ::rpstate::RpStateSlice<S> for UiState<S> {
    fn load_slice(store: &S) -> ::rpstate::Result<Self> {
        Self::new_with(store)
    }
}
fn main() {}
