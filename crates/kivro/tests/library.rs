use std::collections::BTreeMap;

use kivro::config::Config;
use kivro::{ENV_OVERRIDE, Project};
use kivro_core::{MemoryStore, SecretName, SecretString};
use kivro_manifest::Manifest;

const MANIFEST: &str = r#"
[project]
name = "infinity-launcher"

[environment]
default = "dev"

[variables]
DATABASE_URL = { required = true }
AUTH0_CLIENT_SECRET = { required = true }
SENTRY_DSN = { required = false }

[environments.production]
SENTRY_DSN = { required = true }
"#;

fn project() -> Project {
    let manifest = Manifest::parse("/tmp/.kivro.toml", MANIFEST).unwrap();
    Project::new(manifest, Box::new(MemoryStore::new()), Config::default())
}

fn name(n: &str) -> SecretName {
    SecretName::new(n).unwrap()
}

/// `sh` treats backslashes as escapes, so a native Windows path has to be
/// forward-slashed before it can be embedded in a command string.
fn shell_path(path: &std::path::Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// `KIVRO_ENV` is process-global, so tests that mutate it cannot run
/// concurrently with each other.
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn env_guard() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[test]
fn loading_reports_every_missing_required_secret_at_once() {
    let p = project();
    let env = p.environment("dev").unwrap();
    env.set(&name("DATABASE_URL"), &SecretString::new("postgres://x"))
        .unwrap();

    let err = env.load().unwrap_err();
    assert_eq!(err.kind(), "missing_secret");
    assert!(err.to_string().contains("AUTH0_CLIENT_SECRET"));
    assert!(
        err.hint()
            .unwrap()
            .contains("kivro set AUTH0_CLIENT_SECRET")
    );

    env.set(&name("AUTH0_CLIENT_SECRET"), &SecretString::new("shh"))
        .unwrap();
    let set = env.load().unwrap();
    assert_eq!(
        set.get("DATABASE_URL").unwrap().expose_secret(),
        "postgres://x"
    );
    assert!(set.find("SENTRY_DSN").is_none());
}

#[test]
fn status_distinguishes_required_optional_and_undeclared() {
    let p = project();
    let env = p.environment("dev").unwrap();
    env.set(&name("DATABASE_URL"), &SecretString::new("x"))
        .unwrap();
    env.set(&name("LEFTOVER_KEY"), &SecretString::new("y"))
        .unwrap();

    let status = env.status().unwrap();
    assert!(!status.is_satisfied());
    assert_eq!(status.missing_required().len(), 1);
    assert_eq!(
        status.missing_required()[0].name.as_str(),
        "AUTH0_CLIENT_SECRET"
    );
    assert_eq!(status.undeclared().len(), 1);
    assert_eq!(status.undeclared()[0].name.as_str(), "LEFTOVER_KEY");
}

#[test]
fn environments_are_isolated_from_each_other() {
    let p = project();
    p.environment("dev")
        .unwrap()
        .set(&name("DATABASE_URL"), &SecretString::new("dev-db"))
        .unwrap();
    p.environment("production")
        .unwrap()
        .set(&name("DATABASE_URL"), &SecretString::new("prod-db"))
        .unwrap();

    assert_eq!(
        p.environment("dev")
            .unwrap()
            .get(&name("DATABASE_URL"))
            .unwrap()
            .unwrap()
            .expose_secret(),
        "dev-db"
    );
    assert_eq!(
        p.environment("production")
            .unwrap()
            .get(&name("DATABASE_URL"))
            .unwrap()
            .unwrap()
            .expose_secret(),
        "prod-db"
    );
}

#[test]
#[allow(unsafe_code)]
fn environment_precedence_is_cli_then_env_var_then_manifest() {
    let _guard = env_guard();
    let p = project();
    unsafe {
        std::env::remove_var(ENV_OVERRIDE);
    }
    assert_eq!(p.resolve_environment(None).unwrap().name().as_str(), "dev");
    unsafe {
        std::env::set_var(ENV_OVERRIDE, "production");
    }
    assert_eq!(
        p.resolve_environment(None).unwrap().name().as_str(),
        "production"
    );
    // An explicit selection still wins over the variable.
    assert_eq!(
        p.resolve_environment(Some("dev")).unwrap().name().as_str(),
        "dev"
    );
    unsafe {
        std::env::remove_var(ENV_OVERRIDE);
    }
}

#[test]
#[allow(unsafe_code)]
fn global_config_default_does_not_override_the_manifest() {
    let _guard = env_guard();
    let manifest = Manifest::parse("/tmp/.kivro.toml", MANIFEST).unwrap();
    let mut config = Config::default();
    config.defaults.environment = Some("production".into());
    let p = Project::new(manifest, Box::new(MemoryStore::new()), config);
    unsafe {
        std::env::remove_var(ENV_OVERRIDE);
    }
    assert_eq!(p.resolve_environment(None).unwrap().name().as_str(), "dev");
}

#[test]
#[allow(unsafe_code)]
fn global_config_default_applies_when_the_manifest_has_none() {
    let _guard = env_guard();
    let manifest = Manifest::parse(
        "/tmp/.kivro.toml",
        "[project]\nname=\"p\"\n[environment]\nstrict=false\n",
    )
    .unwrap();
    let mut config = Config::default();
    config.defaults.environment = Some("staging".into());
    let p = Project::new(manifest, Box::new(MemoryStore::new()), config);
    unsafe {
        std::env::remove_var(ENV_OVERRIDE);
    }
    assert_eq!(
        p.resolve_environment(None).unwrap().name().as_str(),
        "staging"
    );
}

#[test]
#[allow(unsafe_code)]
fn missing_environment_selection_is_an_actionable_error() {
    let _guard = env_guard();
    let manifest = Manifest::parse(
        "/tmp/.kivro.toml",
        "[project]\nname=\"p\"\n[environment]\nstrict=false\n",
    )
    .unwrap();
    let p = Project::new(manifest, Box::new(MemoryStore::new()), Config::default());
    unsafe {
        std::env::remove_var(ENV_OVERRIDE);
    }
    let err = p.resolve_environment(None).unwrap_err();
    assert_eq!(err.kind(), "no_environment");
    assert!(err.hint().unwrap().contains("--env"));
}

#[test]
fn secret_sets_never_debug_print_values() {
    let p = project();
    let env = p.environment("dev").unwrap();
    env.set(
        &name("DATABASE_URL"),
        &SecretString::new("super-secret-dsn"),
    )
    .unwrap();
    let set = env.load_available().unwrap();

    let rendered = format!("{set:?}");
    assert!(rendered.contains("DATABASE_URL"));
    assert!(!rendered.contains("super-secret-dsn"));
}

#[test]
fn secret_sets_convert_to_process_environment() {
    let mut values = BTreeMap::new();
    values.insert(name("A"), SecretString::new("1"));
    let set = kivro::SecretSet::from_values(
        kivro_core::ProjectName::new("p").unwrap(),
        kivro_core::EnvironmentName::new("dev").unwrap(),
        values,
    );
    assert_eq!(set.environment(), vec![("A".to_string(), "1".to_string())]);
}

#[test]
fn running_a_child_receives_the_secrets_and_propagates_exit_codes() {
    let p = project();
    let env = p.environment("dev").unwrap();
    env.set(&name("DATABASE_URL"), &SecretString::new("injected-value"))
        .unwrap();
    env.set(&name("AUTH0_CLIENT_SECRET"), &SecretString::new("x"))
        .unwrap();
    let set = env.load().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.txt");

    let options = kivro::run::RunOptions::default();
    let code = kivro::run::run(
        "sh",
        &[
            "-c".into(),
            format!("printf '%s' \"$DATABASE_URL\" > '{}'", shell_path(&out)),
        ],
        &set,
        &options,
    )
    .unwrap();
    assert_eq!(code, 0);
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "injected-value");

    let code = kivro::run::run("sh", &["-c".into(), "exit 42".into()], &set, &options).unwrap();
    assert_eq!(code, 42);

    let err = kivro::run::run("definitely-not-a-real-command", &[], &set, &options).unwrap_err();
    assert!(err.to_string().contains("command not found"));
}

#[test]
#[allow(unsafe_code)]
fn non_inheriting_runs_drop_the_parent_environment() {
    let p = project();
    let env = p.environment("dev").unwrap();
    env.set(&name("DATABASE_URL"), &SecretString::new("v"))
        .unwrap();
    env.set(&name("AUTH0_CLIENT_SECRET"), &SecretString::new("v"))
        .unwrap();
    let set = env.load().unwrap();

    unsafe {
        std::env::set_var("KIVRO_TEST_LEAKED", "leaked");
    }
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.txt");

    let options = kivro::run::RunOptions {
        inherit_environment: false,
        extra: Default::default(),
    };
    kivro::run::run(
        "sh",
        &[
            "-c".into(),
            format!(
                "printf '%s' \"${{KIVRO_TEST_LEAKED:-none}}\" > '{}'",
                shell_path(&out)
            ),
        ],
        &set,
        &options,
    )
    .unwrap();
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "none");
    unsafe {
        std::env::remove_var("KIVRO_TEST_LEAKED");
    }
}
