#![forbid(unsafe_code)]

mod cli;
mod commands;
mod ui;

use clap::Parser;
use kivro::config::Config;
use kivro_core::Error;

use crate::cli::{Cli, Command};
use crate::commands::Ctx;
use crate::commands::status::Level;
use crate::ui::Ui;

fn main() {
    let cli = Cli::parse();

    let config = Config::load().unwrap_or_default();
    let ui = Ui::new(
        config.ui.color,
        cli.global.no_color,
        cli.global.json,
        cli.global.quiet,
    );
    let ctx = Ctx {
        ui,
        global: cli.global.clone(),
        config,
    };

    let code = match dispatch(&ctx, &cli.command) {
        Ok(code) => code,
        Err(error) => {
            if ctx.ui.json {
                ctx.ui.json_value(&serde_json::json!({
                    "error": {
                        "kind": error.kind(),
                        "message": error.to_string(),
                        "hint": error.hint(),
                    }
                }));
            } else {
                ctx.ui.error(&error);
            }
            exit_code(&error)
        }
    };
    std::process::exit(code);
}

fn dispatch(ctx: &Ctx, command: &Command) -> Result<i32, Error> {
    match command {
        Command::Init(args) => commands::init::run(ctx, args).map(|_| 0),
        Command::Set(args) => commands::crud::set(ctx, args).map(|_| 0),
        Command::Get(args) => commands::crud::get(ctx, args).map(|_| 0),
        Command::List(args) => commands::crud::list(ctx, args).map(|_| 0),
        Command::Remove(args) => commands::crud::remove(ctx, args).map(|_| 0),
        Command::Status => commands::status::status(ctx).map(|ok| if ok { 0 } else { 3 }),
        Command::Doctor(args) => commands::status::doctor(ctx, args).map(|level| match level {
            Level::Ok | Level::Warn => 0,
            Level::Fail => 7,
        }),
        Command::Run(args) => commands::run::run(ctx, args),
        Command::Import(args) => commands::envfile::import(ctx, args).map(|_| 0),
        Command::Export(args) => commands::envfile::export(ctx, args).map(|_| 0),
        Command::Sync(args) => commands::sync::sync(ctx, args).map(|_| 0),
        Command::Share(args) => commands::bundle::share(ctx, args).map(|_| 0),
        Command::Accept(args) => commands::bundle::accept(ctx, args).map(|_| 0),
    }
}

fn exit_code(error: &Error) -> i32 {
    match error {
        Error::MissingSecret { .. } | Error::MissingSecrets { .. } => 3,
        Error::StoreUnavailable { .. } => 4,
        Error::ManifestNotFound { .. }
        | Error::ManifestInvalid { .. }
        | Error::ManifestTooNew { .. }
        | Error::CliTooOld { .. } => 5,
        Error::Crypto { .. } | Error::BundleFormat { .. } | Error::BundleMismatch { .. } => 6,
        Error::Cancelled => 8,
        _ => 1,
    }
}
