use super::error::RedbStoreError;
use super::tables::TABLE_DATA;
use crate::store::Result;
use crate::store::migration::RawStorage;
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
}
