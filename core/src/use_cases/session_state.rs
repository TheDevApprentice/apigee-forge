use std::sync::Arc;

use crate::{
    domain::{AppMode, SessionState},
    error::LocalStateError,
    ports::LocalStateStore,
};

pub const APP_MODE_KEY: &str = "app.mode.v1";
pub const ORGANIZATION_KEY: &str = "app.organization.v1";
pub const ENVIRONMENT_KEY: &str = "app.environment.v1";
pub const SESSION_STATE_KEY: &str = "app.session_state.v1";

pub struct SessionStatePersistence<S> {
    store: Arc<S>,
}

impl<S> SessionStatePersistence<S>
where
    S: LocalStateStore,
{
    pub fn new(store: Arc<S>) -> Self {
        Self { store }
    }

    pub fn load(&self) -> Result<SessionState, LocalStateError> {
        let mode = self
            .store
            .get(APP_MODE_KEY)?
            .map(|value| serde_json::from_slice::<AppMode>(&value))
            .transpose()
            .map_err(|_| LocalStateError::StorageFailed)?
            .unwrap_or(AppMode::Cloud);

        match mode {
            AppMode::Demo => {
                let state = self
                    .store
                    .get(SESSION_STATE_KEY)?
                    .map(|value| serde_json::from_slice::<SessionState>(&value))
                    .transpose()
                    .map_err(|_| LocalStateError::StorageFailed)?;
                Ok(state
                    .filter(|value| value.mode == AppMode::Demo)
                    .unwrap_or_else(SessionState::demo))
            }
            AppMode::Cloud => Ok(SessionState::cloud()),
        }
    }

    pub fn save(&self, state: &SessionState) -> Result<(), LocalStateError> {
        self.store.set(
            APP_MODE_KEY,
            serde_json::to_vec(&state.mode).map_err(|_| LocalStateError::StorageFailed)?,
        )?;
        match state.mode {
            AppMode::Demo => {
                self.store.set(
                    SESSION_STATE_KEY,
                    serde_json::to_vec(state).map_err(|_| LocalStateError::StorageFailed)?,
                )?;
                self.persist_optional_string(
                    ORGANIZATION_KEY,
                    state.organization.as_ref().map(|value| value.as_str()),
                )?;
                self.persist_optional_string(ENVIRONMENT_KEY, state.environment.as_deref())?;
            }
            AppMode::Cloud => {
                self.store.delete(SESSION_STATE_KEY)?;
                self.store.delete(ORGANIZATION_KEY)?;
                self.store.delete(ENVIRONMENT_KEY)?;
            }
        }
        Ok(())
    }

    pub fn switch_mode(&self, mode: AppMode) -> Result<SessionState, LocalStateError> {
        let state = match mode {
            AppMode::Demo => SessionState::demo(),
            AppMode::Cloud => SessionState::cloud(),
        };
        self.save(&state)?;
        Ok(state)
    }

    fn persist_optional_string(
        &self,
        key: &str,
        value: Option<&str>,
    ) -> Result<(), LocalStateError> {
        match value {
            Some(value) => self.store.set(key, value.as_bytes().to_vec()),
            None => self.store.delete(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use super::{
        SessionStatePersistence, APP_MODE_KEY, ENVIRONMENT_KEY, ORGANIZATION_KEY, SESSION_STATE_KEY,
    };
    use crate::{
        domain::{AppMode, GoogleIdentity, SessionState},
        error::LocalStateError,
        ports::LocalStateStore,
    };

    #[derive(Default)]
    struct FakeStore {
        values: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl LocalStateStore for FakeStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, LocalStateError> {
            Ok(self
                .values
                .lock()
                .map_err(|_| LocalStateError::StorageFailed)?
                .get(key)
                .cloned())
        }

        fn set(&self, key: &str, value: Vec<u8>) -> Result<(), LocalStateError> {
            self.values
                .lock()
                .map_err(|_| LocalStateError::StorageFailed)?
                .insert(key.to_owned(), value);
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), LocalStateError> {
            self.values
                .lock()
                .map_err(|_| LocalStateError::StorageFailed)?
                .remove(key);
            Ok(())
        }
    }

    #[test]
    fn persists_demo_and_restores_it() -> Result<(), LocalStateError> {
        let store = Arc::new(FakeStore::default());
        let persistence = SessionStatePersistence::new(store.clone());
        let demo = SessionState::demo();
        persistence.save(&demo)?;
        let restored = persistence.load()?;
        assert_eq!(restored, demo);
        assert!(store.values.lock().unwrap().contains_key(APP_MODE_KEY));
        assert!(store.values.lock().unwrap().contains_key(ORGANIZATION_KEY));
        assert!(store.values.lock().unwrap().contains_key(ENVIRONMENT_KEY));
        Ok(())
    }

    #[test]
    fn restores_cloud_mode_without_invalidating_authentication_boundary(
    ) -> Result<(), LocalStateError> {
        let store = Arc::new(FakeStore::default());
        let persistence = SessionStatePersistence::new(store.clone());
        persistence.save(&SessionState::cloud_authenticated(GoogleIdentity::new(
            "user@example.com",
        )))?;
        let restored = persistence.load()?;
        assert_eq!(restored.mode, AppMode::Cloud);
        assert_eq!(
            restored.status,
            crate::domain::SessionStatus::AuthenticationRequired
        );
        assert!(restored.organization.is_none());
        assert!(restored.environment.is_none());
        assert!(!store.values.lock().unwrap().contains_key(SESSION_STATE_KEY));
        Ok(())
    }

    #[test]
    fn switching_mode_clears_non_applicable_state() -> Result<(), LocalStateError> {
        let store = Arc::new(FakeStore::default());
        let persistence = SessionStatePersistence::new(store.clone());
        persistence.save(&SessionState::demo())?;
        persistence.switch_mode(AppMode::Cloud)?;
        assert_eq!(persistence.load()?.mode, AppMode::Cloud);
        assert!(!store.values.lock().unwrap().contains_key(SESSION_STATE_KEY));
        Ok(())
    }
}
