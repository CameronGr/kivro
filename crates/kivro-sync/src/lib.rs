//! Synchronisation sources.
//!
//! `kivro sync` answers one question: *the manifest says I need these
//! secrets — which am I missing, and can I get them from somewhere?* The
//! "somewhere" is deliberately behind a trait.
//!
//! [`BundleFileSource`] is the only implementation today: a directory of
//! encrypted bundles, which needs no server and works over any transport a team
//! already has (a shared drive, a repository, a chat attachment).
//!
//! Future sources — an internal HTTP service, S3/R2, git-backed storage,
//! another secret manager — implement [`SyncSource`] without changing the
//! bundle format or anything above this layer. That is the point of the split:
//! the *format* is the compatibility surface, not the transport.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use kivro_core::{Error, Result, Scope, SecretName, SecretString};
use kivro_crypto::{Bundle, OpenKey, open};
use kivro_manifest::SyncConfig;

/// supply the passphrase on demand
pub type PassphraseProvider = Box<dyn Fn(&str) -> Result<SecretString> + Send + Sync>;

/// a source of secrets to and from
pub trait SyncSource {
    /// short id
    fn kind(&self) -> &str;

    /// description
    fn describe(&self) -> String;

    /// names that this source and supply for the scope
    fn available(&self, scope: &Scope) -> Result<Vec<SecretName>>;

    /// fetch the requested names
    fn fetch(
        &self,
        sccope: &Scope,
        names: &[SecretName],
    ) -> Result<BTreeMap<SecretName, SecretString>>;

    /// if [`SyncSource::publish`] is supported
    fn is_writable(&self) -> bool {
        false
    }

    /// publish values to source
    fn publish(&self, _scope: &Scope, _values: &BTreeMap<SecretName, SecretString>) -> Result<()> {
        Err(Error::Sync {
            message: format!("the `{}` source is read only", self.kind()),
        })
    }
}

/// TODO: sync ----------------------------------
#[derive(Debug, Clone, Default)]
pub struct SyncPlan {
    /// required or declared names already in the local store
    pub present: Vec<SecretName>,
    /// declared names with no local value
    pub missing: Vec<SecretName>,
    /// missing names the source can supply
    pub fetchable: Vec<SecretName>,
    /// missing names nothing can supply
    pub unavailable: Vec<SecretName>,
}

impl SyncPlan {
    /// create a sync plan
    pub fn compute(
        declared: &[SecretName],
        present: &[SecretName],
        available: &[SecretName],
    ) -> Self {
        let mut plan = SyncPlan::default();
        for name in declared {
            if present.contains(name) {
                plan.present.push(name.clone());
            } else if available.contains(name) {
                plan.missing.push(name.clone());
                plan.fetchable.push(name.clone());
            } else {
                plan.missing.push(name.clone());
                plan.unavailable.push(name.clone());
            }
        }
        plan
    }

    /// does anything need to be done
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty()
    }
}

/// a directory of encrypted bundles
pub struct BundleFileSource {
    directory: PathBuf,
    passphrase: PassphraseProvider,
}

impl BundleFileSource {
    /// point at a directory of bundles
    pub fn new(directory: impl Into<PathBuf>, passphrase: PassphraseProvider) -> Self {
        Self {
            directory: directory.into(),
            passphrase,
        }
    }

    fn bundle_path(&self, scope: &Scope) -> PathBuf {
        self.directory.join(kivro_crypto::suggested_filename(
            &scope.project,
            &scope.environment,
        ))
    }

    fn read_bundle(&self, scope: &Scope) -> Result<Option<Bundle>> {
        let path = self.bundle_path(scope);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(Error::io("read bundle", &path, e)),
        };

        let passphrase = (self.passphrase)(&path.display().to_string())?;
        let bundle = open(&text, &OpenKey::Passphrase(passphrase))?;

        if bundle.project != scope.project || bundle.environment != scope.environment {
            return Err(Error::BundleMismatch {
                message: format!(
                    "`{}` contains secrets for {}/{}, not {}",
                    path.display(),
                    bundle.project,
                    bundle.environment,
                    scope
                ),
            });
        }
        Ok(Some(bundle))
    }
}

impl SyncSource for BundleFileSource {
    fn kind(&self) -> &str {
        "file"
    }

    fn describe(&self) -> String {
        format!("encrypted bundles in `{}`", self.directory.display())
    }

    fn available(&self, scope: &Scope) -> Result<Vec<SecretName>> {
        Ok(self
            .read_bundle(scope)?
            .map(|b| b.names())
            .unwrap_or_default())
    }

    fn fetch(
        &self,
        sccope: &Scope,
        names: &[SecretName],
    ) -> Result<BTreeMap<SecretName, SecretString>> {
        let Some(bundle) = self.read_bundle(sccope)? else {
            return Ok(BTreeMap::new());
        };
        Ok(bundle
            .secrets
            .into_iter()
            .filter(|(name, _)| names.contains(name))
            .collect())
    }
}

