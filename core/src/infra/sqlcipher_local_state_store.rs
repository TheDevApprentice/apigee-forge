use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
    sync::Mutex,
};

use rusqlite::{params, Connection};

use crate::{error::LocalStateError, ports::LocalStateStore};

pub trait LocalKeyStore: Send + Sync {
    fn get_or_create_key(&self) -> Result<Vec<u8>, LocalStateError>;
}

pub struct KeyringLocalKeyStore {
    service: String,
    username: String,
}

impl KeyringLocalKeyStore {
    pub fn new(service: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            username: username.into(),
        }
    }

    fn entry(&self) -> Result<keyring::Entry, LocalStateError> {
        keyring::Entry::new(&self.service, &self.username)
            .map_err(|_| LocalStateError::StorageFailed)
    }
}

impl LocalKeyStore for KeyringLocalKeyStore {
    fn get_or_create_key(&self) -> Result<Vec<u8>, LocalStateError> {
        let entry = self.entry()?;
        match entry.get_password() {
            Ok(value) => decode_hex(&value),
            Err(keyring::Error::NoEntry) => {
                let mut key = [0_u8; 32];
                getrandom::fill(&mut key).map_err(|_| LocalStateError::StorageFailed)?;
                entry
                    .set_password(&encode_hex(&key))
                    .map_err(|_| LocalStateError::StorageFailed)?;
                Ok(key.to_vec())
            }
            Err(_) => Err(LocalStateError::StorageFailed),
        }
    }
}

pub struct SqlCipherLocalStateStore {
    path: PathBuf,
    connection: Mutex<Connection>,
}

impl SqlCipherLocalStateStore {
    pub fn open(path: impl AsRef<Path>, keys: &dyn LocalKeyStore) -> Result<Self, LocalStateError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| LocalStateError::StorageFailed)?;
        }
        let connection = Connection::open(&path).map_err(|_| LocalStateError::StorageFailed)?;
        let key = keys.get_or_create_key()?;
        connection
            .pragma_update(None, "key", encode_hex(&key))
            .map_err(|_| LocalStateError::StorageFailed)?;
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS local_state (
                    key TEXT PRIMARY KEY NOT NULL,
                    value BLOB NOT NULL
                )",
            )
            .map_err(|_| LocalStateError::StorageFailed)?;
        Ok(Self {
            path,
            connection: Mutex::new(connection),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl LocalStateStore for SqlCipherLocalStateStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, LocalStateError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| LocalStateError::StorageFailed)?;
        let mut statement = connection
            .prepare("SELECT value FROM local_state WHERE key = ?1")
            .map_err(|_| LocalStateError::StorageFailed)?;
        let mut rows = statement
            .query(params![key])
            .map_err(|_| LocalStateError::StorageFailed)?;
        let Some(row) = rows.next().map_err(|_| LocalStateError::StorageFailed)? else {
            return Ok(None);
        };
        row.get(0)
            .map(Some)
            .map_err(|_| LocalStateError::StorageFailed)
    }

    fn set(&self, key: &str, value: Vec<u8>) -> Result<(), LocalStateError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| LocalStateError::StorageFailed)?;
        connection
            .execute(
                "INSERT INTO local_state(key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )
            .map_err(|_| LocalStateError::StorageFailed)?;
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), LocalStateError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| LocalStateError::StorageFailed)?;
        connection
            .execute("DELETE FROM local_state WHERE key = ?1", params![key])
            .map_err(|_| LocalStateError::StorageFailed)?;
        Ok(())
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn decode_hex(value: &str) -> Result<Vec<u8>, LocalStateError> {
    if value.len() % 2 != 0 {
        return Err(LocalStateError::StorageFailed);
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| LocalStateError::StorageFailed)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Mutex};

    use super::{LocalKeyStore, SqlCipherLocalStateStore};
    use crate::ports::LocalStateStore;

    struct FakeKeyStore {
        key: Mutex<Vec<u8>>,
    }

    impl LocalKeyStore for FakeKeyStore {
        fn get_or_create_key(&self) -> Result<Vec<u8>, crate::error::LocalStateError> {
            Ok(self
                .key
                .lock()
                .map_err(|_| crate::error::LocalStateError::StorageFailed)?
                .clone())
        }
    }

    fn test_path() -> PathBuf {
        std::env::temp_dir().join(format!("apigee-forge-sqlcipher-{}.db", std::process::id()))
    }

    #[test]
    fn persists_values_in_an_encrypted_sqlite_store() -> Result<(), crate::error::LocalStateError> {
        let path = test_path();
        let _ = std::fs::remove_file(&path);
        let keys = FakeKeyStore {
            key: Mutex::new(vec![7; 32]),
        };
        let store = SqlCipherLocalStateStore::open(&path, &keys)?;
        store.set("app.mode.v1", b"demo".to_vec())?;
        assert_eq!(store.get("app.mode.v1")?, Some(b"demo".to_vec()));
        store.delete("app.mode.v1")?;
        assert_eq!(store.get("app.mode.v1")?, None);
        drop(store);
        let reopened = SqlCipherLocalStateStore::open(&path, &keys)?;
        assert_eq!(reopened.get("app.mode.v1")?, None);
        drop(reopened);
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn rejects_a_different_encryption_key() -> Result<(), crate::error::LocalStateError> {
        let path = test_path();
        let _ = std::fs::remove_file(&path);
        let original = FakeKeyStore {
            key: Mutex::new(vec![7; 32]),
        };
        let store = SqlCipherLocalStateStore::open(&path, &original)?;
        store.set("secret", b"value".to_vec())?;
        drop(store);
        let wrong = FakeKeyStore {
            key: Mutex::new(vec![8; 32]),
        };
        assert!(SqlCipherLocalStateStore::open(&path, &wrong).is_err());
        let _ = std::fs::remove_file(path);
        Ok(())
    }
}
