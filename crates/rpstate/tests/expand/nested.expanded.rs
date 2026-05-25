use rpstate_macros::rpstate;
pub struct DatabaseConfig {
    pub host: ::rpstate::Field<String, ::rpstate::DefaultStore, ::rpstate::WritableMode>,
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
    ) -> ::rpstate::Result<Self> {
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
    pub fn __schema_field_host(&self) -> ::rpstate::ReadOnly<String> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn host(
        &self,
    ) -> ::rpstate::Field<String, ::rpstate::DefaultStore, ::rpstate::WritableMode> {
        self.host.clone()
    }
}
impl ::rpstate::RpStateNode for DatabaseConfig {
    fn new_node(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        path: &str,
    ) -> ::rpstate::Result<Self> {
        Self::new(store, path)
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct DatabaseConfig_Data {
    pub host: String,
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
    impl _serde::Serialize for DatabaseConfig_Data {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private228::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "DatabaseConfig_Data",
                false as usize + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "host",
                &self.host,
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
    impl<'de> _serde::Deserialize<'de> for DatabaseConfig_Data {
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
                        "host" => _serde::__private228::Ok(__Field::__field0),
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
                marker: _serde::__private228::PhantomData<DatabaseConfig_Data>,
                lifetime: _serde::__private228::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = DatabaseConfig_Data;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "struct DatabaseConfig_Data",
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
                                    &"struct DatabaseConfig_Data with 1 element",
                                ),
                            );
                        }
                    };
                    _serde::__private228::Ok(DatabaseConfig_Data {
                        host: __field0,
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
                    _serde::__private228::Ok(DatabaseConfig_Data {
                        host: __field0,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["host"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "DatabaseConfig_Data",
                FIELDS,
                __Visitor {
                    marker: _serde::__private228::PhantomData::<DatabaseConfig_Data>,
                    lifetime: _serde::__private228::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::default::Default for DatabaseConfig_Data {
    #[inline]
    fn default() -> DatabaseConfig_Data {
        DatabaseConfig_Data {
            host: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::clone::Clone for DatabaseConfig_Data {
    #[inline]
    fn clone(&self) -> DatabaseConfig_Data {
        DatabaseConfig_Data {
            host: ::core::clone::Clone::clone(&self.host),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::fmt::Debug for DatabaseConfig_Data {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "DatabaseConfig_Data",
            "host",
            &&self.host,
        )
    }
}
impl DatabaseConfig_Data {
    #[doc(hidden)]
    pub fn __rpstate_load_from(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        prefix: &str,
    ) -> ::rpstate::Result<Self> {
        Ok(Self {
            host: <::rpstate::DefaultStore as ::rpstate::Store>::get::<
                String,
            >(&**store, &Self::__rpstate_path(prefix, "host"))?
                .unwrap_or_else(|| "localhost".to_string()),
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
            &Self::__rpstate_path(prefix, "host"),
            &self.host,
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
impl ::rpstate::migration::types::RpType for DatabaseConfig_Data {
    const TYPE_HASH: u64 = ::rpstate::migration::types::fnv1a(
        "DatabaseConfig_Data".as_bytes(),
    );
    const TYPE_NAME: &'static str = "DatabaseConfig_Data";
}
impl ::rpstate::migration::fields::RpStateFields for DatabaseConfig_Data {
    const FIELDS: &'static [::rpstate::migration::fields::FieldDescriptor] = &[
        ::rpstate::migration::fields::FieldDescriptor {
            name: "host",
            type_hash: <String as ::rpstate::migration::types::RpType>::TYPE_HASH,
            type_name: "stringify!(String)",
        },
    ];
    const VERSION: u32 = 0u32;
    const SCHEMA_HASH: u64 = ::rpstate::migration::types::schema_hash(Self::FIELDS);
    const PARENT_PREFIX: &'static str = "";
    const MIGRATION_DEPS: &'static [&'static str] = &[];
    fn load_struct(ctx: &mut ::rpstate::MigrationContext) -> ::rpstate::Result<Self> {
        Ok(Self {
            host: ctx.get::<String>("host")?.unwrap_or_else(|| "localhost".to_string()),
        })
    }
    fn save_struct(
        &self,
        ctx: &mut ::rpstate::MigrationContext,
    ) -> ::rpstate::Result<()> {
        ctx.set("host", &self.host)?;
        Ok(())
    }
}
impl ::rpstate::RpState for DatabaseConfig {
    type Data = DatabaseConfig_Data;
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
    ) -> ::rpstate::Result<Self> {
        Ok(Self {
            db: ::std::sync::Arc::new(
                DatabaseConfig::new(
                    store,
                    &::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "{0}.{1}", < Self as ::rpstate::StateScope >::PREFIX, "db",
                            ),
                        )
                    }),
                )?,
            ),
        })
    }
    #[doc(hidden)]
    pub fn __schema_field_db(&self) -> ::rpstate::ReadOnly<DatabaseConfig> {
        ::core::panicking::panic("internal error: entered unreachable code")
    }
    pub fn db(&self) -> ::std::sync::Arc<DatabaseConfig> {
        self.db.clone()
    }
}
impl ::rpstate::RpStateNode for SystemSettings {
    fn new_node(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        _path: &str,
    ) -> ::rpstate::Result<Self> {
        Self::new(store)
    }
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct SystemSettings_Data {
    pub db: <DatabaseConfig as ::rpstate::RpState>::Data,
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
    impl _serde::Serialize for SystemSettings_Data {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private228::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "SystemSettings_Data",
                false as usize + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "db",
                &self.db,
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
    impl<'de> _serde::Deserialize<'de> for SystemSettings_Data {
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
                        "db" => _serde::__private228::Ok(__Field::__field0),
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
                        b"db" => _serde::__private228::Ok(__Field::__field0),
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
                marker: _serde::__private228::PhantomData<SystemSettings_Data>,
                lifetime: _serde::__private228::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = SystemSettings_Data;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "struct SystemSettings_Data",
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
                        <DatabaseConfig as ::rpstate::RpState>::Data,
                    >(&mut __seq)? {
                        _serde::__private228::Some(__value) => __value,
                        _serde::__private228::None => {
                            return _serde::__private228::Err(
                                _serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct SystemSettings_Data with 1 element",
                                ),
                            );
                        }
                    };
                    _serde::__private228::Ok(SystemSettings_Data {
                        db: __field0,
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
                        <DatabaseConfig as ::rpstate::RpState>::Data,
                    > = _serde::__private228::None;
                    while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private228::Option::is_some(&__field0) {
                                    return _serde::__private228::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("db"),
                                    );
                                }
                                __field0 = _serde::__private228::Some(
                                    _serde::de::MapAccess::next_value::<
                                        <DatabaseConfig as ::rpstate::RpState>::Data,
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
                            _serde::__private228::de::missing_field("db")?
                        }
                    };
                    _serde::__private228::Ok(SystemSettings_Data {
                        db: __field0,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["db"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "SystemSettings_Data",
                FIELDS,
                __Visitor {
                    marker: _serde::__private228::PhantomData::<SystemSettings_Data>,
                    lifetime: _serde::__private228::PhantomData,
                },
            )
        }
    }
};
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::default::Default for SystemSettings_Data {
    #[inline]
    fn default() -> SystemSettings_Data {
        SystemSettings_Data {
            db: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::clone::Clone for SystemSettings_Data {
    #[inline]
    fn clone(&self) -> SystemSettings_Data {
        SystemSettings_Data {
            db: ::core::clone::Clone::clone(&self.db),
        }
    }
}
#[automatically_derived]
#[allow(non_camel_case_types)]
impl ::core::fmt::Debug for SystemSettings_Data {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "SystemSettings_Data",
            "db",
            &&self.db,
        )
    }
}
impl SystemSettings_Data {
    #[doc(hidden)]
    pub fn __rpstate_load_from(
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        prefix: &str,
    ) -> ::rpstate::Result<Self> {
        Ok(Self {
            db: <DatabaseConfig as ::rpstate::RpState>::Data::__rpstate_load_from(
                store,
                &Self::__rpstate_path(prefix, "db"),
            )?,
        })
    }
    #[doc(hidden)]
    pub fn __rpstate_save_to(
        &self,
        store: &::std::sync::Arc<::rpstate::DefaultStore>,
        prefix: &str,
    ) -> ::rpstate::Result<()> {
        self.db.__rpstate_save_to(store, &Self::__rpstate_path(prefix, "db"))?;
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
impl ::rpstate::migration::types::RpType for SystemSettings_Data {
    const TYPE_HASH: u64 = ::rpstate::migration::types::fnv1a(
        "SystemSettings_Data".as_bytes(),
    );
    const TYPE_NAME: &'static str = "SystemSettings_Data";
}
impl ::rpstate::migration::fields::RpStateFields for SystemSettings_Data {
    const FIELDS: &'static [::rpstate::migration::fields::FieldDescriptor] = &[
        ::rpstate::migration::fields::FieldDescriptor {
            name: "db",
            type_hash: 0xDEADBEEF
                ^ <<DatabaseConfig as ::rpstate::RpState>::Data as ::rpstate::migration::types::RpType>::TYPE_HASH,
            type_name: "stringify!(DatabaseConfig)",
        },
    ];
    const VERSION: u32 = 0u32;
    const SCHEMA_HASH: u64 = ::rpstate::migration::types::schema_hash(Self::FIELDS);
    const PARENT_PREFIX: &'static str = "sys";
    const MIGRATION_DEPS: &'static [&'static str] = &[];
    fn load_struct(ctx: &mut ::rpstate::MigrationContext) -> ::rpstate::Result<Self> {
        Ok(Self {
            db: {
                let mut sub_ctx = ctx.scoped("db");
                <<DatabaseConfig as ::rpstate::RpState>::Data as ::rpstate::migration::fields::RpStateFields>::load_struct(
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
            let mut sub_ctx = ctx.scoped("db");
            self.db.save_struct(&mut sub_ctx)?;
        }
        Ok(())
    }
}
impl ::rpstate::RpState for SystemSettings {
    type Data = SystemSettings_Data;
}
fn main() {}
