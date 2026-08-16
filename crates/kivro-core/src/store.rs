//! storage module

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::error::Result;
use crate::names::{EnvironmentName, ProjectName, SecretName};
use crate::secret::SecretString;

/// default application namespace, can be set in global config
pub const DEFAULT_APP_NAMESPACE: &str = "kivro-secrets";

/// project + enviroment that a secret belongs to
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scope {
    /// the owning project
    pub project: ProjectName,
    /// environment withing the project
    pub environment: EnvironmentName,
}

impl Scope {
    /// construct a scope
    pub fn new(project: ProjectName, environment: EnvironmentName) -> Self {
        Self {
            project,
            environment,
        }
    }

    /// the service ident that we pass to the OS store
    pub fn service_id(&self, app_namespace: &str) -> String {
        format!("{}:{}:{}", app_namespace, self.project, self.environment)
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.project, self.environment)
    }
}

/// full address for one secret
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoreKey {
    /// project + environment
    pub scope: Scope,
    /// var name
    pub name: SecretName,
}

impl StoreKey {
    /// create a new store key
    pub fn new(scope: Scope, name: SecretName) -> Self {
        Self { scope, name }
    }
}

impl std::fmt::Display for StoreKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.scope, self.name)
    }
}

/// a place secrets can be stored, its important not to expose any secrets via error messages in the implemented struct
pub trait SecretStore: Send + Sync {
    /// backend id i.e. "keyring"
    fn backend(&self) -> &str;

    /// wether the backend is usable on this machine
    fn check_available(&self) -> Result<()> {
        Ok(())
    }

    /// wether values in this backend are protected bt the OS. will be false if using memory store
    fn is_secure(&self) -> bool {
        true
    }

    /// fetch one secret if it exists
    fn get(&self, key: &StoreKey) -> Result<Option<SecretString>>;

    /// store one secret, overwrites any existing value
    fn set(&self, key: &StoreKey, value: &SecretString) -> Result<()>;

    /// remove one secret, returns wether it was removed
    fn delete(&self, key: &StoreKey) -> Result<bool>;

    /// names of every secret in the scope
    fn list(&self, scope: &Scope) -> Result<Vec<SecretName>>;
}

/// in memory store for testing and callers that dont want to touch OS keyring
#[derive(Default)]
pub struct MemoryStore {
    inner: Mutex<BTreeMap<StoreKey, SecretString>>,
}

impl MemoryStore {
    /// new empry store
    pub fn new() -> Self {
        Self::default()
    }

    /// number of secrets in all scopes
    pub fn len(&self) -> usize {
        self.inner.lock().expect("memory store locked").len()
    }

    /// store is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl SecretStore for MemoryStore {
    fn backend(&self) -> &str {
        "memory"
    }

    fn is_secure(&self) -> bool {
        false
    }

    fn get(&self, key: &StoreKey) -> Result<Option<SecretString>> {
        Ok(self
            .inner
            .lock()
            .expect("memory store locked")
            .get(key)
            .cloned())
    }

    fn set(&self, key: &StoreKey, value: &SecretString) -> Result<()> {
        self.inner
            .lock()
            .expect("memory store locked")
            .insert(key.clone(), value.clone());
        Ok(())
    }

    fn delete(&self, key: &StoreKey) -> Result<bool> {
        Ok(self
            .inner
            .lock()
            .expect("memory store locked")
            .remove(key)
            .is_some())
    }

    fn list(&self, scope: &Scope) -> Result<Vec<SecretName>> {
        Ok(self
            .inner
            .lock()
            .expect("memory store locked")
            .keys()
            .filter(|k| &k.scope == scope)
            .map(|k| k.name.clone())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(project: &str, env: &str) -> Scope {
        Scope::new(
            ProjectName::new(project).unwrap(),
            EnvironmentName::new(env).unwrap(),
        )
    }

    fn key(project: &str, env: &str, name: &str) -> StoreKey {
        StoreKey::new(scope(project, env), SecretName::new(name).unwrap())
    }

    #[test]
    fn service_id_is_stable_and_namespaced() {
        assert_eq!(
            scope("infinity-launcher", "dev").service_id("kivro-secrets"),
            "kivro-secrets:infinity-launcher:dev"
        );
    }

    #[test]
    fn distinct_scopes_never_collide() {
        let ids = [
            scope("a", "dev").service_id("app"),
            scope("b", "dev").service_id("app"),
            scope("a", "prod").service_id("app"),
        ];
        let unique: std::collections::BTreeSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len());
    }

    #[test]
    fn memory_store_round_trip() {
        let store = MemoryStore::new();
        let k = key("proj", "dev", "DATABASE_URL");
        assert!(store.get(&k).unwrap().is_none());
        store.set(&k, &SecretString::new("postgres://x")).unwrap();
        assert_eq!(
            store.get(&k).unwrap().unwrap().expose_secret(),
            "postgres://x"
        );
        assert_eq!(store.list(&scope("proj", "dev")).unwrap().len(), 1);
        assert!(store.list(&scope("proj", "prod")).unwrap().is_empty());
        assert!(store.delete(&k).unwrap());
        assert!(!store.delete(&k).unwrap());
    }

    #[test]
    fn same_name_in_two_projects_is_isolated() {
        let store = MemoryStore::new();
        store
            .set(&key("a", "dev", "DATABASE_URL"), &SecretString::new("one"))
            .unwrap();
        store
            .set(&key("b", "dev", "DATABASE_URL"), &SecretString::new("two"))
            .unwrap();
        assert_eq!(
            store
                .get(&key("a", "dev", "DATABASE_URL"))
                .unwrap()
                .unwrap()
                .expose_secret(),
            "one"
        );
        assert_eq!(
            store
                .get(&key("b", "dev", "DATABASE_URL"))
                .unwrap()
                .unwrap()
                .expose_secret(),
            "two"
        );
    }
}
