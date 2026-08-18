pub mod bundle;
pub mod crud;
pub mod envfile;
pub mod init;
pub mod run;
pub mod status;
pub mod sync;

use kivro::config::Config;
use kivro::{Environment, Project};
use kivro_core::{Error, Result};
use kivro_manifest::Manifest;

use crate::cli::GlobalArgs;
use crate::ui::Ui;

pub struct Ctx {
    pub ui: Ui,
    pub global: GlobalArgs,
    pub config: Config,
}

impl Ctx {
    pub fn manifest(&self) -> Result<Manifest> {
        let manifest = match &self.global.project {
            Some(path) if path.is_file() => Manifest::load(path)?,
            Some(path) => Manifest::discover_from(path)?,
            None => Manifest::discover()?,
        };

        manifest.check_cli_version(env!("CARGO_PKG_VERSION"))?;
        Ok(manifest)
    }

    pub fn projet(&self) -> Result<Project> {
        let manifest = self.manifest()?;
        let store = kivro_keyring::open_from_env(&self.config.storage.namespace)?;
        if !store.is_secure() {
            self.ui.warn(format!(
                "using the `{}` backend - secrets are not protected bt the OS",
                store.backend()
            ));
        }
        Ok(Project::new(manifest, store, self.config.clone()))
    }

    pub fn environment<'a>(&self, project: &'a Project) -> Result<Environment<'a>> {
        project.resolve_environment(self.global.env.as_deref())
    }

    pub fn header(&self, env: &Environment<'_>) {
        self.ui.info(format!(
            "{} {} {}",
            self.ui.bold(env.project_name().as_str()),
            self.ui.dim("/"),
            env.name()
        ));
        self.ui.blank();
    }
}

pub fn cancelled() -> Error {
    Error::Cancelled
}
