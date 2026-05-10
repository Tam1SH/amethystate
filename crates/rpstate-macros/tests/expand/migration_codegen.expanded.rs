use rpstate_macros::rpstate;
pub struct NetworkConfig {
    pub host: ::rpstate::Field<String, ::rpstate::store::shared::WritableMode>,
    pub port: ::rpstate::Field<u16, ::rpstate::store::shared::WritableMode>,
    pub connected: ::rpstate::Field<bool, ::rpstate::store::shared::WritableMode>,
}
#[automatically_derived]
impl ::core::clone::Clone for NetworkConfig {
    #[inline]
    fn clone(&self) -> NetworkConfig {
        NetworkConfig {
            host: ::core::clone::Clone::clone(&self.host),
            port: ::core::clone::Clone::clone(&self.port),
            connected: ::core::clone::Clone::clone(&self.connected),
        }
    }
}
impl ::rpstate::StateScope for NetworkConfig {
    const PREFIX: &'static str = "net";
}
impl NetworkConfig {
    pub fn new(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
    ) -> ::rpstate::store::Result<Self> {
        Ok(Self {
            host: ::rpstate::store::field::<
                Self,
                String,
                ::rpstate::DefaultStore,
            >(store, "host", "127.0.0.1".to_string())?,
            port: ::rpstate::store::field::<
                Self,
                u16,
                ::rpstate::DefaultStore,
            >(store, "port", 8080)?,
            connected: ::rpstate::Field::new_volatile(
                ::std::sync::Arc::from("connected".to_string()),
                false,
            ),
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_host() -> ::rpstate::store::shared::ReadOnly<String> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    #[doc(hidden)]
    pub fn __schema_field_port() -> ::rpstate::store::shared::ReadOnly<u16> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    #[doc(hidden)]
    pub fn __schema_field_connected() -> ::rpstate::store::shared::ReadOnly<bool> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn host(
        &self,
    ) -> ::rpstate::Field<String, ::rpstate::store::shared::WritableMode> {
        self.host.clone()
    }
    pub fn port(&self) -> ::rpstate::Field<u16, ::rpstate::store::shared::WritableMode> {
        self.port.clone()
    }
    pub fn connected(
        &self,
    ) -> ::rpstate::Field<bool, ::rpstate::store::shared::WritableMode> {
        self.connected.clone()
    }
}
impl ::rpstate::store::shared::RpStateNode for NetworkConfig {
    fn new_node(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        _path: &str,
    ) -> ::rpstate::store::Result<Self> {
        Self::new(store)
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct NetworkConfig_Data {
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
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for NetworkConfig_Data {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private228::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "NetworkConfig_Data",
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
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for NetworkConfig_Data {
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
                marker: _serde::__private228::PhantomData<NetworkConfig_Data>,
                lifetime: _serde::__private228::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = NetworkConfig_Data;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "struct NetworkConfig_Data",
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
                                    &"struct NetworkConfig_Data with 2 elements",
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
                                    &"struct NetworkConfig_Data with 2 elements",
                                ),
                            );
                        }
                    };
                    _serde::__private228::Ok(NetworkConfig_Data {
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
                    _serde::__private228::Ok(NetworkConfig_Data {
                        host: __field0,
                        port: __field1,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["host", "port"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "NetworkConfig_Data",
                FIELDS,
                __Visitor {
                    marker: _serde::__private228::PhantomData::<NetworkConfig_Data>,
                    lifetime: _serde::__private228::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::default::Default for NetworkConfig_Data {
    #[inline]
    fn default() -> NetworkConfig_Data {
        NetworkConfig_Data {
            host: ::core::default::Default::default(),
            port: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::clone::Clone for NetworkConfig_Data {
    #[inline]
    fn clone(&self) -> NetworkConfig_Data {
        NetworkConfig_Data {
            host: ::core::clone::Clone::clone(&self.host),
            port: ::core::clone::Clone::clone(&self.port),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::fmt::Debug for NetworkConfig_Data {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "NetworkConfig_Data",
            "host",
            &self.host,
            "port",
            &&self.port,
        )
    }
}
impl ::rpstate::store::migration::fields::RpStateFields for NetworkConfig_Data {
    const FIELDS: &'static [::rpstate::store::migration::fields::FieldDescriptor] = &[
        ::rpstate::store::migration::fields::FieldDescriptor {
            name: "host",
            type_hash: <String as ::rpstate::store::migration::types::RpType>::TYPE_HASH,
        },
        ::rpstate::store::migration::fields::FieldDescriptor {
            name: "port",
            type_hash: <u16 as ::rpstate::store::migration::types::RpType>::TYPE_HASH,
        },
    ];
    const VERSION: u32 = 0u32;
    const PARENT_PREFIX: &'static str = "net";
    const MIGRATION_DEPS: &'static [&'static str] = &[];
    fn load_struct(
        ctx: &::rpstate::store::migration::MigrationContext,
    ) -> ::rpstate::store::Result<Self> {
        Ok(Self {
            host: ctx
                .get::<String>("host")?
                .ok_or_else(|| {
                    ::rpstate::store::error::Error::Serialization(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("Field {0} missing during migration", "host"),
                            )
                        }),
                    )
                })?,
            port: ctx
                .get::<u16>("port")?
                .ok_or_else(|| {
                    ::rpstate::store::error::Error::Serialization(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("Field {0} missing during migration", "port"),
                            )
                        }),
                    )
                })?,
        })
    }
    fn save_struct(
        &self,
        ctx: &mut ::rpstate::store::migration::MigrationContext,
    ) -> ::rpstate::store::Result<()> {
        ctx.set("host", &self.host)?;
        ctx.set("port", &self.port)?;
        Ok(())
    }
}
impl ::rpstate::store::shared::RpState for NetworkConfig {
    type Data = NetworkConfig_Data;
}
impl ::rpstate::store::migration::registry::HasMigrations for NetworkConfig {
    const MIGRATION_DEPS: &'static [&'static str] = &[];
    fn migrations() -> ::rpstate::store::migration::Migrator {
        build_migrations()
    }
}
fn build_migrations() -> rpstate::store::migration::Migrator {
    rpstate::store::migration::Migrator::new()
}
fn main() {}
