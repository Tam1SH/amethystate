use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct Change<T> {
    pub source: Option<Uuid>,
    pub old_value: T,
    pub new_value: T,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapChange<K, V> {
    Insert {
        key: K,
        value: V,
        source: Option<Uuid>,
    },
    Update {
        key: K,
        old_value: V,
        new_value: V,
        source: Option<Uuid>,
    },
    Remove {
        key: K,
        old_value: V,
        source: Option<Uuid>,
    },
    Clear {
        source: Option<Uuid>,
    },
}

impl<K, V> MapChange<K, V> {
    pub fn key(&self) -> Option<&K> {
        match self {
            MapChange::Insert { key, .. } => Some(key),
            MapChange::Update { key, .. } => Some(key),
            MapChange::Remove { key, .. } => Some(key),
            MapChange::Clear { .. } => None,
        }
    }

    pub fn source(&self) -> Option<Uuid> {
        match self {
            MapChange::Insert { source, .. } => *source,
            MapChange::Update { source, .. } => *source,
            MapChange::Remove { source, .. } => *source,
            MapChange::Clear { source } => *source,
        }
    }
}
