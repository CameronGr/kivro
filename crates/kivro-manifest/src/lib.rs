//! The `.kivro.toml` project manifest.
//!
//! The manifest is the *non-secret* half of the system: it is committed to
//! source control and says **what a developer needs**, while the OS credential
//! store holds **what the values are**. It must never contain a secret value.
//!
//! # Format stability
//!
//! Three mechanisms keep the format evolvable without breaking older or newer
//! tools:
//!
//! * `[meta] format = <u32>` — a hard compatibility gate. A manifest declaring
//!   a format newer than [`SUPPORTED_FORMAT`] is rejected outright rather than
//!   half-understood.
//! * `[meta] min_cli_version` — a soft gate for features that are additive to
//!   the format but require newer tooling.
//! * Unknown keys and unknown sections are **ignored, not rejected**, and
//!   collected into [`Manifest::unknown_keys`] so `secrets doctor` can surface
//!   them as advisory warnings. This is what lets a newer manifest keep working
//!   with an older CLI when the addition is genuinely optional.
//!
//! # Variables vs settings
//!
//! Inside `[environments.<name>]`, keys are disambiguated by case:
//! `UPPER_SNAKE_CASE` keys are variable declarations, lowercase keys are
//! settings. Secret names are validated as `[A-Z_][A-Z0-9_]*`, so the two sets
//! can never overlap and new settings keys can be added later without any risk
//! of shadowing a user's variable.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod discovery;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use kivro_core::{EnvironmentName, Error, ProjectName, Result, SecretName};
use serde::Deserialize;

pub use discovery::{discover, discover_from};

/// the manifest filename
pub const MANIFEST_FILENAME: &str = ".kivro.toml";

/// the highest meta format value that this build understands
pub const SUPPORTED_FORMAT: u32 = 1;

/// declration of a single variable, doesnt carry vaue
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VariableSpec {
    /// is variable required for project to run
    pub required: bool,
    /// description shown by status
    pub description: Option<String>,
    /// example value, dont put a secret in here
    pub example: Option<String>,
    /// doctor command will warn you of depricated secrets
    pub depricated: bool,
}

/// where `secrets sync` should look for missing secrets
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncConfig {
    /// backend descriminator
    pub kind: String,
    /// backend specific settings
    pub settings: BTreeMap<String, String>,
}

/// a parsed and validated manifest
#[derive(Debug, Clone)]
pub struct Manifest {
    /// path of the manifest file
    pub path: PathBuf,
    /// directory containing the manifest
    pub root: PathBuf,
    /// Project identity
    pub project: ProjectName,
    /// Declared format version
    pub format: u32,
    /// Minimum CLI version required
    pub min_cli_version: Option<String>,
    /// Environment used when nothing else selects one
    pub default_environment: Option<EnvironmentName>,
    /// explicitly declared environment list
    pub declared_environments: Vec<EnvironmentName>,
    /// whether undeclared environments are rejected
    pub strict_environments: bool,
    /// variables that apply to every environment
    pub base_variables: BTreeMap<SecretName, VariableSpec>,
    /// per environment declarations
    pub environment_variables: BTreeMap<EnvironmentName, BTreeMap<SecretName, VariableSpec>>,
    /// optional sync backend configuration
    pub sync: Option<SyncConfig>,
    /// Keys this build did not recognise
    pub unknown_keys: Vec<String>,
}

/// a manifest reduced to a single environment
#[derive(Debug, Clone)]
pub struct ResolvedEnvironment {
    /// the project this is for
    pub project: ProjectName,
    /// the selected environment
    pub environment: EnvironmentName,
    /// effective variable set
    pub variables: BTreeMap<SecretName, VariableSpec>,
}

impl ResolvedEnvironment {
    /// Names of required variables
    pub fn required(&self) -> Vec<SecretName> {
        self.variables
            .iter()
            .filter(|(_, s)| s.required)
            .map(|(n, _)| n.clone())
            .collect()
    }

    /// Names of optional variables
    pub fn optional(&self) -> Vec<SecretName> {
        self.variables
            .iter()
            .filter(|(_, s)| !s.required)
            .map(|(n, _)| n.clone())
            .collect()
    }

    /// Every declared name
    pub fn names(&self) -> Vec<SecretName> {
        self.variables.keys().cloned().collect()
    }
}

