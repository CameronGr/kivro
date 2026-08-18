use std::path::Path;

use kivro::envfile;
use kivro_core::{Error, Result};

use crate::cli::DoctorArgs;

use super::Ctx;

pub fn status(ctx: &Ctx) -> Result<bool> {
    let project = ctx.projet()?;
    let env = ctx.environment(&project)?;
    let status = env.status()?;
    let missing = status.missing_required();

    if ctx.ui.json {
        ctx.ui.json_value(&serde_json::json!({
            "project": status.project.as_str(),
            "environment": status.environment.as_str(),
            "satisfied": status.is_satisfied(),
            "missing": missing.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
            "secrets": status.entries.iter().map(|e| serde_json::json!({
                "name": e.name.as_str(),
                "present": e.present,
                "required": e.required,
                "declared": e.declared,
            })).collect::<Vec<_>>(),
        }));
        return Ok(status.is_satisfied());
    }

    ctx.header(&env);

    let required: Vec<_> = status.entries.iter().filter(|e| e.required).collect();
    if !required.is_empty() {
        ctx.ui.info("Required secrets:");
        ctx.ui.blank();
        for entry in &required {
            let mark = if entry.present {
                ctx.ui.present()
            } else {
                ctx.ui.absent()
            };
            ctx.ui.info(format!("  {mark} {}", entry.name));
        }
        ctx.ui.blank();
    }

    let optional: Vec<_> = status
        .entries
        .iter()
        .filter(|e| e.declared && !e.required)
        .collect();
    if !optional.is_empty() {
        ctx.ui.info("Optional secrets:");
        ctx.ui.blank();
        for entry in &optional {
            let mark = if entry.present {
                ctx.ui.present()
            } else {
                ctx.ui.dim("-")
            };
            ctx.ui.info(format!("  {mark} {}", entry.name));
        }
        ctx.ui.blank();
    }

    let undeclared = status.undeclared();
    if !undeclared.is_empty() {
        ctx.ui.info("Stored but not declared:");
        ctx.ui.blank();
        for entry in &undeclared {
            ctx.ui
                .info(format!("  {} {}", ctx.ui.note_mark(), entry.name));
        }
        ctx.ui.blank();
    }

    if missing.is_empty() {
        ctx.ui.info(format!(
            "{} all required secrets are present.",
            ctx.ui.present()
        ));
    } else {
        ctx.ui.info(format!(
            "{} secret{} missing.",
            missing.len(),
            if missing.len() == 1 { "" } else { "s" }
        ));
        ctx.ui.blank();
        ctx.ui.info("Run:");
        for entry in &missing {
            ctx.ui.info(format!("    kivro set {}", entry.name));
        }
    }

    Ok(status.is_satisfied())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Ok,
    Warn,
    Fail,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Ok => "ok",
            Level::Warn => "warning",
            Level::Fail => "error",
        }
    }
}

pub struct Check {
    pub level: Level,
    pub title: String,
    pub detail: Option<String>,
}

impl Check {
    fn ok(title: impl Into<String>) -> Self {
        Self {
            level: Level::Ok,
            title: title.into(),
            detail: None,
        }
    }
    fn warn(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            level: Level::Warn,
            title: title.into(),
            detail: Some(detail.into()),
        }
    }
    fn fail(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            level: Level::Fail,
            title: title.into(),
            detail: Some(detail.into()),
        }
    }
    fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

pub fn doctor(ctx: &Ctx, args: &DoctorArgs) -> Result<Level> {
    let mut checks = Vec::new();

    let manifest = match ctx.manifest() {
        Ok(manifest) => {
            checks
                .push(Check::ok("manifest found").with_detail(manifest.path.display().to_string()));
            checks.push(Check::ok("manifest is valid"));
            checks.push(Check::ok("project identity").with_detail(manifest.project.to_string()));
            checks.push(
                Check::ok("CLI version is compatible")
                    .with_detail(format!("kivro {}", env!("CARGO_PKG_VERSION"))),
            );
            for key in &manifest.unknown_keys {
                checks.push(Check::warn(
                    format!("unrecognised manifest key `{key}`"),
                    "ignored by this version — it may need a newer CLI",
                ));
            }
            Some(manifest)
        }
        Err(e) => {
            checks.push(Check::fail("manifest", e.to_string()));
            None
        }
    };

    let store = kivro_keyring::open_from_env(&ctx.config.storage.namespace);
    match &store {
        Ok(store) => match store.check_available() {
            Ok(()) if store.is_secure() => {
                checks.push(Check::ok("credential store").with_detail(store.backend().to_string()))
            }
            Ok(()) => checks.push(Check::warn(
                format!("credential store is `{}`", store.backend()),
                "secrets are not protected by the OS; unset KIVRO_STORE for normal use",
            )),
            Err(e) => checks.push(Check::fail("credential store", e.to_string())),
        },
        Err(e) => checks.push(Check::fail("credential store", e.to_string())),
    }

    if let (Some(manifest), Ok(store)) = (manifest.as_ref(), store) {
        let project = kivro::Project::new(manifest.clone(), store, ctx.config.clone());
        match ctx.environment(&project) {
            Ok(env) => {
                checks.push(Check::ok("environment resolved").with_detail(env.name().to_string()));
                match env.status() {
                    Ok(status) => {
                        let missing = status.missing_required();
                        if missing.is_empty() {
                            checks.push(Check::ok("required secrets are present"));
                        } else {
                            checks.push(Check::fail(
                                format!("{} required secret(s) missing", missing.len()),
                                missing
                                    .iter()
                                    .map(|e| e.name.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", "),
                            ));
                        }
                        let deprecated: Vec<_> = status
                            .entries
                            .iter()
                            .filter(|e| e.deprecated && e.present)
                            .map(|e| e.name.to_string())
                            .collect();
                        if !deprecated.is_empty() {
                            checks.push(Check::warn(
                                "deprecated secrets are still stored",
                                deprecated.join(", "),
                            ));
                        }
                    }
                    Err(e) => checks.push(Check::fail("reading stored secrets", e.to_string())),
                }
            }
            Err(e) => checks.push(Check::fail("environment", e.to_string())),
        }
    }

    if let Some(manifest) = manifest.as_ref() {
        checks.extend(git_checks(ctx, &manifest.root, args.fix_gitignore)?);
    }

    report(ctx, &checks)
}

