//! The `kivro` library.
//!
//! This is the API the CLI is built on, and the one other internal tools should
//! use. Nothing CLI-specific (argument parsing, terminal formatting, prompting,
//! exit codes) appears here.
//!
//! ```no_run
//! use kivro::Project;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let project = Project::discover()?;
//! let kivro = project.environment("dev")?.load()?;
//! let database_url = kivro.get("DATABASE_URL")?;
//!
//! std::process::Command::new("cargo")
//!     .args(["run"])
//!     .envs(kivro.environment())
//!     .spawn()?;
//! # let _ = database_url;
//! # Ok(())
//! # }
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod envfile;
pub mod run;

use std::collections::BTreeMap;
use std::path::Path;

use kivro_core::{
    EnvironmentName, Error, ProjectName, Result, Scope, SecretName, SecretStore, SecretString,
    StoreKey,
};
use kivro_manifest::{Manifest, ResolvedEnvironment, VariableSpec};

pub use config::Config;
pub use kivro_core as core;
pub use kivro_core::{Error as SecretsError, SecretString as Secret};
pub use kivro_crypto as crypto;
pub use kivro_manifest as manifest;

/// Environment variable that overrides the manifest default environment
pub const ENV_OVERRIDE: &str = "KIVRO_ENV";

/// a discovered project
pub struct Project {
    manifest: Manifest,
    store: Box<dyn SecretStore>,
    config: Config,
}

impl Project {
    /// Discover the project containing the current directory.
    pub fn discover() -> Result<Self> {
        Self::discover_from(std::env::current_dir().map_err(Error::RawIo)?)
    }

    /// Discover the project containing `path`.
    pub fn discover_from(path: impl AsRef<Path>) -> Result<Self> {
        let manifest = Manifest::discover_from(path)?;
        let config = Config::load()?;
        let store = kivro_keyring::open_from_env(&config.storage.namespace)?;
        Ok(Self {
            manifest,
            store,
            config,
        })
    }

    /// Assemble a project from parts. Used by tests and by embedders that
    /// supply their own store.
    pub fn new(manifest: Manifest, store: Box<dyn SecretStore>, config: Config) -> Self {
        Self {
            manifest,
            store,
            config,
        }
    }

    /// The parsed manifest.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// The project name.
    pub fn name(&self) -> &ProjectName {
        &self.manifest.project
    }

    /// The backing store.
    pub fn store(&self) -> &dyn SecretStore {
        self.store.as_ref()
    }

    /// The loaded global configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Select an environment by name.
    pub fn environment(&self, name: &str) -> Result<Environment<'_>> {
        let name = EnvironmentName::new(name)?;
        let resolved = self.manifest.resolve(&name)?;
        Ok(Environment {
            project: self,
            resolved,
        })
    }

    /// Select an environment following the documented precedence:
    ///
    /// 1. `explicit` (the CLI's `--env`)
    /// 2. the `KIVRO_ENV` environment variable
    /// 3. `[environment] default` in the manifest
    /// 4. `[defaults] environment` in the global configuration
    /// 5. otherwise an error
    ///
    /// The manifest outranks global configuration deliberately: a per-machine
    /// preference must not silently change which environment a shared project
    /// runs against.
    pub fn resolve_environment(&self, explicit: Option<&str>) -> Result<Environment<'_>> {
        if let Some(name) = explicit {
            return self.environment(name);
        }
        if let Ok(name) = std::env::var(ENV_OVERRIDE) {
            if !name.is_empty() {
                return self.environment(&name);
            }
        }
        if let Some(default) = &self.manifest.default_environment {
            return self.environment(default.as_str());
        }
        if let Some(name) = &self.config.defaults.environment {
            return self.environment(name);
        }
        Err(Error::NoEnvironment)
    }
}

/// One environment of one project.
///
/// `Debug` renders identity only; it holds a store handle, never values.
pub struct Environment<'a> {
    project: &'a Project,
    resolved: ResolvedEnvironment,
}

