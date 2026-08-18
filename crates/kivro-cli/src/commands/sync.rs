use kivro_core::Result;
use kivro_sync::{SyncPlan, from_config};

use crate::cli::SyncArgs;
use crate::ui;

use super::Ctx;

pub fn sync(ctx: &Ctx, args: &SyncArgs) -> Result<()> {
    let project = ctx.projet()?;
    let env = ctx.environment(&project)?;
    let status = env.status()?;

    let declared: Vec<_> = env.declarations().keys().cloned().collect();
    let present: Vec<_> = status
        .entries
        .iter()
        .filter(|e| e.present)
        .map(|e| e.name.clone())
        .collect();

    let source = match &project.manifest().sync {
        Some(config) => Some(from_config(
            config,
            &project.manifest().root,
            Box::new(|what| ui::passphrase(&format!("Passphrase for {what}: "))),
        )?),
        None => None,
    };

    let available = match &source {
        Some(source) => source.available(&env.scope())?,
        None => Vec::new(),
    };
    let plan = SyncPlan::compute(&declared, &present, &available);

    let mut fetched: Vec<String> = Vec::new();
    let mut plan = plan;
    if args.apply && !plan.fetchable.is_empty() {
        let source = source.as_ref().expect("fetchable implies a source");
        for (name, value) in source.fetch(&env.scope(), &plan.fetchable)? {
            env.set(&name, &value)?;
            fetched.push(name.to_string());
        }
        let present: Vec<_> = env
            .status()?
            .entries
            .iter()
            .filter(|e| e.present)
            .map(|e| e.name.clone())
            .collect();
        plan = SyncPlan::compute(&declared, &present, &available);
    }

    if ctx.ui.json {
        ctx.ui.json_value(&serde_json::json!({
            "project": env.project_name().as_str(),
            "environment": env.name().as_str(),
            "source": source.as_ref().map(|s| s.describe()),
            "present": plan.present.iter().map(|n| n.to_string()).collect::<Vec<_>>(),
            "missing": plan.missing.iter().map(|n| n.to_string()).collect::<Vec<_>>(),
            "fetchable": plan.fetchable.iter().map(|n| n.to_string()).collect::<Vec<_>>(),
            "unavailable": plan.unavailable.iter().map(|n| n.to_string()).collect::<Vec<_>>(),
            "fetched": fetched,
        }));
        return Ok(());
    }

    ctx.header(&env);
    match &source {
        Some(source) => ctx.ui.info(format!("Source: {}", source.describe())),
        None => ctx.ui.info(
            ctx.ui
                .dim("No sync source configured — add a [sync] section to .kivro.toml"),
        ),
    }
    ctx.ui.blank();

    if !plan.present.is_empty() {
        ctx.ui.info("Local secrets:");
        for name in &plan.present {
            ctx.ui.info(format!("  {} {name}", ctx.ui.present()));
        }
        ctx.ui.blank();
    }
    if !plan.missing.is_empty() {
        ctx.ui.info("Missing:");
        for name in &plan.missing {
            let note = if plan.fetchable.contains(name) {
                ctx.ui.dim("  (available from source)")
            } else {
                String::new()
            };
            ctx.ui.info(format!("  {} {name}{note}", ctx.ui.absent()));
        }
        ctx.ui.blank();
    }

    if !fetched.is_empty() {
        ctx.ui.info(format!(
            "{} fetched {} secret(s)",
            ctx.ui.present(),
            fetched.len()
        ));
    } else if !plan.fetchable.is_empty() {
        ctx.ui.info(format!(
            "{} secret(s) can be fetched — run `kivro sync --apply`",
            plan.fetchable.len()
        ));
    } else if plan.is_complete() {
        ctx.ui.info(format!("{} nothing to do.", ctx.ui.present()));
    }

    Ok(())
}
