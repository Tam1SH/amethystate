use super::error::RedbStoreError;
use super::tables::TABLE_DATA;
use crate::migration::RawStorage;
use crate::store::Result;
use redb::ReadableTable;

pub(super) struct RedbRawStorage<'a> {
    txn: &'a redb::WriteTransaction,
}

impl<'a> RedbRawStorage<'a> {
    pub(super) fn new(txn: &'a redb::WriteTransaction) -> Self {
        Self { txn }
    }
}

impl RawStorage for RedbRawStorage<'_> {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let table = self
            .txn
            .open_table(TABLE_DATA)
            .map_err(RedbStoreError::from)?;
        Ok(table
            .get(key)
            .map_err(RedbStoreError::from)?
            .map(|v| v.value().to_vec()))
    }

    fn set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        let mut table = self
            .txn
            .open_table(TABLE_DATA)
            .map_err(RedbStoreError::from)?;
        table.insert(key, value).map_err(RedbStoreError::from)?;
        Ok(())
    }

    fn delete(&mut self, key: &str) -> Result<()> {
        let mut table = self
            .txn
            .open_table(TABLE_DATA)
            .map_err(RedbStoreError::from)?;
        table.remove(key).map_err(RedbStoreError::from)?;
        Ok(())
    }

    fn scan_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>> {
        use redb::ReadableTable;
        let table = self
            .txn
            .open_table(TABLE_DATA)
            .map_err(RedbStoreError::from)?;
        let mut result = Vec::new();
        for entry in table.iter().map_err(RedbStoreError::from)? {
            let (k, v) = entry.map_err(RedbStoreError::from)?;
            let key = k.value().to_string();
            if key.starts_with(prefix) {
                result.push((key, v.value().to_vec()));
            }
        }
        Ok(result)
    }
}