impl std::fmt::Debug for Environment<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Environment")
            .field("project", &self.resolved.project)
            .field("environment", &self.resolved.environment)
            .finish()
    }
}

impl<'a> Environment<'a> {
    /// The environment name.
    pub fn name(&self) -> &EnvironmentName {
        &self.resolved.environment
    }

    /// The owning project name.
    pub fn project_name(&self) -> &ProjectName {
        &self.resolved.project
    }

    /// The storage scope this environment addresses.
    pub fn scope(&self) -> Scope {
        Scope::new(
            self.resolved.project.clone(),
            self.resolved.environment.clone(),
        )
    }

    /// Declared variables for this environment.
    pub fn declarations(&self) -> &BTreeMap<SecretName, VariableSpec> {
        &self.resolved.variables
    }

    fn key(&self, name: &SecretName) -> StoreKey {
        StoreKey::new(self.scope(), name.clone())
    }

    /// Store a value.
    pub fn set(&self, name: &SecretName, value: &SecretString) -> Result<()> {
        self.project.store.set(&self.key(name), value)
    }

    /// Fetch one value, if present.
    pub fn get(&self, name: &SecretName) -> Result<Option<SecretString>> {
        self.project.store.get(&self.key(name))
    }

    /// Delete one value. Returns whether it existed.
    pub fn remove(&self, name: &SecretName) -> Result<bool> {
        self.project.store.delete(&self.key(name))
    }

    /// Names present in the store, including ones the manifest does not declare.
    pub fn stored_names(&self) -> Result<Vec<SecretName>> {
        let mut names = self.project.store.list(&self.scope())?;
        // The store's index can lag; declared names are probed directly so
        // `status` is correct even if enumeration is not.
        for name in self.resolved.variables.keys() {
            if !names.contains(name) && self.get(name)?.is_some() {
                names.push(name.clone());
            }
        }
        names.sort();
        names.dedup();
        Ok(names)
    }

    /// Per-variable presence, for `status`, `list` and `doctor`.
    pub fn status(&self) -> Result<EnvironmentStatus> {
        let stored = self.stored_names()?;
        let mut entries = Vec::new();
        for (name, spec) in &self.resolved.variables {
            entries.push(SecretStatus {
                name: name.clone(),
                required: spec.required,
                present: stored.contains(name) || self.get(name)?.is_some(),
                declared: true,
                deprecated: spec.depricated,
                description: spec.description.clone(),
            });
        }
        for name in stored {
            if !self.resolved.variables.contains_key(&name) {
                entries.push(SecretStatus {
                    name,
                    required: false,
                    present: true,
                    declared: false,
                    deprecated: false,
                    description: None,
                });
            }
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(EnvironmentStatus {
            project: self.resolved.project.clone(),
            environment: self.resolved.environment.clone(),
            entries,
        })
    }

    /// Load every declared secret, failing if a required one is missing.
    pub fn load(&self) -> Result<SecretSet> {
        let set = self.load_available()?;
        let missing: Vec<String> = self
            .resolved
            .variables
            .iter()
            .filter(|(name, spec)| spec.required && !set.values.contains_key(*name))
            .map(|(name, _)| name.to_string())
            .collect();

        if !missing.is_empty() {
            return Err(Error::MissingSecrets {
                names: missing,
                project: self.resolved.project.to_string(),
                environment: self.resolved.environment.to_string(),
            });
        }
        Ok(set)
    }

    /// Load whatever is present, without enforcing `required`.
    pub fn load_available(&self) -> Result<SecretSet> {
        let mut values = BTreeMap::new();
        for name in self.resolved.variables.keys() {
            if let Some(value) = self.get(name)? {
                values.insert(name.clone(), value);
            }
        }
        Ok(SecretSet {
            project: self.resolved.project.clone(),
            environment: self.resolved.environment.clone(),
            values,
        })
    }

    /// Load every stored secret in the scope, declared or not. Used by `share`.
    pub fn load_all_stored(&self) -> Result<SecretSet> {
        let mut values = BTreeMap::new();
        for name in self.stored_names()? {
            if let Some(value) = self.get(&name)? {
                values.insert(name, value);
            }
        }
        Ok(SecretSet {
            project: self.resolved.project.clone(),
            environment: self.resolved.environment.clone(),
            values,
        })
    }
}

/// Presence of one secret.
#[derive(Debug, Clone)]
pub struct SecretStatus {
    /// Variable name.
    pub name: SecretName,
    /// Whether the manifest requires it.
    pub required: bool,
    /// Whether a value is stored.
    pub present: bool,
    /// Whether the manifest declares it (`false` means stored but undeclared).
    pub declared: bool,
    /// Whether the manifest marks it deprecated.
    pub deprecated: bool,
    /// Manifest description, if any.
    pub description: Option<String>,
}

/// Presence of every secret in an environment.
#[derive(Debug, Clone)]
pub struct EnvironmentStatus {
    /// Project name.
    pub project: ProjectName,
    /// Environment name.
    pub environment: EnvironmentName,
    /// One entry per declared or stored secret.
    pub entries: Vec<SecretStatus>,
}

impl EnvironmentStatus {
    /// Required secrets with no stored value.
    pub fn missing_required(&self) -> Vec<&SecretStatus> {
        self.entries
            .iter()
            .filter(|e| e.required && !e.present)
            .collect()
    }

