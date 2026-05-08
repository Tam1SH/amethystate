use super::RawStorage;
use crate::store::Result;
use crate::store::error::Error;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct MigrationContext<'a> {
    prefix: String,
    storage: &'a mut dyn RawStorage,
}

impl<'a> MigrationContext<'a> {
    pub fn new(prefix: String, storage: &'a mut dyn RawStorage) -> Self {
        Self { prefix, storage }
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let raw = self.get_raw(key)?;
        match raw {
            Some(bytes) => Ok(Some(
                rmp_serde::from_slice(&bytes).map_err(|e| Error::Serialization(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    pub fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()> {
        let bytes = rmp_serde::to_vec(value).map_err(|e| Error::Serialization(e.to_string()))?;
        self.set_raw(key, &bytes)
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        self.storage.delete(&self.scoped(key))
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
                rmp_serde::from_slice(&bytes).map_err(|e| Error::Serialization(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    pub fn global_set<T: Serialize>(&mut self, full_key: &str, value: &T) -> Result<()> {
        let bytes = rmp_serde::to_vec(value).map_err(|e| Error::Serialization(e.to_string()))?;
        self.storage.set(full_key, &bytes)
    }

    pub fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.storage.get(&self.scoped(key))
    }

    pub fn set_raw(&mut self, key: &str, value: &[u8]) -> Result<()> {
        self.storage.set(&self.scoped(key), value)
    }

    fn scoped(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}.{}", self.prefix, key)
        }
    }
}
