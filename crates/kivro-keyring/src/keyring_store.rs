//! Adapter over the OS credential store
//! * Windows — Credential Manager
//! * macOS — Keychain
//! * Linux — Secret Service (GNOME Keyring, KWallet, …)
//!
//! # Key layout
//!
//! One credential per secret:
//!
//! ```text
//! service = "<app>:<project>:<environment>"
//! user    = "<SECRET_NAME>"
//! ```

use std::collections::BTreeSet;

use kivro_core::{Error, Result, Scope, SecretName, SecretStore, SecretString, StoreKey};

/// reserved user name for the per-scope name index
const INDEX_USER: &str = "__index";

/// Credential store backed [`SecretStore`]
pub struct KeyringStore {
    app_namespace: String,
}

impl KeyringStore {
    /// Create a store rooted at app_namespace
    pub fn new(app_namespace: impl Into<String>) -> Self {
        Self {
            app_namespace: app_namespace.into(),
        }
    }

    fn entry(&self, scope: &Scope, user: &str) -> Result<keyring::Entry> {
        let service = scope.service_id(&self.app_namespace);
        keyring::Entry::new(&service, user).map_err(|e| map_error("open", e))
    }

    fn read_index(&self, scope: &Scope) -> Result<BTreeSet<SecretName>> {
        let entry = self.entry(scope, INDEX_USER)?;
        match entry.get_password() {
            Ok(raw) => {
                let names: Vec<String> = serde_json::from_str(&raw).unwrap_or_default();
                Ok(names
                    .into_iter()
                    .filter_map(|n| SecretName::new(n).ok())
                    .collect())
            }
            Err(keyring::Error::NoEntry) => Ok(BTreeSet::new()),
            Err(e) => Err(map_error("list", e)),
        }
    }

    fn write_index(&self, scope: &Scope, names: &BTreeSet<SecretName>) -> Result<()> {
        let entry = self.entry(scope, INDEX_USER)?;
        if names.is_empty() {
            return match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(map_error("list", e)),
            };
        }
        let payload = serde_json::to_string(&names.iter().map(|n| n.as_str()).collect::<Vec<_>>())
            .map_err(|e| Error::Store {
                operation: "list",
                message: e.to_string(),
            })?;
        entry
            .set_password(&payload)
            .map_err(|e| map_error("list", e))
    }
}

impl SecretStore for KeyringStore {
    fn backend(&self) -> &str {
        if cfg!(target_os = "windows") {
            "windows-credential-manager"
        } else if cfg!(target_os = "macos") {
            "macos-keychain"
        } else {
            "secret-service"
        }
    }

    fn check_available(&self) -> Result<()> {
        let scope = Scope::new(
            kivro_core::ProjectName::new("kivro-probe").expect("valid"),
            kivro_core::EnvironmentName::new("probe").expect("valid"),
        );
        let entry = self.entry(&scope, INDEX_USER)?;
        match entry.get_password() {
            Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Error::StoreUnavailable {
                backend: self.backend().to_string(),
                message: describe(e),
            }),
        }
    }

    fn get(&self, key: &StoreKey) -> Result<Option<SecretString>> {
        let entry = self.entry(&key.scope, key.name.as_str())?;
        match entry.get_password() {
            Ok(value) => Ok(Some(SecretString::new(value))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(map_error("get", e)),
        }
    }

    fn set(&self, key: &StoreKey, value: &SecretString) -> Result<()> {
        let entry = self.entry(&key.scope, key.name.as_str())?;
        entry
            .set_password(value.expose_secret())
            .map_err(|e| map_error("set", e))?;

        let mut index = self.read_index(&key.scope)?;
        if index.insert(key.name.clone()) {
            self.write_index(&key.scope, &index)?;
        }
        Ok(())
    }

    fn delete(&self, key: &StoreKey) -> Result<bool> {
        let entry = self.entry(&key.scope, key.name.as_str())?;
        let existed = match entry.delete_credential() {
            Ok(()) => true,
            Err(keyring::Error::NoEntry) => false,
            Err(e) => return Err(map_error("delete", e)),
        };

        let mut index = self.read_index(&key.scope)?;
        if index.remove(&key.name) {
            self.write_index(&key.scope, &index)?;
        }
        Ok(existed)
    }

    fn list(&self, scope: &Scope) -> Result<Vec<SecretName>> {
        Ok(self.read_index(scope)?.into_iter().collect())
    }
}

fn map_error(operation: &'static str, e: keyring::Error) -> Error {
    match e {
        keyring::Error::NoStorageAccess(inner) => Error::StoreUnavailable {
            backend: "os-keyring".to_string(),
            message: inner.to_string(),
        },
        other => Error::Store {
            operation,
            message: describe(other),
        },
    }
}

fn describe(e: keyring::Error) -> String {
    match e {
        keyring::Error::NoEntry => "no such entry".to_string(),
        keyring::Error::BadEncoding(_) => {
            "the stored value is not valid UTF-8 (contents withheld)".to_string()
        }
        keyring::Error::TooLong(what, max) => {
            format!("`{what}` exceeds the {max} byte limit of this credential store")
        }
        keyring::Error::Invalid(attr, reason) => format!("invalid {attr}: {reason}"),
        keyring::Error::Ambiguous(creds) => {
            format!(
                "{} ambiguous matching credentials (contents withheld)",
                creds.len()
            )
        }
        keyring::Error::PlatformFailure(inner) | keyring::Error::NoStorageAccess(inner) => {
            inner.to_string()
        }
        other => format!("{other:?}"),
    }
}