fn git_checks(ctx: &Ctx, root: &Path, fix: bool) -> Result<Vec<Check>> {
    let mut checks = Vec::new();
    let gitignore_path = root.join(".gitignore");
    let gitignore = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
    let ignores_env = gitignore
        .lines()
        .map(str::trim)
        .any(|l| matches!(l, ".env" | ".env.*" | "*.env" | ".env*"));

    let env_path = root.join(".env");
    let env_text = std::fs::read_to_string(&env_path).ok();

    match (&env_text, ignores_env) {
        (Some(text), false) if envfile::looks_like_secrets(text) => checks.push(Check::fail(
            "`.env` contains secrets and is not ignored by git",
            "add `.env` to .gitignore, then `kivro import .env`",
        )),
        (Some(_), false) => checks.push(Check::warn(
            "`.env` is present and not ignored by git",
            "add `.env` to .gitignore",
        )),
        (Some(_), true) => checks.push(Check::warn(
            "`.env` is present",
            "it is git-ignored, but `kivro import .env` moves it into the credential store",
        )),
        (None, _) => checks.push(Check::ok("no stray `.env` in the project root")),
    }

    if !ignores_env && env_text.is_some() {
        let addition = "\n# Added by `kivro doctor --fix-gitignore`\n.env\n.env.*\n*.kivro\n";
        if fix {
            if ctx.ui.confirm(
                &format!(
                    "Append recommended entries to {}?",
                    gitignore_path.display()
                ),
                true,
            ) {
                let mut current = gitignore.clone();
                current.push_str(addition);
                std::fs::write(&gitignore_path, current)
                    .map_err(|e| Error::io("write .gitignore", &gitignore_path, e))?;
                checks.push(Check::ok("updated .gitignore"));
            } else {
                checks.push(Check::warn(".gitignore unchanged", "declined"));
            }
        } else {
            checks.push(Check::warn(
                "recommended .gitignore entries missing",
                "run `kivro doctor --fix-gitignore`, or add `.env`, `.env.*` and `*.kivro` yourself",
            ));
        }
    }

    let stray_bundles: Vec<String> = std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(&format!(".{}", kivro_crypto::BUNDLE_EXTENSION))
                .then_some(name)
        })
        .collect();
    if !stray_bundles.is_empty() {
        checks.push(Check::warn(
            "encrypted bundles in the project root",
            format!("{} — delete them once accepted", stray_bundles.join(", ")),
        ));
    }

    Ok(checks)
}

fn report(ctx: &Ctx, checks: &[Check]) -> Result<Level> {
    let worst = checks
        .iter()
        .map(|c| c.level)
        .max_by_key(|l| match l {
            Level::Ok => 0,
            Level::Warn => 1,
            Level::Fail => 2,
        })
        .unwrap_or(Level::Ok);

    if ctx.ui.json {
        ctx.ui.json_value(&serde_json::json!({
            "status": worst.as_str(),
            "checks": checks.iter().map(|c| serde_json::json!({
                "level": c.level.as_str(),
                "title": c.title,
                "detail": c.detail,
            })).collect::<Vec<_>>(),
        }));
        return Ok(worst);
    }

    for check in checks {
        let mark = match check.level {
            Level::Ok => ctx.ui.present(),
            Level::Warn => ctx.ui.note_mark(),
            Level::Fail => ctx.ui.absent(),
        };
        ctx.ui.info(format!("{mark} {}", check.title));
        if let Some(detail) = &check.detail {
            ctx.ui.info(ctx.ui.dim(&format!("    {detail}")));
        }
    }
    ctx.ui.blank();
    match worst {
        Level::Ok => ctx.ui.info("No problems found."),
        Level::Warn => ctx.ui.info("Finished with warnings."),
        Level::Fail => ctx.ui.info("Problems found."),
    }
    Ok(worst)
}
