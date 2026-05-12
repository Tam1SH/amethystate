pub trait RpType {
    const TYPE_HASH: u64;
    const TYPE_NAME: &'static str;
}

pub const fn fnv1a(s: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut i = 0;
    while i < s.len() {
        hash ^= s[i] as u64;
        hash = hash.wrapping_mul(0x100000000001b3);
        i += 1;
    }
    hash
}

pub const fn schema_hash(fields: &[super::fields::FieldDescriptor]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut i = 0;
    while i < fields.len() {
        let fh = fnv1a(fields[i].name.as_bytes()) ^ fields[i].type_hash;
        h ^= fh;
        h = h.wrapping_mul(0x100000000001b3);
        i += 1;
    }
    h
}

macro_rules! impl_rp_type_primitive {
    ($($t:ty),+) => {
        $(impl RpType for $t {
            const TYPE_HASH: u64 = fnv1a(stringify!($t).as_bytes());
            const TYPE_NAME: &'static str = stringify!($t);
        })+
    }
}

impl_rp_type_primitive!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, bool, String
);

impl<T: RpType> RpType for Vec<T> {
    const TYPE_HASH: u64 = fnv1a(b"Vec") ^ T::TYPE_HASH;
    const TYPE_NAME: &'static str = "Vec";
}

impl<T: RpType> RpType for Option<T> {
    const TYPE_HASH: u64 = fnv1a(b"Option") ^ T::TYPE_HASH;
    const TYPE_NAME: &'static str = "Option";
}
