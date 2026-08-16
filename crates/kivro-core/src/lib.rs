//! core model for kivro

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod names;
pub mod secret;
pub mod store;

pub use error::{Error, Result};
pub use names::{EnvironmentName, ProjectName, SecretName};
pub use secret::SecretString;
pub use store::{DEFAULT_APP_NAMESPACE, MemoryStore, Scope, SecretStore, StoreKey};
