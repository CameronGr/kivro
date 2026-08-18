use kivro::run::RunOptions;
use kivro_core::Result;

use crate::cli::RunArgs;

use super::Ctx;

pub fn run(ctx: &Ctx, args: &RunArgs) -> Result<i32> {
    let project = ctx.projet()?;
    let env = ctx.environment(&project)?;

    let secrets = env.load()?;

    let (program, rest) = args
        .command
        .split_first()
        .expect("clap enforces at least one aguement");

    if !ctx.ui.quiet {
        ctx.ui.info(ctx.ui.dim(&format!(
            "{} / {} — {} secret{} injected",
            env.project_name(),
            env.name(),
            secrets.len(),
            if secrets.len() == 1 { "" } else { "s" }
        )));
    }

    let _ = ctrlc::set_handler(|| {});

    let options = RunOptions {
        inherit_environment: !args.no_inherit,
        extra: Default::default(),
    };
    kivro::run::run(program, rest, &secrets, &options)
}
