#[macro_export]
macro_rules! register_migrations {
    ($T:ty) => {
        #[cfg(not(target_arch = "wasm32"))]
        ::inventory::submit! {
            $crate::migration::registry::MigrationStepEntry {
                prefix:       <$T as $crate::StateScope>::PREFIX,
                target_version: 0,
                description: "",
                dependencies: <$T as $crate::migration::registry::HasMigrations>::MIGRATION_DEPS,
                run: |_| Ok(()),
            }
        }
    };
}

#[macro_export]
macro_rules! migrate {
    (
        $old:path => $new:path,
        rename: $rename:tt,
        convert: $convert:tt,
        |$($args:ident),* $(,)?| $logic_block:block
    ) => {
        $crate::migrate!(@route $old => $new, rename: $rename, convert: $convert, |$($args),*| $logic_block);
    };

    (
        $old:path => $new:path,
        rename: $rename:tt,
        |$($args:ident),* $(,)?| $logic_block:block
    ) => {
        $crate::migrate!(@route $old => $new, rename: $rename, |$($args),*| $logic_block);
    };

    (
        $old:path => $new:path,
        convert: $convert:tt,
        |$($args:ident),* $(,)?| $logic_block:block
    ) => {
        $crate::migrate!(@route $old => $new, rename: [], convert: $convert, |$($args),*| $logic_block);
    };

    (
        $old:path => $new:path,
        |$($args:ident),* $(,)?| $logic_block:block
    ) => {
        $crate::migrate!(@route $old => $new, rename: [], |$($args),*| $logic_block);
    };

    (@route $old:path => $new:path, rename: $rename:tt $(, convert: $convert:tt)? , |$old_val:ident| $logic_block:block) => {
        $crate::migrate!(@impl $old => $new, rename: $rename $(, convert: $convert)? , |$old_val, _unused_ctx| $logic_block);
    };

    (@route $old:path => $new:path, rename: $rename:tt $(, convert: $convert:tt)? , |$old_val:ident, $ctx_val:ident| $logic_block:block) => {
        $crate::migrate!(@impl $old => $new, rename: $rename $(, convert: $convert)? , |$old_val, $ctx_val| $logic_block);
    };

    (
        @impl $old:path => $new:path,
        rename: [$($old_f:ident => $new_f:ident),* $(,)?]
        $(, convert: [$($conv_f:ident : $conv_old:ty => $conv_new:ty),* $(,)?])?
        , |$old_val:ident, $ctx_val:ident| $logic_block:block
    ) => {
        const _: () = {
            #[allow(dead_code, clippy::no_effect, unused_variables)]
            fn _check_fields(old: &$old, new: &$new) {
                $(
                    let _ = &old.$old_f;
                    let _ = &new.$new_f;
                )*
            }
        };

        impl $crate::migration::migrate_from::MigrateFrom<$old> for $new {
            const RENAMES: &'static [(&'static str, &'static str)] = &[
                $((stringify!($old_f), stringify!($new_f))),*
            ];

            const CONVERTS: &'static [(&'static str, u64, u64)] = &[
                $($( (
                    stringify!($conv_f),
                    <$conv_old as $crate::migration::types::RpType>::TYPE_HASH,
                    <$conv_new as $crate::migration::types::RpType>::TYPE_HASH,
                ) ),*)?
            ];

            fn migrate($old_val: $old, $ctx_val: &mut $crate::migration::MigrationContext) -> $crate::Result<Self> {
                $logic_block
            }
        }

        $crate::inventory::submit! {
            $crate::migration::registry::MigrationStepEntry {
                prefix: <$new as $crate::migration::fields::RpStateFields>::PARENT_PREFIX,
                target_version: <$new as $crate::migration::fields::RpStateFields>::VERSION,
                dependencies: <$new as $crate::migration::fields::RpStateFields>::MIGRATION_DEPS,
                description: "migrate!",
                schema_hash: <$new as $crate::migration::fields::RpStateFields>::SCHEMA_HASH,
                fields: <$new as $crate::migration::fields::RpStateFields>::FIELDS,
                run: |ctx| {
                    use $crate::migration::fields::RpStateFields;
                    use $crate::migration::migrate_from::MigrateFrom;

                    let old_data = <$old as RpStateFields>::load_struct(ctx)?;
                    let new_data = <$new as MigrateFrom<$old>>::migrate(old_data, ctx)?;

                    for field in <$old as RpStateFields>::FIELDS {
                        let is_renamed = <$new as MigrateFrom<$old>>::RENAMES
                            .iter()
                            .any(|(old_k, _)| *old_k == field.name);
                        let is_kept = <$new as RpStateFields>::FIELDS
                            .iter()
                            .any(|f| f.name == field.name);

                        if is_renamed || !is_kept {
                            ctx.delete(field.name)?;
                        }
                    }

                    new_data.save_struct(ctx)?;
                    Ok(())
                }
            }
        }
    };
}

#[macro_export]
macro_rules! migrate_field {
    ($ctx:ident, $old_obj:ident . $field:ident) => {
        $ctx.nested::<_, _>(stringify!($field), $old_obj.$field)?
    };

    ($ctx:ident, $key:expr, $old_val:expr) => {
        $ctx.nested::<_, _>($key, $old_val)?
    };
}
