use std::path::Path;

use kivro_core::{EnvironmentName, Error, ProjectName, Result};
use kivro_manifest::{MANIFEST_FILENAME, Manifest};

use crate::cli::InitArgs;

use super::Ctx;

pub fn run(ctx: &Ctx, args: &InitArgs) -> Result<()> {
    let directory = match &ctx.global.project {
        Some(path) if path.is_file() => path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        Some(path) => path.clone(),
        None => std::env::current_dir().map_err(Error::RawIo)?,
    };
    let path = directory.join(MANIFEST_FILENAME);

    if path.exists() && !args.force {
        return Err(Error::AlreadyExists { path });
    }
    let name = match &args.name {
        Some(name) => ProjectName::new(name.clone())?,
        None => {
            let derived = directory
                .canonicalize()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .unwrap_or_else(|| "project".to_string());
            ProjectName::new(sanitise(&derived)).map_err(|_| {
                Error::Other(format!(
                    "cannot derive a project name from `{derived}`; pass --name"
                ))
            })?
        }
    };

    let environment = EnvironmentName::new(args.default_env.clone())?;

    std::fs::write(&path, Manifest::template(&name, &environment))
        .map_err(|e| Error::io("write manifest", &path, e))?;

    if ctx.ui.json {
        ctx.ui.json_value(&serde_json::json!({
            "created": path.display().to_string(),
            "project": name.as_str(),
            "environment": environment.as_str(),
        }));
        return Ok(());
    }

    ctx.ui.info(format!("Created {}", path.display()));
    ctx.ui.blank();
    ctx.ui.info(format!("  project:     {name}"));
    ctx.ui.info(format!("  environment: {environment}"));
    ctx.ui.blank();
    ctx.ui
        .info("Next: declare your variables in the manifest, then");
    ctx.ui.info("      kivro set DATABASE_URL");
    Ok(())
}

fn sanitise(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '-'
            }
        })
        .collect();
    cleaned
        .trim_matches(|c: char| !c.is_ascii_alphanumeric())
        .to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn sanitises_awkward_directory_names() {
        assert_eq!(super::sanitise("my project!"), "my-project");
        assert_eq!(super::sanitise("infinity-launcher"), "infinity-launcher");
        assert_eq!(super::sanitise("--weird--"), "weird");
    }
}
