use std::io::{IsTerminal, Write};

use kivro_core::{Error, Result, SecretName};

use crate::cli::{GetArgs, ListArgs, RemoveArgs, SetArgs};
use crate::ui;

use super::Ctx;

pub fn set(ctx: &Ctx, args: &SetArgs) -> Result<()> {
    let project = ctx.projet()?;
    let env = ctx.environment(&project)?;
    let name = SecretName::new(args.name.clone())?;

    let value = if args.stdin {
        ui::read_secret_from_stdin()?
    } else if ui::stdin_is_tty() {
        if args.no_confirm {
            ui::prompt_secret(&format!("Value for {name}: "))?
        } else {
            ui::prompt_secret_confirmed(&format!("Value for {name}: "))?
        }
    } else {
        return Err(Error::Other(format!(
            "not a terminal; pipe the value instead:\n\n    echo \"$VALUE\" | kivro set {name} --stdin"
        )));
    };

    if value.is_empty()
        && !ctx
            .ui
            .confirm("The value is empty, store it anyways?", false)
    {
        return Err(super::cancelled());
    }

    let existed = env.get(&name)?.is_some();
    env.set(&name, &value)?;

    if !env.declarations().contains_key(&name) {
        ctx.ui.warn(format!(
            "`{name}` is not declared in the manifest; add it so your team knows its needed"
        ));
    }

    if ctx.ui.json {
        ctx.ui.json_value(&serde_json::json!({
            "name" : name.as_str(),
            "environment" : env.name().as_str(),
            "action": if existed { "updated"} else {"created"},
        }));
    } else {
        ctx.ui.info(format!(
            "{} {name} {}",
            ctx.ui.present(),
            ctx.ui.dim(&format!(
                "({} in {})",
                if existed { "updated" } else { "stored" },
                env.name()
            ))
        ));
    }
    Ok(())
}

pub fn get(ctx: &Ctx, args: &GetArgs) -> Result<()> {
    let project = ctx.projet()?;
    let env = ctx.environment(&project)?;
    let name = SecretName::new(args.name.clone())?;

    let Some(value) = env.get(&name)? else {
        return Err(Error::MissingSecret {
            name: name.to_string(),
            project: env.project_name().to_string(),
            environment: env.name().to_string(),
        });
    };

    if !args.show {
        if ctx.ui.json {
            ctx.ui.json_value(&serde_json::json!({
                "name": name.as_str(),
                "present": true,
                "length": value.len(),
            }));
        } else {
            ctx.ui.info(format!(
                "{} {name} is set {}",
                ctx.ui.present(),
                ctx.ui.dim(&format!("({} bytes)", value.len()))
            ));
            ctx.ui.info(ctx.ui.dim("  pass --show to print the value"));
        }
        return Ok(());
    }

    if std::io::stdout().is_terminal() {
        ctx.ui
            .warn("printing a secret to a terminal — it will remain in scrollback");
    }
    let mut out = std::io::stdout().lock();
    let written = out
        .write_all(value.expose_secret().as_bytes())
        .and_then(|()| out.write_all(b"\n"))
        .and_then(|()| out.flush());
    match written {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(Error::RawIo(e)),
    }
}

pub fn list(ctx: &Ctx, args: &ListArgs) -> Result<()> {
    let project = ctx.projet()?;
    let env = ctx.environment(&project)?;
    let status = env.status()?;

    let entries: Vec<_> = status
        .entries
        .iter()
        .filter(|e| args.all || e.declared)
        .collect();

    if ctx.ui.json {
        ctx.ui.json_value(&serde_json::json!({
            "project": status.project.as_str(),
            "environment": status.environment.as_str(),
            "secrets": entries.iter().map(|e| serde_json::json!({
                "name": e.name.as_str(),
                "present": e.present,
                "required": e.required,
                "declared": e.declared,
            })).collect::<Vec<_>>(),
        }));
        return Ok(());
    }

    ctx.header(&env);
    if entries.is_empty() {
        ctx.ui.info(ctx.ui.dim("  no variables declared or stored"));
        return Ok(());
    }
    for entry in entries {
        let mark = if entry.present {
            ctx.ui.present()
        } else {
            ctx.ui.absent()
        };
        let mut suffix = Vec::new();
        if !entry.declared {
            suffix.push("undeclared".to_string());
        }
        if !entry.required && entry.declared {
            suffix.push("optional".to_string());
        }
        if entry.deprecated {
            suffix.push("deprecated".to_string());
        }
        let note = if suffix.is_empty() {
            String::new()
        } else {
            ctx.ui.dim(&format!("  ({})", suffix.join(", ")))
        };
        ctx.ui.info(format!("{mark} {}{note}", entry.name));
    }
    Ok(())
}

pub fn remove(ctx: &Ctx, args: &RemoveArgs) -> Result<()> {
    let project = ctx.projet()?;
    let env = ctx.environment(&project)?;
    let name = SecretName::new(args.name.clone())?;

    if env.get(&name)?.is_none() {
        ctx.ui.info(format!(
            "{} {name} is not set in {}",
            ctx.ui.dim("-"),
            env.name()
        ));
        return Ok(());
    }

    if !args.yes
        && !ctx.ui.confirm(
            &format!(
                "Delete `{name}` from {}/{}?",
                env.project_name(),
                env.name()
            ),
            false,
        )
    {
        return Err(super::cancelled());
    }

    env.remove(&name)?;
    if ctx.ui.json {
        ctx.ui
            .json_value(&serde_json::json!({ "name": name.as_str(), "removed": true }));
    } else {
        ctx.ui.info(format!("{} removed {name}", ctx.ui.present()));
    }
    Ok(())
}
