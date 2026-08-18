use std::path::PathBuf;

use kivro_core::{Error, Result, SecretString};
use kivro_crypto::{Bundle, OpenKey, SealKey, SealOptions, open, peek, seal};

use crate::cli::{AcceptArgs, ShareArgs};
use crate::ui;

use super::Ctx;

pub fn share(ctx: &Ctx, args: &ShareArgs) -> Result<()> {
    let project = ctx.projet()?;
    let env = ctx.environment(&project)?;

    let secrets = if args.all {
        env.load_all_stored()?
    } else {
        env.load_available()?
    };
    if secrets.is_empty() {
        return Err(Error::Other(format!(
            "nothing to share: no secrets are stored for {}/{}",
            env.project_name(),
            env.name()
        )));
    }

    let path: PathBuf = args.out.clone().unwrap_or_else(|| {
        project
            .manifest()
            .root
            .join(kivro_crypto::suggested_filename(
                env.project_name(),
                env.name(),
            ))
    });

    let key = if args.recipient.is_empty() {
        ctx.ui.info("Choose a passhphrase to protect this bundle");
        ctx.ui.info(
            ctx.ui
                .dim("  Send it to the recipient over a different channel than the file"),
        );
        let passphrase = match std::env::var_os(ui::PASSPHRASE_ENV) {
            Some(_) => ui::passphrase("Passphrase: ")?,
            None if !ui::stdin_is_tty() => {
                return Err(Error::Other(format!(
                    "not a terminal: pass --recipient <age public key>, or set {} to share non-interactively",
                    ui::PASSPHRASE_ENV
                )));
            }
            None => ui::prompt_secret_confirmed("Passphrase: ")?,
        };
        if passphrase.len() < 12 {
            ctx.ui
                .warn("consider picking a stronger passphrase, prefer several random words");
        }
        SealKey::Passphrase(passphrase)
    } else {
        SealKey::Recipients(args.recipient.clone())
    };

    let created_ad = timestamp();
    let bundle = Bundle::new(
        env.project_name().clone(),
        env.name().clone(),
        secrets.values().clone(),
    )
    .with_metadata(
        Some(created_ad),
        std::env::var("USER")
            .ok()
            .or_else(|| std::env::var("USERNAME").ok()),
    );

    let options = SealOptions {
        hint_identity: true,
        hint_names: args.hint_names,
    };
    let text = seal(&bundle, &key, options)?;
    std::fs::write(&path, text).map_err(|e| Error::io("write bundle", &path, e))?;

    if ctx.ui.json {
        ctx.ui.json_value(&serde_json::json!({
            "written": path.display().to_string(),
            "count": secrets.len(),
            "cipher": if args.recipient.is_empty() { kivro_crypto::CIPHER_SCRYPT} else { kivro_crypto::CIPHER_X25519}
        }));
        return Ok(());
    }

    ctx.ui.blank();
    ctx.ui.info(format!(
        "{} wrote {} ({} secret{})",
        ctx.ui.present(),
        path.display(),
        secrets.len(),
        if secrets.len() == 1 { "" } else { "s" }
    ));
    ctx.ui.blank();
    ctx.ui.info("the file is encrypted an safe to send");
    ctx.ui.info(format!(
        "the recipient runs: kivro accept {}",
        path.display()
    ));
    Ok(())
}

pub fn accept(ctx: &Ctx, args: &AcceptArgs) -> Result<()> {
    let project = ctx.projet()?;

    let text =
        std::fs::read_to_string(&args.path).map_err(|e| Error::io("read bundle", &args.path, e))?;

    let hint = peek(&text)?;
    if !hint.is_supported() {
        return Err(Error::BundleFormat {
            message: format!(
                "unsupported bundle (format {}, cipher `{}`); upgrade the `kivro` CLI",
                hint.format, hint.cipher
            ),
        });
    }
    if let (Some(p), Some(e)) = (&hint.project, &hint.environment) {
        ctx.ui
            .info(format!("Bundle claims to hold secrets for {p}/{e}."));
    }

    let key = match &args.identity {
        Some(path) => {
            let text = std::fs::read_to_string(path)
                .map_err(|e| Error::io("read identity file", path, e))?;
            let identities: Vec<SecretString> = text
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(SecretString::new)
                .collect();
            if identities.is_empty() {
                return Err(Error::Other(format!(
                    "no identities found in {}",
                    path.display()
                )));
            }
            OpenKey::Identities(identities)
        }
        None if hint.needs_passphrase() => OpenKey::Passphrase(ui::passphrase("Passphrase: ")?),
        None => {
            return Err(Error::Other(
                "this bundle is encrypted to age recipients; pass --identity <file>".into(),
            ));
        }
    };

    let bundle = open(&text, &key)?;

    if &bundle.project != project.name() {
        return Err(Error::BundleMismatch {
            message: format!(
                "the bundle holds secrets for `{}`, but this project is `{}`",
                bundle.project,
                project.name()
            ),
        });
    }

    let env = project.environment(bundle.environment.as_str())?;
    let mut stored = Vec::new();
    let mut skipped = Vec::new();
    for (name, value) in &bundle.secrets {
        if !args.force && env.get(name)?.is_some() {
            skipped.push(name.to_string());
            continue;
        }
        env.set(name, value)?;
        stored.push(name.to_string());
    }

    if ctx.ui.json {
        ctx.ui.json_value(&serde_json::json!({
            "project": bundle.project.as_str(),
            "environment": bundle.environment.as_str(),
            "stored": stored,
            "skipped_existing": skipped,
        }));
        return Ok(());
    }

    ctx.ui.info(format!(
        "{} stored {} secret{} in {}/{}",
        ctx.ui.present(),
        stored.len(),
        if stored.len() == 1 { "" } else { "s" },
        bundle.project,
        bundle.environment
    ));
    for name in &stored {
        ctx.ui.info(format!("  {} {name}", ctx.ui.present()));
    }
    if !skipped.is_empty() {
        ctx.ui.info(ctx.ui.dim(&format!(
            "  {} already set (pass --force to overwrite): {}",
            skipped.len(),
            skipped.join(", ")
        )));
    }
    ctx.ui.blank();
    ctx.ui.info(ctx.ui.dim(&format!(
        "Delete {} now that it has been imported.",
        args.path.display()
    )));
    Ok(())
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let days = (secs / 86_400) as i64;
    let time = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        time / 3600,
        (time % 3600) / 60,
        time % 60
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn timestamp_is_rfc3339_shaped() {
        let ts = super::timestamp();
        assert_eq!(ts.len(), 20, "{ts}");
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.as_bytes()[4], b'-');
        assert_eq!(ts.as_bytes()[10], b'T');
    }
}
