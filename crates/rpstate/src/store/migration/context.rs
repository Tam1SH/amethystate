use super::RawStorage;
use crate::store::Result;

pub struct MigrationContext<'a> {
    prefix: String,
    storage: &'a mut dyn RawStorage,
}

impl<'a> MigrationContext<'a> {
    pub fn new(prefix: String, storage: &'a mut dyn RawStorage) -> Self {
        Self { prefix, storage }
    }

    pub fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.storage.get(&self.scoped(key))
    }

    pub fn set_raw(&mut self, key: &str, value: &[u8]) -> Result<()> {
        self.storage.set(&self.scoped(key), value)
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
    pub fn global_get(&self, full_key: &str) -> Result<Option<Vec<u8>>> {
        self.storage.get(full_key)
    }

    pub fn global_set(&mut self, full_key: &str, value: &[u8]) -> Result<()> {
        self.storage.set(full_key, value)
    }

    fn scoped(&self, key: &str) -> String {
        format!("{}.{}", self.prefix, key)
    }
}
