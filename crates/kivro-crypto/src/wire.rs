//! the on disk representation of a bundle

use std::collections::BTreeMap;

use kivro_core::{EnvironmentName, ProjectName, SecretName, SecretString};
use serde::{Deserialize, Serialize};

/// outer envelope, everything it continas is unauthorized so it doesnt contain any sensitive data, it only serves to direct tooling before passphrase is available
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Envelope {
    pub magic: String,
    pub format: u32,
    pub cipher: String,
    #[serde(default)]
    pub hint: Hint,
    pub payload: String,
}

/// essentially metadata
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Hint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
}

/// The unauthenticated payload, searalized then encrypted
#[derive(Debug, Deserialize)]
pub(crate) struct PayloadIn {
    pub format: u32,
    pub project: ProjectName,
    pub environment: EnvironmentName,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub created_by: Option<String>,
    pub secrets: BTreeMap<SecretName, SecretString>,
}

/// the payload for the serializer
#[derive(Serialize)]
pub(crate) struct PayloadOut<'a> {
    pub format: u32,
    pub project: &'a ProjectName,
    pub environment: &'a EnvironmentName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<&'a str>,
    pub secrets: BTreeMap<&'a str, Exposed<'a>>,
}

pub(crate) struct Exposed<'a>(pub &'a SecretString);

impl Serialize for Exposed<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.expose_secret())
    }
}
