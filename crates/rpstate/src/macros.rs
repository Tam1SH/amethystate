#[macro_export]
macro_rules! migrate_field {
    ($ctx:ident, $old_obj:ident . $field:ident) => {
        $ctx.nested::<_, _>(stringify!($field), $old_obj.$field)?
    };

    ($ctx:ident, $key:expr, $old_val:expr) => {
        $ctx.nested::<_, _>($key, $old_val)?
    };
}
