//! Encrypted secret bundles
//! # File layout
//!
//! ```text
//! {
//!   "magic":   "kivro-bundle",
//!   "format":  1,
//!   "cipher":  "age-v1-scrypt",
//!   "hint":    { ... unauthenticated routing metadata ... },
//!   "payload": "-----BEGIN AGE ENCRYPTED FILE----- ..."
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod wire;

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::str::FromStr;

use age::armor::{ArmoredReader, ArmoredWriter, Format};
use kivro_core::{EnvironmentName, Error, ProjectName, Result, SecretName, SecretString};
use zeroize::Zeroizing;

/// envelope magic string
pub const BUNDLE_MAGIC: &str = "kivro-bundle";
/// highest envelope format this build reads/writes
pub const BUNDLE_FORMAT: u32 = 1;
/// file extension for bundles
pub const BUNDLE_EXTENSION: &str = "kivro";

/// passphrase mode: age scrypr recipient
pub const CIPHER_SCRYPT: &str = "age-v1-scrypt";
/// pubic key mode: age x25519
pub const CIPHER_X25519: &str = "age-v1-x25519";

/// how a bundle should be encrypted
pub enum SealKey {
    /// human supplied passphrase
    Passphrase(SecretString),
    /// One or more age1 public keys
    Recipients(Vec<String>),
}

/// how a bundle should be decrypted
pub enum OpenKey {
    /// he passphrase used to seal it
    Passphrase(SecretString),
    /// one or more age identities
    Identities(Vec<SecretString>),
}

/// A bundles contents, decrypted
#[derive(Debug)]
pub struct Bundle {
    /// project the secrets belong to
    pub project: ProjectName,
    /// environment the secrets belong to
    pub environment: EnvironmentName,
    /// when the bundle was created
    pub created_at: Option<String>,
    /// creator label
    pub created_by: Option<String>,
    /// the secrets themselves
    pub secrets: BTreeMap<SecretName, SecretString>,
}

impl Bundle {
    /// create a bundle for project/environment
    pub fn new(
        project: ProjectName,
        environment: EnvironmentName,
        secrets: BTreeMap<SecretName, SecretString>,
    ) -> Self {
        Self {
            project,
            environment,
            created_at: None,
            created_by: None,
            secrets,
        }
    }

    /// attach the optional metadata
    pub fn with_metadata(mut self, created_at: Option<String>, created_by: Option<String>) -> Self {
        self.created_at = created_at;
        self.created_by = created_by;
        self
    }

    /// names carried by the bumdle
    pub fn names(&self) -> Vec<SecretName> {
        self.secrets.keys().cloned().collect()
    }
}

///options controlling what the unauthenicated hint discloses
#[derive(Debug, Clone, Copy)]
pub struct SealOptions {
    /// include project/environment in the header
    pub hint_identity: bool,
    /// include secret names in the hint, default to false since its inharently less safe this way
    pub hint_names: bool,
}

impl Default for SealOptions {
    fn default() -> Self {
        Self {
            hint_identity: true,
            hint_names: false,
        }
    }
}

/// encrypt a bundle, returns the text file
pub fn seal(bundle: &Bundle, key: &SealKey, options: SealOptions) -> Result<String> {
    let payload = wire::PayloadOut {
        format: BUNDLE_FORMAT,
        project: &bundle.project,
        environment: &bundle.environment,
        created_at: bundle.created_at.as_deref(),
        created_by: bundle.created_by.as_deref(),
        secrets: bundle
            .secrets
            .iter()
            .map(|(name, value)| (name.as_str(), wire::Exposed(value)))
            .collect(),
    };

    let plaintext = Zeroizing::new(serde_json::to_vec(&payload).map_err(|e| Error::Crypto {
        message: format!("cannot searalize payload: {e}"),
    })?);

    let (cipher, armored) = match key {
        SealKey::Passphrase(passphrase) => {
            if passphrase.is_empty() {
                return Err(Error::Crypto {
                    message: "passphrase must not be empty".into(),
                });
            }
            let encryptor = age::Encryptor::with_user_passphrase(age_secret(passphrase));
            (CIPHER_SCRYPT, encrypt_with(encryptor, &plaintext)?)
        }
        SealKey::Recipients(recipients) => {
            let parsed: Vec<age::x25519::Recipient> = recipients
                .iter()
                .map(|r| {
                    age::x25519::Recipient::from_str(r.trim()).map_err(|e| Error::Crypto {
                        message: format!("invalid recipient: {e}"),
                    })
                })
                .collect::<Result<_>>()?;
            if parsed.is_empty() {
                return Err(Error::Crypto {
                    message: "at least one recipient is required".into(),
                });
            }
            let encryptor =
                age::Encryptor::with_recipients(parsed.iter().map(|r| r as &dyn age::Recipient))
                    .map_err(|e| Error::Crypto {
                        message: e.to_string(),
                    })?;
            (CIPHER_X25519, encrypt_with(encryptor, &plaintext)?)
        }
    };

    let envelope = wire::Envelope {
        magic: BUNDLE_MAGIC.to_string(),
        format: BUNDLE_FORMAT,
        cipher: cipher.to_string(),
        hint: wire::Hint {
            project: options.hint_identity.then(|| bundle.project.to_string()),
            environment: options
                .hint_identity
                .then(|| bundle.environment.to_string()),
            created_at: bundle.created_at.clone(),
            created_by: bundle.created_by.clone(),
            names: options
                .hint_names
                .then(|| bundle.names().iter().map(|n| n.to_string()).collect()),
        },
        payload: armored,
    };

    serde_json::to_string_pretty(&envelope)
        .map(|mut s| {
            s.push('\n');
            s
        })
        .map_err(|e| Error::Crypto {
            message: e.to_string(),
        })
}

