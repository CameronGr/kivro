//! an insecure file-based store, purely for testing and sandbox ci only

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use kivro_core::{Error, Result, Scope, SecretName, SecretStore, SecretString, StoreKey};

/// Plaintext json store, dont use me for production
pub struct FileStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl FileStore {
    /// Open a store
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            lock: Mutex::new(()),
        }
    }

    fn entry_key(key: &StoreKey) -> String {
        format!(
            "{}:{}:{}",
            key.scope.project, key.scope.environment, key.name
        )
    }

    fn read(&self) -> Result<BTreeMap<String, String>> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => serde_json::from_str(&text).map_err(|e| Error::Store {
                operation: "get",
                message: format!("corrupt store file `{}`: {e}", self.path.display()),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(e) => Err(Error::io("read secret store", &self.path, e)),
        }
    }

    fn write(&self, data: &BTreeMap<String, String>) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Error::io("create store directory", parent, e))?;
            }
        }
        let text = serde_json::to_string_pretty(data).map_err(|e| Error::Store {
            operation: "set",
            message: e.to_string(),
        })?;
        write_private(&self.path, text.as_bytes())
    }
}

/// Write a file that is readable only by the current user
pub(crate) fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;

    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|e| Error::io("create", path, e))?;
    file.write_all(bytes)
        .map_err(|e| Error::io("write", path, e))?;
    file.flush().map_err(|e| Error::io("write", path, e))?;
    Ok(())
}

impl SecretStore for FileStore {
    fn backend(&self) -> &str {
        "file (insecure)"
    }

    fn is_secure(&self) -> bool {
        false
    }

    fn get(&self, key: &StoreKey) -> Result<Option<SecretString>> {
        let _guard = self.lock.lock().expect("file store poisoned");
        Ok(self
            .read()?
            .get(&Self::entry_key(key))
            .map(SecretString::new))
    }

    fn set(&self, key: &StoreKey, value: &SecretString) -> Result<()> {
        let _guard = self.lock.lock().expect("file store poisoned");
        let mut data = self.read()?;
        data.insert(Self::entry_key(key), value.expose_secret().to_string());
        self.write(&data)
    }

    fn delete(&self, key: &StoreKey) -> Result<bool> {
        let _guard = self.lock.lock().expect("file store poisoned");
        let mut data = self.read()?;
        let existed = data.remove(&Self::entry_key(key)).is_some();
        if existed {
            self.write(&data)?;
        }
        Ok(existed)
    }

    fn list(&self, scope: &Scope) -> Result<Vec<SecretName>> {
        let _guard = self.lock.lock().expect("file store poisoned");
        let prefix = format!("{}:{}:", scope.project, scope.environment);
        Ok(self
            .read()?
            .keys()
            .filter_map(|k| k.strip_prefix(&prefix))
            .filter_map(|n| SecretName::new(n).ok())
            .collect())
    }
}
