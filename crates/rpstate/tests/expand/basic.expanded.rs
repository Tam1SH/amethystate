use rpstate_macros::rpstate;
pub struct AppConfig {
    pub port: ::rpstate::Field<u16, ::rpstate::DefaultStore, ::rpstate::WritableMode>,
    pub session_id: ::rpstate::Field<
        String,
        ::rpstate::DefaultStore,
        ::rpstate::WritableMode,
    >,
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
    ) -> ::rpstate::Result<Self> {
        Ok(Self {
            port: ::rpstate::field::<Self, u16>(store, "port", 8080)?,
            session_id: ::rpstate::Field::new_volatile(
                ::std::sync::Arc::from(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "{0}.{1}", < Self as ::rpstate::StateScope >::PREFIX,
                                "session_id",
                            ),
                        )
                    }),
                ),
                "localhost".to_string(),
            ),
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_port(&self) -> ::rpstate::ReadOnly<u16> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    #[doc(hidden)]
    pub fn __schema_field_session_id(&self) -> ::rpstate::ReadOnly<String> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn port(
        &self,
    ) -> ::rpstate::Field<u16, ::rpstate::DefaultStore, ::rpstate::WritableMode> {
        self.port.clone()
    }
    pub fn session_id(
        &self,
    ) -> ::rpstate::Field<String, ::rpstate::DefaultStore, ::rpstate::WritableMode> {
        self.session_id.clone()
    }
}
impl ::rpstate::RpStateNode for AppConfig {
    fn new_node(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        _path: &str,
    ) -> ::rpstate::Result<Self> {
        Self::new(store)
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct AppConfig_Data {
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
    impl _serde::Serialize for AppConfig_Data {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private228::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "AppConfig_Data",
                false as usize + 1,
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
    impl<'de> _serde::Deserialize<'de> for AppConfig_Data {
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
                        "port" => _serde::__private228::Ok(__Field::__field0),
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
                        b"port" => _serde::__private228::Ok(__Field::__field0),
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
                marker: _serde::__private228::PhantomData<AppConfig_Data>,
                lifetime: _serde::__private228::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = AppConfig_Data;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "struct AppConfig_Data",
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
                        u16,
                    >(&mut __seq)? {
                        _serde::__private228::Some(__value) => __value,
                        _serde::__private228::None => {
                            return _serde::__private228::Err(
                                _serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct AppConfig_Data with 1 element",
                                ),
                            );
                        }
                    };
                    _serde::__private228::Ok(AppConfig_Data { port: __field0 })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private228::Option<u16> = _serde::__private228::None;
                    while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private228::Option::is_some(&__field0) {
                                    return _serde::__private228::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("port"),
                                    );
                                }
                                __field0 = _serde::__private228::Some(
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
                            _serde::__private228::de::missing_field("port")?
                        }
                    };
                    _serde::__private228::Ok(AppConfig_Data { port: __field0 })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["port"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "AppConfig_Data",
                FIELDS,
                __Visitor {
                    marker: _serde::__private228::PhantomData::<AppConfig_Data>,
                    lifetime: _serde::__private228::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::default::Default for AppConfig_Data {
    #[inline]
    fn default() -> AppConfig_Data {
        AppConfig_Data {
            port: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::clone::Clone for AppConfig_Data {
    #[inline]
    fn clone(&self) -> AppConfig_Data {
        AppConfig_Data {
            port: ::core::clone::Clone::clone(&self.port),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::fmt::Debug for AppConfig_Data {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "AppConfig_Data",
            "port",
            &&self.port,
        )
    }
}
impl ::rpstate::migration::types::RpType for AppConfig_Data {
    const TYPE_HASH: u64 = ::rpstate::migration::types::fnv1a(
        "AppConfig_Data".as_bytes(),
    );
    const TYPE_NAME: &'static str = "AppConfig_Data";
}
impl ::rpstate::migration::fields::RpStateFields for AppConfig_Data {
    const FIELDS: &'static [::rpstate::migration::fields::FieldDescriptor] = &[
        ::rpstate::migration::fields::FieldDescriptor {
            name: "port",
            type_hash: <u16 as ::rpstate::migration::types::RpType>::TYPE_HASH,
            type_name: "stringify!(u16)",
        },
    ];
    const VERSION: u32 = 0u32;
    const SCHEMA_HASH: u64 = ::rpstate::migration::types::schema_hash(Self::FIELDS);
    const PARENT_PREFIX: &'static str = "app";
    const MIGRATION_DEPS: &'static [&'static str] = &[];
    fn load_struct(ctx: &mut ::rpstate::MigrationContext) -> ::rpstate::Result<Self> {
        Ok(Self {
            port: ctx.get::<u16>("port")?.unwrap_or_else(|| 8080),
        })
    }
    fn save_struct(
        &self,
        ctx: &mut ::rpstate::MigrationContext,
    ) -> ::rpstate::Result<()> {
        ctx.set("port", &self.port)?;
        Ok(())
    }
}
impl ::rpstate::RpState for AppConfig {
    type Data = AppConfig_Data;
}
fn main() {}
