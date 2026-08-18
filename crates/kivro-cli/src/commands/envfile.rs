use kivro::envfile;
use kivro_core::{Error, Result};

use crate::cli::{ExportArgs, ImportArgs};

use super::Ctx;

pub fn import(ctx: &Ctx, args: &ImportArgs) -> Result<()> {
    let project = ctx.projet()?;
    let env = ctx.environment(&project)?;

    let path = if args.path.is_absolute() {
        args.path.clone()
    } else {
        project.manifest().root.join(&args.path)
    };
    let text = std::fs::read_to_string(&path).map_err(|e| Error::io("read", &path, e))?;
    let parsed = envfile::parse(&path, &text)?;

    for (key, line) in &parsed.skipped {
        ctx.ui.warn(format!(
            "line {line}: skipping `{key}` - not a valid variable name"
        ));
    }

    let mut imported = Vec::new();
    let mut skipped_existing = Vec::new();
    for (name, value) in &parsed.entries {
        if !args.force && env.get(name)?.is_some() {
            skipped_existing.push(name.to_string());
            continue;
        }
        env.set(name, value)?;
        imported.push(name.to_string());
    }

    if ctx.ui.json {
        ctx.ui.json_value(&serde_json::json!({
            "source": path.display().to_string(),
            "imported": imported,
            "skipped_existing": skipped_existing,
            "skipped_invalid": parsed.skipped.iter().map(|(k, _)| k).collect::<Vec<_>>(),
        }));
    } else {
        ctx.ui.info(format!(
            "{} imported {} secret{} into {}/{}",
            ctx.ui.present(),
            imported.len(),
            if imported.len() == 1 { "" } else { "s" },
            env.project_name(),
            env.name()
        ));
        for name in &imported {
            ctx.ui.info(format!("  {} {name}", ctx.ui.present()));
        }
        if !skipped_existing.is_empty() {
            ctx.ui.info(ctx.ui.dim(&format!(
                "  {} already set (pass --force to overwrite): {}",
                skipped_existing.len(),
                skipped_existing.join(", ")
            )));
        }
    }

    if args.delete_source {
        if ctx
            .ui
            .confirm(&format!("Delete {}?", path.display()), false)
        {
            std::fs::remove_file(&path).map_err(|e| Error::io("delete", &path, e))?;
            ctx.ui
                .info(format!("{} deleted {}", ctx.ui.present(), path.display()));
        } else {
            ctx.ui.info(ctx.ui.dim("  source file kept"));
        }
    } else if !parsed.entries.is_empty() {
        ctx.ui.blank();
        ctx.ui.info(ctx.ui.dim(&format!(
               "{} still contains plaintext secrets. Delete it when you are satisfied the import worked.",
               path.display()
           )));
    }

    Ok(())
}

pub fn export(ctx: &Ctx, args: &ExportArgs) -> Result<()> {
    let project = ctx.projet()?;
    let env = ctx.environment(&project)?;

    let path = if args.out.is_absolute() {
        args.out.clone()
    } else {
        project.manifest().root.join(&args.out)
    };
    if path.exists() && !args.force {
        return Err(Error::AlreadyExists { path });
    }

    if !args.yes {
        ctx.ui.warn(format!(
            "this writes plaintext secrets to {} — make sure it is git-ignored",
            path.display()
        ));
        if !ctx.ui.confirm("Continue?", false) {
            return Err(super::cancelled());
        }
    }

    let secrets = env.load_available()?;
    let text = envfile::render(secrets.values());
    write_private(&path, text.as_bytes())?;

    if ctx.ui.json {
        ctx.ui.json_value(&serde_json::json!({
            "written": path.display().to_string(),
            "count": secrets.len(),
            "names": secrets.names().iter().map(|n| n.to_string()).collect::<Vec<_>>(),
        }));
    } else {
        ctx.ui.info(format!(
            "{} wrote {} secret{} to {}",
            ctx.ui.present(),
            secrets.len(),
            if secrets.len() == 1 { "" } else { "s" },
            path.display()
        ));
    }
    Ok(())
}

fn write_private(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|e| Error::io("create", path, e))?;
    file.write_all(bytes)
        .map_err(|e| Error::io("write", path, e))
}
