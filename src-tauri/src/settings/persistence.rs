//! Non-secret settings persisted with `tauri-plugin-store` in the app data
//! directory (never in the repository).

use std::sync::Arc;

use serde::{de::DeserializeOwned, Serialize};
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Store, StoreExt};

use super::Settings;
use crate::toggl::Account;

const STORE_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";
const ACCOUNT_KEY: &str = "account";

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("settings store error: {0}")]
    Store(#[from] tauri_plugin_store::Error),
    #[error("could not serialize settings: {0}")]
    Serialize(#[from] serde_json::Error),
}

pub struct SettingsRepository {
    store: Arc<Store<Wry>>,
}

impl SettingsRepository {
    pub fn open(app: &AppHandle) -> Result<Self, PersistenceError> {
        Ok(Self {
            store: app.store(STORE_FILE)?,
        })
    }

    /// Loads settings, falling back to defaults for missing or invalid data.
    pub fn load_settings(&self) -> Settings {
        self.get::<Settings>(SETTINGS_KEY)
            .unwrap_or_default()
            .sanitized()
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), PersistenceError> {
        self.put(SETTINGS_KEY, settings)
    }

    pub fn load_account(&self) -> Option<Account> {
        self.get(ACCOUNT_KEY)
    }

    pub fn save_account(&self, account: Option<&Account>) -> Result<(), PersistenceError> {
        match account {
            Some(account) => self.put(ACCOUNT_KEY, account),
            None => {
                self.store.delete(ACCOUNT_KEY);
                Ok(self.store.save()?)
            }
        }
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let value = self.store.get(key)?;
        serde_json::from_value(value)
            .map_err(|err| log::warn!("ignoring invalid stored value for {key}: {err}"))
            .ok()
    }

    fn put<T: Serialize>(&self, key: &str, value: &T) -> Result<(), PersistenceError> {
        self.store.set(key, serde_json::to_value(value)?);
        Ok(self.store.save()?)
    }
}
