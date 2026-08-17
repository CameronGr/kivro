//! error types for kivro

use std::path::PathBuf;

/// convience result
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum Error {
    #[error("no `{filename}` found in `{start}` or any parent dir")]
    ManifestNotFound {
        filename: &'static str,
        start: PathBuf,
    },

    #[error("invalid manifest `{path}`: {message}")]
    ManifestInvalid { path: PathBuf, message: String },

    #[error(
        "manifest `{path}` requires format version {found}, but this build supports at most {supported}"
    )]
    ManifestTooNew {
        path: PathBuf,
        found: u32,
        supported: u32,
    },

    #[error("manifest `{path}` requires secrets {required} or newer (running {running})")]
    CliTooOld {
        path: PathBuf,
        required: String,
        running: String,
    },

    #[error("invalid {kind} `{value}`: {reason}")]
    InvalidName {
        kind: &'static str,
        value: String,
        reason: String,
    },

    #[error("unknown environment `{name}` (declared: {})", available.join(", "))]
    UnknownEnvironment {
        name: String,
        available: Vec<String>,
    },

    #[error("no environment selected and the manifest declares no default")]
    NoEnvironment,

    #[error("required secret `{name}` is missing for {project}/{environment}")]
    MissingSecret {
        name: String,
        project: String,
        environment: String,
    },

    #[error("{} required secret(s) missing for {project}/{environment}: {}", names.len(), names.join(", "))]
    MissingSecrets {
        names: Vec<String>,
        project: String,
        environment: String,
    },

    #[error("secret storage backend `{backend}` is unavailable: {message}")]
    StoreUnavailable { backend: String, message: String },

    #[error("secret storage error during `{operation}`: {message}")]
    Store {
        operation: &'static str,
        message: String,
    },

    #[error("cryptographic operation failed: {message}")]
    Crypto { message: String },

    #[error("invalid secret bundle: {message}")]
    BundleFormat { message: String },

    #[error("bundle mismatch: {message}")]
    BundleMismatch { message: String },

    #[error("cannot parse `{path}` at line {line}: {message}")]
    EnvFormat {
        path: PathBuf,
        line: usize,
        message: String,
    },

    #[error("sync error: {message}")]
    Sync { message: String },

    #[error("invalid configuration `{path}`: {message}")]
    Config { path: PathBuf, message: String },

    #[error("cancelled")]
    Cancelled,

    #[error("`{path}` already exists")]
    AlreadyExists { path: PathBuf },

    #[error("{operation} `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(transparent)]
    RawIo(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl Error {
    /// attach a path and operation to a std error
    pub fn io(operation: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            operation,
            path: path.into(),
            source,
        }
    }

    /// machine stable id for output
    pub fn kind(&self) -> &'static str {
        match self {
            Error::ManifestNotFound { .. } => "manifest_not_found",
            Error::ManifestInvalid { .. } => "manifest_invalid",
            Error::ManifestTooNew { .. } => "manifest_too_new",
            Error::CliTooOld { .. } => "cli_too_old",
            Error::InvalidName { .. } => "invalid_name",
            Error::UnknownEnvironment { .. } => "unknown_environment",
            Error::NoEnvironment => "no_environment",
            Error::MissingSecret { .. } | Error::MissingSecrets { .. } => "missing_secret",
            Error::StoreUnavailable { .. } => "store_unavailable",
            Error::Store { .. } => "store_error",
            Error::Crypto { .. } => "crypto_error",
            Error::BundleFormat { .. } => "bundle_format",
            Error::BundleMismatch { .. } => "bundle_mismatch",
            Error::EnvFormat { .. } => "env_format",
            Error::Sync { .. } => "sync_error",
            Error::Config { .. } => "config_invalid",
            Error::Cancelled => "cancelled",
            Error::AlreadyExists { .. } => "already_exists",
            Error::Io { .. } | Error::RawIo(_) => "io_error",
            Error::Other(_) => "error",
        }
    }

    /// hints for the user
    pub fn hint(&self) -> Option<String> {
        match self {
            Error::ManifestNotFound { .. } => {
                Some("run `kivro init` in your project root to create one".into())
            }
            Error::MissingSecret { name, .. } => Some(format!("run `kivro set {name}`")),
            Error::MissingSecrets { names, .. } => Some(format!(
                "run:\n{}",
                names.iter().map(|n| format!("    kivro set {n}")).collect::<Vec<_>>().join("\n")
            )),
            Error::UnknownEnvironment { available, .. } if !available.is_empty() => {
                Some(format!("declared environments: {}", available.join(", ")))
            }
            Error::NoEnvironment => Some(
                "pass `--env <name>`, set KIVRO_ENV, or add `[environment] default = \"dev\"` to .kivro.toml"
                    .into(),
            ),
            Error::StoreUnavailable { .. } => Some("run `kivro doctor` for details".into()),
            Error::CliTooOld { .. } | Error::ManifestTooNew { .. } => {
                Some("upgrade the `kivro` CLI".into())
            }
            Error::AlreadyExists { .. } => Some("pass `--force` to overwrite".into()),
            _ => None,
        }
    }
}