impl Manifest {
    /// Read and parse a manifest from disk
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text =
            std::fs::read_to_string(path).map_err(|e| Error::io("read manifest", path, e))?;
        Self::parse(path, &text)
    }

    /// discover the nearest manifest by walking up from start dir and parse it
    pub fn discover_from(start: impl AsRef<Path>) -> Result<Self> {
        Self::load(discovery::discover_from(start.as_ref())?)
    }

    /// discover the nearest manifest from the current directory and parse it
    pub fn discover() -> Result<Self> {
        Self::load(discovery::discover()?)
    }

    /// parse manifest text
    pub fn parse(path: impl AsRef<Path>, text: &str) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let invalid = |message: String| Error::ManifestInvalid {
            path: path.clone(),
            message,
        };

        let raw: RawManifest =
            toml::from_str(text).map_err(|e| invalid(e.message().to_string()))?;
        let mut unknown_keys: Vec<String> = raw.extra.keys().cloned().collect();

        if raw.meta.format > SUPPORTED_FORMAT {
            return Err(Error::ManifestTooNew {
                path,
                found: raw.meta.format,
                supported: SUPPORTED_FORMAT,
            });
        }

        let name = raw
            .project
            .name
            .ok_or_else(|| invalid("missing `[project] name`".to_string()))?;
        let project = ProjectName::new(name).map_err(|e| invalid(e.to_string()))?;

        let base_variables = parse_variables("variables", &raw.variables, &mut unknown_keys)
            .map_err(|e| invalid(e))?;

        let mut environment_variables = BTreeMap::new();
        for (env_name, value) in &raw.environments {
            let env = EnvironmentName::new(env_name.clone()).map_err(|e| invalid(e.to_string()))?;
            let table = value
                .as_table()
                .ok_or_else(|| invalid(format!("`[environments.{env_name}]` must be a table")))?;

            let mut vars: BTreeMap<String, RawVariableSpec> = BTreeMap::new();
            for (key, val) in table {
                match key.as_str() {
                    "description" | "inherit" => {}
                    "variables" => {
                        let nested: BTreeMap<String, RawVariableSpec> =
                            val.clone().try_into().map_err(|e| {
                                invalid(format!("`[environments.{env_name}.variables]`: {e}"))
                            })?;
                        vars.extend(nested);
                    }
                    other
                        if other
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_uppercase() || c == '_') =>
                    {
                        let spec: RawVariableSpec = val.clone().try_into().map_err(|e| {
                            invalid(format!("`{other}` in `[environments.{env_name}]`: {e}"))
                        })?;
                        vars.insert(other.to_string(), spec);
                    }
                    other => unknown_keys.push(format!("environments.{env_name}.{other}")),
                }
            }

            let parsed = parse_variables(
                &format!("environments.{env_name}"),
                &vars,
                &mut unknown_keys,
            )
            .map_err(|e| invalid(e))?;
            environment_variables.insert(env, parsed);
        }

        let default_environment = match raw.environment.default {
            Some(d) => Some(EnvironmentName::new(d).map_err(|e| invalid(e.to_string()))?),
            None => None,
        };

        let mut declared_environments = Vec::new();
        for name in &raw.environment.list {
            declared_environments
                .push(EnvironmentName::new(name.clone()).map_err(|e| invalid(e.to_string()))?);
        }

        let sync = raw.sync.map(|s| SyncConfig {
            kind: s.kind,
            settings: s.settings,
        });

        let manifest = Manifest {
            root: path.parent().unwrap_or(Path::new(".")).to_path_buf(),
            path,
            project,
            format: raw.meta.format,
            min_cli_version: raw.meta.min_cli_version,
            default_environment,
            declared_environments,
            strict_environments: raw.environment.strict.unwrap_or(true),
            base_variables,
            environment_variables,
            sync,
            unknown_keys,
        };

        if let Some(default) = &manifest.default_environment {
            if !manifest.declared_environments.is_empty()
                && !manifest.declared_environments.contains(default)
            {
                return Err(Error::ManifestInvalid {
                    path: manifest.path.clone(),
                    message: format!(
                        "default environment `{default}` is not in `[environment] list`"
                    ),
                });
            }
        }
        for env in manifest.environment_variables.keys() {
            if !manifest.declared_environments.is_empty()
                && !manifest.declared_environments.contains(env)
            {
                return Err(Error::ManifestInvalid {
                    path: manifest.path.clone(),
                    message: format!("`[environments.{env}]` is not in `[environment] list`"),
                });
            }
        }

        Ok(manifest)
    }

    /// Every environment the manifest knows about
    pub fn environments(&self) -> Vec<EnvironmentName> {
        let mut set: BTreeSet<EnvironmentName> =
            self.environment_variables.keys().cloned().collect();
        set.extend(self.declared_environments.iter().cloned());
        if let Some(default) = &self.default_environment {
            set.insert(default.clone());
        }
        set.into_iter().collect()
    }

    /// Reduce the manifest to one environment
    pub fn resolve(&self, environment: &EnvironmentName) -> Result<ResolvedEnvironment> {
        let declared = self.environments();
        if self.strict_environments && !declared.is_empty() && !declared.contains(environment) {
            return Err(Error::UnknownEnvironment {
                name: environment.to_string(),
                available: declared.iter().map(|e| e.to_string()).collect(),
            });
        }

        let mut variables = self.base_variables.clone();
        if let Some(overrides) = self.environment_variables.get(environment) {
            for (name, spec) in overrides {
                variables.insert(name.clone(), spec.clone());
            }
        }

        Ok(ResolvedEnvironment {
            project: self.project.clone(),
            environment: environment.clone(),
            variables,
        })
    }

    /// check min_cli_version against the running version
    pub fn check_cli_version(&self, running: &str) -> Result<()> {
        let Some(required) = &self.min_cli_version else {
            return Ok(());
        };
        if version_tuple(required) > version_tuple(running) {
            return Err(Error::CliTooOld {
                path: self.path.clone(),
                required: required.clone(),
                running: running.to_string(),
            });
        }
        Ok(())
    }

    /// Render a starter manifest for `secrets init`
    pub fn template(project: &ProjectName, environment: &EnvironmentName) -> String {
        format!(
            "# Managed by `secrets`. This file is safe to commit.\n\
             # It declares WHICH secrets this project needs, never their values.\n\
             \n\
             [meta]\n\
             format = {SUPPORTED_FORMAT}\n\
             \n\
             [project]\n\
             name = \"{project}\"\n\
             \n\
             [environment]\n\
             default = \"{environment}\"\n\
             \n\
             # Variables that apply to every environment.\n\
             [variables]\n\
             # DATABASE_URL = {{ required = true, description = \"Primary Postgres DSN\" }}\n\
             \n\
             # Per-environment declarations override the ones above.\n\
             # [environments.{environment}]\n\
             # DATABASE_URL = {{ required = true }}\n\
             \n\
             # [environments.production]\n\
             # DATABASE_URL = {{ required = true }}\n"
        )
    }
}

