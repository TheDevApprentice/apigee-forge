use crate::error::LocalStateError;

pub trait LocalStateStore: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, LocalStateError>;
    fn set(&self, key: &str, value: Vec<u8>) -> Result<(), LocalStateError>;
    fn delete(&self, key: &str) -> Result<(), LocalStateError>;
}