    /// Whether every required secret is present.
    pub fn is_satisfied(&self) -> bool {
        self.missing_required().is_empty()
    }

    /// Stored values with no matching declaration.
    pub fn undeclared(&self) -> Vec<&SecretStatus> {
        self.entries.iter().filter(|e| !e.declared).collect()
    }
}

/// Secret values loaded into memory for one environment.
pub struct SecretSet {
    project: ProjectName,
    environment: EnvironmentName,
    values: BTreeMap<SecretName, SecretString>,
}

impl SecretSet {
    /// Build a set directly, e.g. from a decrypted bundle.
    pub fn from_values(
        project: ProjectName,
        environment: EnvironmentName,
        values: BTreeMap<SecretName, SecretString>,
    ) -> Self {
        Self {
            project,
            environment,
            values,
        }
    }

    /// Fetch a value, erroring if it is absent.
    pub fn get(&self, name: &str) -> Result<&SecretString> {
        let key = SecretName::new(name)?;
        self.values.get(&key).ok_or_else(|| Error::MissingSecret {
            name: name.to_string(),
            project: self.project.to_string(),
            environment: self.environment.to_string(),
        })
    }

    /// Fetch a value if present.
    pub fn find(&self, name: &str) -> Option<&SecretString> {
        SecretName::new(name).ok().and_then(|n| self.values.get(&n))
    }

    /// The variables, ready for [`std::process::Command::envs`].
    ///
    /// This is the one place secrets leave the type system as plain strings,
    /// because the standard library's process API takes `AsRef<OsStr>`.
    pub fn environment(&self) -> Vec<(String, String)> {
        self.values
            .iter()
            .map(|(name, value)| (name.to_string(), value.expose_secret().to_string()))
            .collect()
    }

    /// Borrow the underlying map.
    pub fn values(&self) -> &BTreeMap<SecretName, SecretString> {
        &self.values
    }

    /// Consume the set, yielding the underlying map.
    pub fn into_values(self) -> BTreeMap<SecretName, SecretString> {
        self.values
    }

    /// The names held.
    pub fn names(&self) -> Vec<SecretName> {
        self.values.keys().cloned().collect()
    }

    /// Project the values belong to.
    pub fn project_name(&self) -> &ProjectName {
        &self.project
    }

    /// Environment the values belong to.
    pub fn environment_name(&self) -> &EnvironmentName {
        &self.environment
    }

    /// Number of values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl std::fmt::Debug for SecretSet {
    /// Renders names and counts only — never values.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretSet")
            .field("project", &self.project)
            .field("environment", &self.environment)
            .field(
                "names",
                &self.values.keys().map(|k| k.as_str()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