/// Decrypt a bundle
pub fn open(text: &str, key: &OpenKey) -> Result<Bundle> {
    let envelope: wire::Envelope = serde_json::from_str(text).map_err(|e| Error::BundleFormat {
        message: format!("not a secret bundle: {e}"),
    })?;

    if envelope.magic != BUNDLE_MAGIC {
        return Err(Error::BundleFormat {
            message: format!("unexpected magic `{}`", envelope.magic),
        });
    }
    if envelope.format > BUNDLE_FORMAT {
        return Err(Error::BundleFormat {
            message: format!(
                "bundle format {} is newer than the supported format {BUNDLE_FORMAT}; upgrade the `kivro` CLI",
                envelope.format
            ),
        });
    }

    let plaintext = match (envelope.cipher.as_str(), key) {
        (CIPHER_SCRYPT, OpenKey::Passphrase(passphrase)) => {
            decrypt_passphrase(&envelope.payload, passphrase)?
        }
        (CIPHER_X25519, OpenKey::Identities(identities)) => {
            decrypt_identities(&envelope.payload, identities)?
        }
        (CIPHER_SCRYPT, _) => {
            return Err(Error::Crypto {
                message: "this bundle needs a passphrase".into(),
            });
        }
        (CIPHER_X25519, _) => {
            return Err(Error::Crypto {
                message: "this bundle needs an age identity".into(),
            });
        }
        (other, _) => {
            return Err(Error::BundleFormat {
                message: format!(
                    "unsupported cipher `{other}` (this build supports `{CIPHER_SCRYPT}` and `{CIPHER_X25519}`)"
                ),
            });
        }
    };

    let payload: wire::PayloadIn =
        serde_json::from_slice(&plaintext).map_err(|e| Error::BundleFormat {
            message: format!("decrypted payload is malformed ({})", e.classify_str()),
        })?;

    if payload.format > BUNDLE_FORMAT {
        return Err(Error::BundleFormat {
            message: format!(
                "payload format {} is newer than {BUNDLE_FORMAT}",
                payload.format
            ),
        });
    }

    if let Some(project) = &envelope.hint.project {
        if project != payload.project.as_str() {
            return Err(Error::BundleMismatch {
                message: "the bundle header names a different project than its contents".into(),
            });
        }
    }
    if let Some(environment) = &envelope.hint.environment {
        if environment != payload.environment.as_str() {
            return Err(Error::BundleMismatch {
                message: "the bundle header names a different environment than its contents".into(),
            });
        }
    }
    if let Some(names) = &envelope.hint.names {
        let actual: Vec<String> = payload.secrets.keys().map(|n| n.to_string()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        if sorted != actual {
            return Err(Error::BundleMismatch {
                message: "the bundle header lists different secret names than its contents".into(),
            });
        }
    }

    Ok(Bundle {
        project: payload.project,
        environment: payload.environment,
        created_at: payload.created_at,
        created_by: payload.created_by,
        secrets: payload.secrets,
    })
}

/// Read header without decrypting
pub fn peek(text: &str) -> Result<UntrustedHint> {
    let envelope: wire::Envelope = serde_json::from_str(text).map_err(|e| Error::BundleFormat {
        message: format!("not a secret bundle: {e}"),
    })?;
    if envelope.magic != BUNDLE_MAGIC {
        return Err(Error::BundleFormat {
            message: format!("unexpected magic `{}`", envelope.magic),
        });
    }
    Ok(UntrustedHint {
        format: envelope.format,
        cipher: envelope.cipher,
        project: envelope.hint.project,
        environment: envelope.hint.environment,
        created_at: envelope.hint.created_at,
        created_by: envelope.hint.created_by,
        names: envelope.hint.names,
    })
}

/// Unauthenticated envelope metadata
#[derive(Debug, Clone)]
pub struct UntrustedHint {
    /// Envelope format version
    pub format: u32,
    /// Cipher identifier
    pub cipher: String,
    /// Claimed project
    pub project: Option<String>,
    /// Claimed environment
    pub environment: Option<String>,
    /// Claimed creation timestamp
    pub created_at: Option<String>,
    /// Claimed creator
    pub created_by: Option<String>,
    /// Claimed secret names
    pub names: Option<Vec<String>>,
}

impl UntrustedHint {
    /// Whether this build can decrypt the bundle at all
    pub fn is_supported(&self) -> bool {
        self.format <= BUNDLE_FORMAT
            && matches!(self.cipher.as_str(), CIPHER_SCRYPT | CIPHER_X25519)
    }

    /// Whether the bundle expects a passphrase rather than an identity file
    pub fn needs_passphrase(&self) -> bool {
        self.cipher == CIPHER_SCRYPT
    }
}

/// Suggested filename for a bundle
pub fn suggested_filename(project: &ProjectName, environment: &EnvironmentName) -> String {
    format!("{project}.{environment}.{BUNDLE_EXTENSION}")
}

fn encrypt_with(encryptor: age::Encryptor, plaintext: &[u8]) -> Result<String> {
    let mut out = Vec::new();
    let armor =
        ArmoredWriter::wrap_output(&mut out, Format::AsciiArmor).map_err(|e| Error::Crypto {
            message: e.to_string(),
        })?;
    let mut writer = encryptor.wrap_output(armor).map_err(|e| Error::Crypto {
        message: e.to_string(),
    })?;
    writer.write_all(plaintext).map_err(|e| Error::Crypto {
        message: e.to_string(),
    })?;
    writer
        .finish()
        .and_then(|armor| armor.finish())
        .map_err(|e| Error::Crypto {
            message: e.to_string(),
        })?;
    String::from_utf8(out).map_err(|_| Error::Crypto {
        message: "armor is not UTF-8".into(),
    })
}

fn decrypt_passphrase(armored: &str, passphrase: &SecretString) -> Result<Zeroizing<Vec<u8>>> {
    let decryptor = new_decryptor(armored)?;
    if !decryptor.is_scrypt() {
        return Err(Error::Crypto {
            message: "this bundle is encrypted to age identities, not a passphrase".into(),
        });
    }
    // The default max work factor is measured per device and bounded at roughly
    // 16 seconds, which is the DoS ceiling we want for untrusted input.
    let identity = age::scrypt::Identity::new(age_secret(passphrase));
    let reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(decrypt_error)?;
    read_all(reader)
}

fn decrypt_identities(armored: &str, identities: &[SecretString]) -> Result<Zeroizing<Vec<u8>>> {
    let parsed: Vec<age::x25519::Identity> = identities
        .iter()
        .map(|i| {
            age::x25519::Identity::from_str(i.expose_secret().trim()).map_err(|e| Error::Crypto {
                message: format!("invalid age identity: {e}"),
            })
        })
        .collect::<Result<_>>()?;

    let decryptor = new_decryptor(armored)?;
    if decryptor.is_scrypt() {
        return Err(Error::Crypto {
            message: "this bundle is encrypted with a passphrase, not age identities".into(),
        });
    }
    let reader = decryptor
        .decrypt(parsed.iter().map(|i| i as &dyn age::Identity))
        .map_err(decrypt_error)?;
    read_all(reader)
}

fn age_secret(value: &SecretString) -> age::secrecy::SecretString {
    age::secrecy::SecretString::new(value.expose_secret().to_owned().into_boxed_str())
}

type ArmoredInput<'a> = ArmoredReader<std::io::BufReader<&'a [u8]>>;

fn new_decryptor(armored: &str) -> Result<age::Decryptor<ArmoredInput<'_>>> {
    age::Decryptor::new(ArmoredReader::new(armored.as_bytes())).map_err(|e| Error::BundleFormat {
        message: format!("payload is not a valid age file: {e}"),
    })
}

fn decrypt_error(e: age::DecryptError) -> Error {
    match e {
        age::DecryptError::NoMatchingKeys | age::DecryptError::DecryptionFailed => Error::Crypto {
            message: "decryption failed: wrong passphrase or identity, or the bundle was modified"
                .into(),
        },
        other => Error::Crypto {
            message: other.to_string(),
        },
    }
}

fn read_all<R: Read>(mut reader: R) -> Result<Zeroizing<Vec<u8>>> {
    let mut buf = Zeroizing::new(Vec::new());
    reader.read_to_end(&mut buf).map_err(|e| Error::Crypto {
        message: format!("truncated payload: {e}"),
    })?;
    Ok(buf)
}

/// Classify a serde error without echoing the input
trait ClassifyStr {
    fn classify_str(&self) -> &'static str;
}

impl ClassifyStr for serde_json::Error {
    fn classify_str(&self) -> &'static str {
        match self.classify() {
            serde_json::error::Category::Io => "io",
            serde_json::error::Category::Syntax => "invalid JSON",
            serde_json::error::Category::Data => "unexpected structure",
            serde_json::error::Category::Eof => "truncated",
        }
    }
}
