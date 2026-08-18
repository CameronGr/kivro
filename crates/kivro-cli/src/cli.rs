use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "kivro", version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args, Clone, Default)]
pub struct GlobalArgs {
    #[arg(long, short = 'e', global = true, value_name = "NAME")]
    pub env: Option<String>,

    #[arg(long, short = 'p', global = true, value_name = "PATH")]
    pub project: Option<PathBuf>,

    #[arg(long, global = true)]
    pub json: bool,

    #[arg(long, global = true)]
    pub no_color: bool,

    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init(InitArgs),

    Set(SetArgs),

    Get(GetArgs),

    List(ListArgs),

    Remove(RemoveArgs),

    Status,

    Doctor(DoctorArgs),

    #[command(disable_help_flag = true)]
    Run(RunArgs),

    Import(ImportArgs),

    Export(ExportArgs),

    Sync(SyncArgs),

    Share(ShareArgs),

    Accept(AcceptArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long, value_name = "NAME")]
    pub name: Option<String>,

    #[arg(long, default_value = "dev", value_name = "NAME")]
    pub default_env: String,

    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct SetArgs {
    pub name: String,

    #[arg(long)]
    pub stdin: bool,

    #[arg(long)]
    pub no_confirm: bool,
}

#[derive(Debug, Args)]
pub struct GetArgs {
    pub name: String,

    #[arg(long)]
    pub show: bool,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct RemoveArgs {
    pub name: String,

    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    #[arg(long)]
    pub fix_gitignore: bool,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[arg(long)]
    pub no_inherit: bool,

    #[arg(long, action = clap::ArgAction::Help)]
    pub help: Option<bool>,

    #[arg(trailing_var_arg = true, required = true, value_name = "COMMAND")]
    pub command: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    #[arg(default_value = ".env")]
    pub path: PathBuf,

    #[arg(long)]
    pub force: bool,

    #[arg(long)]
    pub delete_source: bool,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[arg(long, short = 'o', default_value = ".env")]
    pub out: PathBuf,

    #[arg(long)]
    pub force: bool,

    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct SyncArgs {
    #[arg(long)]
    pub apply: bool,
}

#[derive(Debug, Args)]
pub struct ShareArgs {
    #[arg(long, short = 'o', value_name = "PATH")]
    pub out: Option<PathBuf>,

    #[arg(long, value_name = "AGE_PUBLIC_KEY")]
    pub recipient: Vec<String>,

    #[arg(long)]
    pub all: bool,

    #[arg(long)]
    pub hint_names: bool,

    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct AcceptArgs {
    pub path: PathBuf,

    #[arg(long, value_name = "PATH")]
    pub identity: Option<PathBuf>,

    #[arg(long)]
    pub force: bool,
}