fn version_tuple(v: &str) -> (u64, u64, u64) {
    let mut parts = v.trim().trim_start_matches('v').split(['.', '-', '+']);
    let mut next = || {
        parts
            .next()
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(0)
    };
    (next(), next(), next())
}

fn parse_variables(
    section: &str,
    raw: &BTreeMap<String, RawVariableSpec>,
    unknown_keys: &mut Vec<String>,
) -> std::result::Result<BTreeMap<SecretName, VariableSpec>, String> {
    let mut out = BTreeMap::new();
    for (name, spec) in raw {
        if name.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
            unknown_keys.push(format!("{section}.{name}"));
            continue;
        }
        let name = SecretName::new(name.clone()).map_err(|e| format!("in `[{section}]`: {e}"))?;
        out.insert(name, spec.clone().into_spec());
    }
    Ok(out)
}

#[derive(Debug, Deserialize, Default)]
struct RawManifest {
    #[serde(default)]
    meta: RawMeta,
    #[serde(default)]
    project: RawProject,
    #[serde(default)]
    environment: RawEnvironmentDefaults,
    #[serde(default)]
    variables: BTreeMap<String, RawVariableSpec>,
    #[serde(default)]
    environments: BTreeMap<String, toml::Value>,
    #[serde(default)]
    sync: Option<RawSync>,
    #[serde(flatten)]
    extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct RawMeta {
    format: u32,
    min_cli_version: Option<String>,
}

impl Default for RawMeta {
    fn default() -> Self {
        Self {
            format: 1,
            min_cli_version: None,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawProject {
    name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawEnvironmentDefaults {
    default: Option<String>,
    strict: Option<bool>,
    list: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawSync {
    kind: String,
    #[serde(flatten, default)]
    settings: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawVariableSpec {
    /// `VAR = true` — shorthand for `{ required = true }`
    Required(bool),
    Table {
        #[serde(default)]
        required: Option<bool>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        example: Option<String>,
        #[serde(default)]
        deprecated: Option<bool>,
    },
}

impl RawVariableSpec {
    fn into_spec(self) -> VariableSpec {
        match self {
            RawVariableSpec::Required(required) => VariableSpec {
                required,
                ..Default::default()
            },
            RawVariableSpec::Table {
                required,
                description,
                example,
                deprecated,
            } => VariableSpec {
                required: required.unwrap_or(true),
                description,
                example,
                depricated: deprecated.unwrap_or(false),
            },
        }
    }
}
