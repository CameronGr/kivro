use kivro::Project;

fn main() {}

fn start() -> Result<(), secrets_core::Error> {
    // Walks up from the current directory for `.kivro.toml`.
    let project = Project::discover()?;

    // `None` means: follow the documented precedence — KIVRO_ENV, then the
    // manifest default, then the global config default.
    let env = project.resolve_environment(None)?;

    println!("project:     {}", project.name());
    println!("environment: {}", env.name());

    // A readiness check that reports everything wrong at once, rather than
    // failing on the first missing variable.
    let status = env.status()?;
    if !status.is_satisfied() {
        for entry in status.missing_required() {
            eprintln!("missing: {}", entry.name);
        }
    }

    // `load` enforces `required = true`; `load_available` does not.
    let secrets = env.load()?;
    println!("loaded {} secret(s)", secrets.len());

    // `get` returns a `&SecretString`. It has no Display and no Serialize, and
    // its Debug is redacted, so it cannot reach a log by accident.
    let database_url = secrets.get("DATABASE_URL")?;
    println!("DATABASE_URL is {} bytes", database_url.len());

    // Debug on the whole set prints names only — try it.
    println!("{secrets:?}");

    // `expose_secret()` is the one audited way out. Keep the call adjacent to
    // the thing that consumes it.
    connect(database_url.expose_secret());

    // Handing the whole set to a child process:
    //
    //   std::process::Command::new("cargo")
    //       .args(["run"])
    //       .envs(secrets.environment())
    //       .spawn()?;
    //
    // or, with exit-code propagation already handled:
    //
    //   let code = kivro::run::run("cargo", &["run".into()], &secrets,
    //                                &kivro::run::RunOptions::default())?;

    Ok(())
}

fn connect(dsn: &str) {
    // Pretend this is a database driver. Note what is *not* here: no logging of
    // the DSN, and no storing it in a struct that derives Debug.
    let _ = dsn;
    println!("connected");
}
