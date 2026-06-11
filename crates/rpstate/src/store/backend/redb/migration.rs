use super::error::RedbStoreError;
use super::tables::{
    TABLE_DATA, TABLE_META, TABLE_MIGRATION_LOG, TABLE_SCHEMA_SNAPSHOT, TableReader, TableWriter,
};
use crate::migration::AppliedStep;
use crate::store::meta::{PrefixMeta, SchemaSnapshot};
use crate::store::{CodecFormat, MigrationBackend, Result};
use redb::ReadableTable;

pub(super) struct RedbMigrationBackend<'a> {
    txn: &'a redb::WriteTransaction,
}

impl<'a> RedbMigrationBackend<'a> {
    pub(super) fn new(txn: &'a redb::WriteTransaction) -> Self {
        Self { txn }
    }
}

impl MigrationBackend for RedbMigrationBackend<'_> {
    fn format(&self) -> CodecFormat {
        CodecFormat::MessagePack
    }

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
    fn get_meta(&self, prefix: &str) -> Result<Option<PrefixMeta>> {
        Ok(self.txn.load_typed(TABLE_META, prefix)?)
    }

    fn set_meta(&mut self, prefix: &str, meta: &PrefixMeta) -> Result<()> {
        Ok(self.txn.save_typed(TABLE_META, prefix, meta)?)
    }

    fn get_schema_snapshot(&self, prefix: &str) -> Result<Option<SchemaSnapshot>> {
        Ok(self.txn.load_typed(TABLE_SCHEMA_SNAPSHOT, prefix)?)
    }

    fn set_schema_snapshot(&mut self, prefix: &str, snapshot: &SchemaSnapshot) -> Result<()> {
        Ok(self
            .txn
            .save_typed(TABLE_SCHEMA_SNAPSHOT, prefix, snapshot)?)
    }

    fn get_migration_log(&self, prefix: &str) -> Result<Option<Vec<AppliedStep>>> {
        Ok(self.txn.load_typed(TABLE_MIGRATION_LOG, prefix)?)
    }

    fn set_migration_log(&mut self, prefix: &str, log: &[AppliedStep]) -> Result<()> {
        Ok(self.txn.save_typed(TABLE_MIGRATION_LOG, prefix, &log)?)
    }
}
