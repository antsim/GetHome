//! API token storage in the operating system's credential store
//! (macOS Keychain). The token is never written to a file.

use keyring::Entry;

const ACCOUNT: &str = "toggl-api-token";

#[derive(Debug, thiserror::Error)]
#[error("credential store error: {0}")]
pub struct SecretError(#[from] keyring::Error);

/// Storage for the Toggl API token. A trait so that it can be replaced in
/// tests or on platforms without a credential store.
pub trait SecretStore: Send + Sync {
    fn load_token(&self) -> Result<Option<String>, SecretError>;
    fn save_token(&self, token: &str) -> Result<(), SecretError>;
    fn delete_token(&self) -> Result<(), SecretError>;
}

/// Keychain-backed [`SecretStore`]. `service` is the app identifier.
#[derive(Debug, Clone)]
pub struct KeychainStore {
    service: String,
}

impl KeychainStore {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    fn entry(&self) -> Result<Entry, SecretError> {
        Ok(Entry::new(&self.service, ACCOUNT)?)
    }
}

impl SecretStore for KeychainStore {
    fn load_token(&self) -> Result<Option<String>, SecretError> {
        match self.entry()?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    fn save_token(&self, token: &str) -> Result<(), SecretError> {
        Ok(self.entry()?.set_password(token)?)
    }

    fn delete_token(&self) -> Result<(), SecretError> {
        match self.entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}
