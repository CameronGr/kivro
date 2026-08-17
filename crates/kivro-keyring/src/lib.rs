//! credential store adapters for kivro

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod file_store;
#[cfg(feature = "os-keyring")]
mod keyring_store;

use std::path::PathBuf;

use kivro_core::{Error, MemoryStore, Result, SecretStore};

pub use file_store::FileStore;
#[cfg(feature = "os-keyring")]
pub use keyring_store::KeyringStore;

/// Environment variable selecting a non-default backend
pub const STORE_ENV: &str = "KIVRO_STORE";
/// Environment variable giving the path used by [`FileStore`]
pub const STORE_FILE_ENV: &str = "KIVRO_STORE_FILE";

/// which backend to open
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreKind {
    /// the OS cred store
    OsKeyring,
    /// plaintext json, for testing only
    File(PathBuf),
    /// in process memory
    Memory,
}

impl StoreKind {
    /// Resolve the backend from the process environment
    pub fn from_env() -> Result<Self> {
        match std::env::var(STORE_ENV).ok().as_deref() {
            None | Some("") | Some("keyring") | Some("os") => Ok(StoreKind::OsKeyring),
            Some("memory") => Ok(StoreKind::Memory),
            Some("file") => {
                let path = std::env::var(STORE_FILE_ENV).map_err(|_| {
                    Error::Other(format!(
                        "{STORE_ENV}=file requires {STORE_FILE_ENV} to point at a writable path"
                    ))
                })?;
                Ok(StoreKind::File(PathBuf::from(path)))
            }
            Some(other) => Err(Error::Other(format!(
                "unknown {STORE_ENV} value `{other}` (expected `keyring`, `file` or `memory`)"
            ))),
        }
    }
}

/// Open a store of the given kind
pub fn open(kind: StoreKind, app_namespace: &str) -> Result<Box<dyn SecretStore>> {
    match kind {
        StoreKind::OsKeyring => {
            #[cfg(feature = "os-keyring")]
            {
                let _ = app_namespace;
                Ok(Box::new(KeyringStore::new(app_namespace)))
            }
            #[cfg(not(feature = "os-keyring"))]
            {
                let _ = app_namespace;
                Err(Error::StoreUnavailable {
                    backend: "os-keyring".into(),
                    message: "this build was compiled without the `os-keyring` feature".into(),
                })
            }
        }
        StoreKind::File(path) => Ok(Box::new(FileStore::new(path))),
        StoreKind::Memory => Ok(Box::new(MemoryStore::new())),
    }
}

/// Open the backend selected by the environment.
pub fn open_from_env(app_namespace: &str) -> Result<Box<dyn SecretStore>> {
    open(StoreKind::from_env()?, app_namespace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivro_core::{EnvironmentName, ProjectName, Scope, SecretName, SecretString, StoreKey};

    fn key(project: &str, env: &str, name: &str) -> StoreKey {
        StoreKey::new(
            Scope::new(
                ProjectName::new(project).unwrap(),
                EnvironmentName::new(env).unwrap(),
            ),
            SecretName::new(name).unwrap(),
        )
    }

    #[test]
    fn file_store_round_trips_and_isolates_scopes() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileStore::new(dir.path().join("store.json"));

        store
            .set(&key("a", "dev", "DATABASE_URL"), &SecretString::new("one"))
            .unwrap();
        store
            .set(&key("b", "dev", "DATABASE_URL"), &SecretString::new("two"))
            .unwrap();
        store
            .set(
                &key("a", "prod", "DATABASE_URL"),
                &SecretString::new("three"),
            )
            .unwrap();

        assert_eq!(
            store
                .get(&key("a", "dev", "DATABASE_URL"))
                .unwrap()
                .unwrap()
                .expose_secret(),
            "one"
        );
        let scope = Scope::new(
            ProjectName::new("a").unwrap(),
            EnvironmentName::new("dev").unwrap(),
        );
        assert_eq!(
            store.list(&scope).unwrap(),
            vec![SecretName::new("DATABASE_URL").unwrap()]
        );

        assert!(store.delete(&key("a", "dev", "DATABASE_URL")).unwrap());
        assert!(!store.delete(&key("a", "dev", "DATABASE_URL")).unwrap());
        assert!(
            store
                .get(&key("a", "dev", "DATABASE_URL"))
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .get(&key("b", "dev", "DATABASE_URL"))
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn file_store_is_flagged_insecure() {
        assert!(!FileStore::new("/tmp/nope.json").is_secure());
    }

    #[cfg(unix)]
    #[test]
    fn file_store_is_created_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.json");
        FileStore::new(&path)
            .set(&key("a", "dev", "A"), &SecretString::new("x"))
            .unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(
            mode & 0o077,
            0,
            "store file must not be group/world readable"
        );
    }

    #[test]
    fn unknown_backend_names_are_rejected_not_guessed() {
        temp_env("KIVRO_STORE", Some("keychain"), || {
            assert!(StoreKind::from_env().is_err());
        });
        temp_env("KIVRO_STORE", None, || {
            assert_eq!(StoreKind::from_env().unwrap(), StoreKind::OsKeyring);
        });
    }

    #[allow(unsafe_code)]
    fn temp_env(key: &str, value: Option<&str>, f: impl FnOnce()) {
        let previous = std::env::var(key).ok();
        match value {
            Some(v) => unsafe { std::env::set_var(key, v) },
            None => unsafe { std::env::remove_var(key) },
        }
        f();
        match previous {
            Some(v) => unsafe { std::env::set_var(key, v) },
            None => unsafe { std::env::remove_var(key) },
        }
    }
}