/// build a source from a manifest sync section
pub fn from_config(
    config: &SyncConfig,
    root: &Path,
    passphrase: PassphraseProvider,
) -> Result<Box<dyn SyncSource>> {
    match config.kind.as_str() {
        "file" => {
            let raw = config.settings.get("path").ok_or_else(|| Error::Sync {
                message: "`[sync] kind = \"file\"` requires `path`".into(),
            })?;
            let path = Path::new(raw);
            let directory = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            Ok(Box::new(BundleFileSource::new(directory, passphrase)))
        }
        other => Err(Error::Sync {
            message: format!(
                "unknown sync backend `{other}` (this build supports `file`); \
                upgrade the CLI if your team has adopted a newer one"
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivro_core::{EnvironmentName, ProjectName};
    use kivro_crypto::{SealKey, SealOptions, seal};

    fn n(s: &str) -> SecretName {
        SecretName::new(s).unwrap()
    }

    fn scope(project: &str, env: &str) -> Scope {
        Scope::new(
            ProjectName::new(project).unwrap(),
            EnvironmentName::new(env).unwrap(),
        )
    }

    fn provider() -> PassphraseProvider {
        Box::new(|_| Ok(SecretString::new("pass")))
    }

    fn write_bundle(dir: &Path, scope: &Scope, names: &[&str]) {
        let secrets = names
            .iter()
            .map(|name| (n(name), SecretString::new(format!("value-of-{name}"))))
            .collect();
        let bundle = Bundle::new(scope.project.clone(), scope.environment.clone(), secrets);
        let text = seal(
            &bundle,
            &SealKey::Passphrase(SecretString::new("pass")),
            SealOptions::default(),
        )
        .unwrap();
        std::fs::write(
            dir.join(kivro_crypto::suggested_filename(
                &scope.project,
                &scope.environment,
            )),
            text,
        )
        .unwrap();
    }

    #[test]
    fn plan_partitions_declared_names() {
        let declared = vec![n("A"), n("B"), n("C")];
        let present = vec![n("A")];
        let available = vec![n("B")];
        let plan = SyncPlan::compute(&declared, &present, &available);

        assert_eq!(plan.present, vec![n("A")]);
        assert_eq!(plan.missing, vec![n("B"), n("C")]);
        assert_eq!(plan.fetchable, vec![n("B")]);
        assert_eq!(plan.unavailable, vec![n("C")]);
        assert!(!plan.is_complete());
        assert!(SyncPlan::compute(&declared, &declared, &[]).is_complete());
    }

    #[test]
    fn file_source_reads_only_its_own_scope() {
        let dir = tempfile::tempdir().unwrap();
        let dev = scope("app", "dev");
        write_bundle(dir.path(), &dev, &["A", "B"]);

        let source = BundleFileSource::new(dir.path(), provider());
        assert_eq!(source.available(&dev).unwrap(), vec![n("A"), n("B")]);
        assert!(
            source
                .available(&scope("app", "production"))
                .unwrap()
                .is_empty()
        );

        let fetched = source.fetch(&dev, &[n("A")]).unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[&n("A")].expose_secret(), "value-of-A");
    }

    #[test]
    fn a_renamed_bundle_cannot_impersonate_another_project() {
        let dir = tempfile::tempdir().unwrap();
        let real = scope("other-app", "dev");
        let secrets = [(n("A"), SecretString::new("v"))].into_iter().collect();
        let bundle = Bundle::new(real.project.clone(), real.environment.clone(), secrets);
        let text = seal(
            &bundle,
            &SealKey::Passphrase(SecretString::new("pass")),
            SealOptions::default(),
        )
        .unwrap();
        std::fs::write(dir.path().join("app.dev.kivro"), text).unwrap();

        let source = BundleFileSource::new(dir.path(), provider());
        let err = source.available(&scope("app", "dev")).unwrap_err();
        assert_eq!(err.kind(), "bundle_mismatch");
    }

    #[test]
    fn wrong_passphrase_surfaces_as_a_crypto_error() {
        let dir = tempfile::tempdir().unwrap();
        let dev = scope("app", "dev");
        write_bundle(dir.path(), &dev, &["A"]);

        let source =
            BundleFileSource::new(dir.path(), Box::new(|_| Ok(SecretString::new("wrong"))));
        assert_eq!(source.available(&dev).unwrap_err().kind(), "crypto_error");
    }

    #[test]
    fn unknown_backends_are_refused_with_guidance() {
        let config = SyncConfig {
            kind: "s3".into(),
            settings: BTreeMap::new(),
        };
        let err = from_config(&config, Path::new("/tmp"), provider())
            .err()
            .expect("refused");
        assert!(err.to_string().contains("unknown sync backend"));

        let config = SyncConfig {
            kind: "file".into(),
            settings: BTreeMap::new(),
        };
        let err = from_config(&config, Path::new("/tmp"), provider())
            .err()
            .expect("refused");
        assert!(err.to_string().contains("requires `path`"));
    }

    #[test]
    fn relative_paths_resolve_against_the_project_root() {
        let mut settings = BTreeMap::new();
        settings.insert("path".to_string(), "team".to_string());
        let config = SyncConfig {
            kind: "file".into(),
            settings,
        };
        let root = Path::new("/projects/app");
        let source = match from_config(&config, root, provider()) {
            Ok(source) => source,
            Err(e) => panic!("expected a source: {e}"),
        };
        let expected = root.join("team");
        assert!(source.describe().contains(&expected.display().to_string()));
    }

    #[test]
    fn read_only_sources_refuse_publish() {
        let dir = tempfile::tempdir().unwrap();
        let source = BundleFileSource::new(dir.path(), provider());
        assert!(!source.is_writable());
        assert_eq!(
            source
                .publish(&scope("a", "dev"), &BTreeMap::new())
                .unwrap_err()
                .kind(),
            "sync_error"
        );
    }
}
