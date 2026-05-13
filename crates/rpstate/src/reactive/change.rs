#[derive(Debug, Clone, PartialEq)]
pub struct Change<T> {
    pub old_value: T,
    pub new_value: T,
}
impl<T> Change<T> {
    pub fn with_value(mut self, value: T) -> Self {
        self.new_value = value;
        self
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum MapChange<K, V> {
    Insert { key: K, value: V },
    Update { key: K, old_value: V, new_value: V },
    Remove { key: K, old_value: V },
    Clear,
}
impl<K, V> MapChange<K, V> {
    pub fn key(&self) -> Option<&K> {
        match self {
            MapChange::Insert { key, .. } => Some(key),
            MapChange::Update { key, .. } => Some(key),
            MapChange::Remove { key, .. } => Some(key),
            MapChange::Clear => None,
        }
    }
}
