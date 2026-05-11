use super::{MigrateFrom, RawStorage};
use crate::store::codec::CodecError;
use crate::store::migration::fields::RpStateFields;
use crate::store::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct MigrationContext<'a> {
    prefix: String,
    storage: &'a mut dyn RawStorage,
}

impl<'a> MigrationContext<'a> {
    pub fn new(prefix: String, storage: &'a mut dyn RawStorage) -> Self {
        Self { prefix, storage }
    }

    pub fn nested<TOld, TNew>(&mut self, key: &str, old_data: TOld) -> Result<TNew>
    where
        TOld: RpStateFields,
        TNew: MigrateFrom<TOld> + RpStateFields,
    {
        let mut sub_ctx = self.scoped(key);

        let new_data = TNew::migrate(old_data, &mut sub_ctx)?;

        for old_f in TOld::FIELDS {
            let is_renamed = TNew::RENAMES.iter().any(|(ok, _)| *ok == old_f.name);
            let is_kept = TNew::FIELDS.iter().any(|nf| nf.name == old_f.name);

            if is_renamed || !is_kept {
                sub_ctx.delete(old_f.name)?;
            }
        }
        Ok(new_data)
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let raw = self.get_raw(key)?;
        match raw {
            Some(bytes) => Ok(Some(
                rmp_serde::from_slice(&bytes).map_err(CodecError::from)?,
            )),
            None => Ok(None),
        }
    }

    pub fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()> {
        let bytes = rmp_serde::to_vec(value).map_err(CodecError::from)?;
        self.set_raw(key, &bytes)
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        self.storage.delete(&self.scoped_path(key))
    }

    pub fn rename(&mut self, from: &str, to: &str) -> Result<()> {
        if let Some(bytes) = self.get_raw(from)? {
            self.set_raw(to, &bytes)?;
            self.delete(from)?;
        }
        Ok(())
    }

    pub fn transform<TOld, TNew>(
        &mut self,
        key: &str,
        f: impl FnOnce(TOld) -> Result<TNew>,
    ) -> Result<()>
    where
        TOld: DeserializeOwned,
        TNew: Serialize,
    {
        if let Some(old_val) = self.get::<TOld>(key)? {
            let new_val = f(old_val)?;
            self.set(key, &new_val)?;
        }
        Ok(())
    }

    pub fn merge<TOld1, TOld2, TNew>(
        &mut self,
        from: (&str, &str),
        into: &str,
        f: impl FnOnce(TOld1, TOld2) -> Result<TNew>,
    ) -> Result<()>
    where
        TOld1: DeserializeOwned,
        TOld2: DeserializeOwned,
        TNew: Serialize,
    {
        if let (Some(v1), Some(v2)) = (self.get::<TOld1>(from.0)?, self.get::<TOld2>(from.1)?) {
            let new_val = f(v1, v2)?;
            self.set(into, &new_val)?;
            self.delete(from.0)?;
            self.delete(from.1)?;
        }
        Ok(())
    }

    pub fn split<TOld, TNew1, TNew2>(
        &mut self,
        from: &str,
        into: (&str, &str),
        f: impl FnOnce(TOld) -> Result<(TNew1, TNew2)>,
    ) -> Result<()>
    where
        TOld: DeserializeOwned,
        TNew1: Serialize,
        TNew2: Serialize,
    {
        if let Some(old_val) = self.get::<TOld>(from)? {
            let (v1, v2) = f(old_val)?;
            self.set(into.0, &v1)?;
            self.set(into.1, &v2)?;
            self.delete(from)?;
        }
        Ok(())
    }

    pub fn global_get<T: DeserializeOwned>(&self, full_key: &str) -> Result<Option<T>> {
        let raw = self.storage.get(full_key)?;
        match raw {
            Some(bytes) => Ok(Some(
                rmp_serde::from_slice(&bytes).map_err(CodecError::from)?,
            )),
            None => Ok(None),
        }
    }

    pub fn global_set<T: Serialize>(&mut self, full_key: &str, value: &T) -> Result<()> {
        let bytes = rmp_serde::to_vec(value).map_err(CodecError::from)?;
        self.storage.set(full_key, &bytes)
    }

    pub fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.storage.get(&self.scoped_path(key))
    }

    pub fn set_raw(&mut self, key: &str, value: &[u8]) -> Result<()> {
        self.storage.set(&self.scoped_path(key), value)
    }

    pub fn scoped(&mut self, sub_prefix: &str) -> MigrationContext<'_> {
        MigrationContext {
            prefix: self.scoped_path(sub_prefix),
            storage: self.storage,
        }
    }

    fn scoped_path(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}.{}", self.prefix, key)
        }
    }
}
