use std::path::{Path, PathBuf};

use assert_cmd::Command;
use assert_cmd::prelude::*;
use tempfile::TempDir;

const MANIFEST: &str = r#"
[meta]
format = 1

[project]
name = "infinity-launcher"

[environment]
default = "dev"

[variables]
DATABASE_URL = { required = true }
AUTH0_CLIENT_ID = { required = true }
SENTRY_DSN = { required = false }

[environments.production]
SENTRY_DSN = { required = true }
"#;

struct Fixture {
    dir: TempDir,
    store: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join(".kivro.toml"), MANIFEST).unwrap();
        let store = dir.path().join("store.json");
        Self { dir, store }
    }

    fn empty() -> Self {
        let dir = TempDir::new().unwrap();
        let store = dir.path().join("store.json");
        Self { dir, store }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    /// A `kivro` invocation rooted in the fixture directory.
    fn cmd(&self) -> Command {
        self.cmd_in(self.path(), &self.store)
    }

    /// An invocation with an explicit working directory and store.
    fn cmd_in(&self, cwd: &Path, store: &Path) -> Command {
        let mut cmd = Command::cargo_bin("kivro").unwrap();
        // `assert_cmd::Command` is used instead of `std::process::Command` for
        // `write_stdin`, which is how `kivro set --stdin` is driven here.
        cmd.current_dir(cwd)
            .env("KIVRO_STORE", "file")
            .env("KIVRO_STORE_FILE", store)
            .env("KIVRO_CONFIG_DIR", self.path().join("config"))
            .env_remove("KIVRO_ENV")
            .env_remove("KIVRO_PASSPHRASE")
            .env("NO_COLOR", "1");
        cmd
    }

    fn set(&self, name: &str, value: &str) {
        let mut cmd = self.cmd();
        cmd.args(["set", name, "--stdin"]).write_stdin(value);
        cmd.assert().success();
    }
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

// ---------------------------------------------------------------------------
// init
// ---------------------------------------------------------------------------

#[test]
fn init_creates_a_manifest_and_refuses_to_clobber_it() {
    let f = Fixture::empty();
    f.cmd()
        .args(["init", "--name", "demo-project"])
        .assert()
        .success();

    let manifest = std::fs::read_to_string(f.path().join(".kivro.toml")).unwrap();
    assert!(manifest.contains("name = \"demo-project\""));
    assert!(
        !f.path().join(".env").exists(),
        "init must never create a .env"
    );

    // Second run leaves the file alone.
    std::fs::write(f.path().join(".kivro.toml"), "[project]\nname = \"kept\"\n").unwrap();
    let output = f.cmd().arg("init").output().unwrap();
    assert!(!output.status.success());
    assert!(stderr(&output).contains("already exists"));
    assert!(
        std::fs::read_to_string(f.path().join(".kivro.toml"))
            .unwrap()
            .contains("kept")
    );

    f.cmd()
        .args(["init", "--name", "demo-project", "--force"])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// discovery, status, list
// ---------------------------------------------------------------------------

#[test]
fn commands_discover_the_manifest_from_a_subdirectory() {
    let f = Fixture::new();
    let nested = f.path().join("crates").join("app").join("src");
    std::fs::create_dir_all(&nested).unwrap();

    let output = f.cmd_in(&nested, &f.store).arg("status").output().unwrap();
    assert!(stdout(&output).contains("infinity-launcher"));
}

#[test]
fn status_exit_code_reflects_missing_required_secrets() {
    let f = Fixture::new();

    let output = f.cmd().arg("status").output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(3),
        "missing secrets must fail the build"
    );
    assert!(stdout(&output).contains("DATABASE_URL"));

    f.set("DATABASE_URL", "postgres://localhost/app");
    f.set("AUTH0_CLIENT_ID", "abc123");

    let output = f.cmd().arg("status").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("all required secrets are present"));
}

#[test]
fn list_shows_names_and_presence_but_never_values() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "postgres://super-secret-host/db");

    let output = f.cmd().arg("list").output().unwrap();
    let text = stdout(&output);
    assert!(text.contains("DATABASE_URL"));
    assert!(text.contains("AUTH0_CLIENT_ID"));
    assert!(!text.contains("super-secret-host"));
}

#[test]
fn json_output_is_machine_readable_and_value_free() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "postgres://super-secret-host/db");

    for args in [
        vec!["status", "--json"],
        vec!["list", "--json"],
        vec!["doctor", "--json"],
    ] {
        let output = f.cmd().args(&args).output().unwrap();
        let text = stdout(&output);
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{args:?} produced invalid JSON: {e}\n{text}"));
        assert!(
            !text.contains("super-secret-host"),
            "{args:?} leaked a value"
        );
        assert!(parsed.is_object());
    }
}

// ---------------------------------------------------------------------------
// set / get / remove
// ---------------------------------------------------------------------------

#[test]
fn get_withholds_the_value_unless_show_is_passed() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "postgres://localhost/app");

    let output = f.cmd().args(["get", "DATABASE_URL"]).output().unwrap();
    output.clone().assert().success();
    assert!(!stdout(&output).contains("postgres://localhost/app"));
    assert!(stdout(&output).contains("is set"));

    let output = f
        .cmd()
        .args(["get", "DATABASE_URL", "--show"])
        .output()
        .unwrap();
    assert_eq!(stdout(&output).trim_end(), "postgres://localhost/app");
}

#[test]
fn getting_an_absent_secret_is_an_actionable_error() {
    let f = Fixture::new();
    let output = f.cmd().args(["get", "DATABASE_URL"]).output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(stderr(&output).contains("kivro set DATABASE_URL"));
}

#[test]
fn stdin_values_keep_internal_whitespace_and_drop_one_trailing_newline() {
    let f = Fixture::new();
    let mut cmd = f.cmd();
    cmd.args(["set", "DATABASE_URL", "--stdin"])
        .write_stdin("line one\nline two\n");
    cmd.assert().success();

    let output = f
        .cmd()
        .args(["get", "DATABASE_URL", "--show"])
        .output()
        .unwrap();
    assert_eq!(stdout(&output), "line one\nline two\n");
}

#[test]
fn setting_an_undeclared_secret_warns_but_succeeds() {
    let f = Fixture::new();
    let mut cmd = f.cmd();
    cmd.args(["set", "NOT_IN_MANIFEST", "--stdin"])
        .write_stdin("x");
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    assert!(stderr(&output).contains("not declared"));

    // It shows up only with --all.
    let output = f.cmd().arg("list").output().unwrap();
    assert!(!stdout(&output).contains("NOT_IN_MANIFEST"));
    let output = f.cmd().args(["list", "--all"]).output().unwrap();
    assert!(stdout(&output).contains("NOT_IN_MANIFEST"));
}

#[test]
fn remove_deletes_a_value() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "x");
    f.cmd()
        .args(["remove", "DATABASE_URL", "--yes"])
        .assert()
        .success();
    let output = f.cmd().args(["get", "DATABASE_URL"]).output().unwrap();
    assert_eq!(output.status.code(), Some(3));
}

// ---------------------------------------------------------------------------
// environments
// ---------------------------------------------------------------------------

#[test]
fn environments_are_separate_and_selectable() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "dev-db");

    let mut cmd = f.cmd();
    cmd.args(["--env", "production", "set", "DATABASE_URL", "--stdin"])
        .write_stdin("prod-db");
    cmd.assert().success();

    let output = f
        .cmd()
        .args(["get", "DATABASE_URL", "--show"])
        .output()
        .unwrap();
    assert_eq!(stdout(&output).trim_end(), "dev-db");

    let output = f
        .cmd()
        .args(["--env", "production", "get", "DATABASE_URL", "--show"])
        .output()
        .unwrap();
    assert_eq!(stdout(&output).trim_end(), "prod-db");

    // KIVRO_ENV works too, and --env beats it.
    let output = f
        .cmd()
        .env("KIVRO_ENV", "production")
        .args(["get", "DATABASE_URL", "--show"])
        .output()
        .unwrap();
    assert_eq!(stdout(&output).trim_end(), "prod-db");

    let output = f
        .cmd()
        .env("KIVRO_ENV", "production")
        .args(["--env", "dev", "get", "DATABASE_URL", "--show"])
        .output()
        .unwrap();
    assert_eq!(stdout(&output).trim_end(), "dev-db");
}

#[test]
fn an_undeclared_environment_is_rejected() {
    let f = Fixture::new();
    let output = f.cmd().args(["--env", "prod", "status"]).output().unwrap();
    assert!(!output.status.success());
    assert!(stderr(&output).contains("unknown environment"));
    assert!(stderr(&output).contains("production"));
}

#[test]
fn a_missing_manifest_is_reported_with_the_fix() {
    let f = Fixture::empty();
    let output = f.cmd().arg("status").output().unwrap();
    assert_eq!(output.status.code(), Some(5));
    assert!(stderr(&output).contains("kivro init"));
}

// ---------------------------------------------------------------------------
// run
// ---------------------------------------------------------------------------

#[test]
fn run_injects_secrets_and_propagates_the_exit_code() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "injected-dsn");
    f.set("AUTH0_CLIENT_ID", "injected-id");

    let output = f
        .cmd()
        .args([
            "run",
            "--",
            "sh",
            "-c",
            "printf '%s|%s' \"$DATABASE_URL\" \"$AUTH0_CLIENT_ID\"",
        ])
        .output()
        .unwrap();
    assert!(stdout(&output).contains("injected-dsn|injected-id"));

    let output = f
        .cmd()
        .args(["run", "--", "sh", "-c", "exit 23"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(23));
}

#[test]
fn run_refuses_to_start_when_a_required_secret_is_missing() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "x");

    let output = f
        .cmd()
        .args(["run", "--", "sh", "-c", "echo should-not-run"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(!stdout(&output).contains("should-not-run"));
    assert!(stderr(&output).contains("AUTH0_CLIENT_ID"));
}

#[test]
fn run_never_writes_an_env_file() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "x");
    f.set("AUTH0_CLIENT_ID", "y");
    f.cmd()
        .args(["run", "--", "sh", "-c", "true"])
        .assert()
        .success();
    assert!(!f.path().join(".env").exists());
}

// ---------------------------------------------------------------------------
// import / export
// ---------------------------------------------------------------------------

#[test]
fn import_and_export_round_trip() {
    let f = Fixture::new();
    std::fs::write(
        f.path().join(".env"),
        "# comment\nDATABASE_URL=postgres://imported/db\nAUTH0_CLIENT_ID=\"quoted id\"\nlowercase=ignored\n",
    )
    .unwrap();

    let output = f.cmd().args(["import", ".env"]).output().unwrap();
    assert!(output.status.success());
    assert!(
        stderr(&output).contains("lowercase"),
        "invalid keys must be reported"
    );
    // Import never deletes the source on its own.
    assert!(f.path().join(".env").exists());

    let output = f
        .cmd()
        .args(["get", "AUTH0_CLIENT_ID", "--show"])
        .output()
        .unwrap();
    assert_eq!(stdout(&output).trim_end(), "quoted id");

    std::fs::remove_file(f.path().join(".env")).unwrap();
    f.cmd().args(["export", "--yes"]).assert().success();
    let written = std::fs::read_to_string(f.path().join(".env")).unwrap();
    assert!(written.contains("DATABASE_URL=\"postgres://imported/db\""));
    assert!(written.contains("WARNING"));

    // Existing files are not overwritten without --force.
    let output = f.cmd().args(["export", "--yes"]).output().unwrap();
    assert!(!output.status.success());
    f.cmd()
        .args(["export", "--yes", "--force"])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// share / accept
// ---------------------------------------------------------------------------

#[test]
fn share_and_accept_move_secrets_between_developers() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "shared-dsn");
    f.set("AUTH0_CLIENT_ID", "shared-id");

    f.cmd()
        .env("KIVRO_PASSPHRASE", "correct horse battery staple")
        .args(["share", "--out", "bundle.kivro"])
        .assert()
        .success();

    let bundle = std::fs::read_to_string(f.path().join("bundle.kivro")).unwrap();
    assert!(!bundle.contains("shared-dsn"));
    assert!(
        !bundle.contains("DATABASE_URL"),
        "names are withheld by default"
    );

    // A second developer: same project, a different credential store.
    let other_store = f.path().join("other-store.json");
    f.cmd_in(f.path(), &other_store)
        .env("KIVRO_PASSPHRASE", "correct horse battery staple")
        .args(["accept", "bundle.kivro"])
        .assert()
        .success();

    let output = f
        .cmd_in(f.path(), &other_store)
        .args(["get", "DATABASE_URL", "--show"])
        .output()
        .unwrap();
    assert_eq!(stdout(&output).trim_end(), "shared-dsn");

    // Wrong passphrase, and the exit code says why.
    let third_store = f.path().join("third-store.json");
    let output = f
        .cmd_in(f.path(), &third_store)
        .env("KIVRO_PASSPHRASE", "wrong")
        .args(["accept", "bundle.kivro"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(6));
    assert!(stderr(&output).contains("wrong passphrase"));
}

#[test]
fn share_and_accept_work_with_age_recipients() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "keyed-dsn");
    f.set("AUTH0_CLIENT_ID", "keyed-id");

    let identity = age::x25519::Identity::generate();
    let recipient = identity.to_public().to_string();
    let identity_path = f.path().join("key.txt");
    {
        use age::secrecy::ExposeSecret;
        std::fs::write(
            &identity_path,
            format!("{}\n", identity.to_string().expose_secret()),
        )
        .unwrap();
    }

    f.cmd()
        .args(["share", "--out", "keyed.kivro", "--recipient", &recipient])
        .assert()
        .success();

    let other_store = f.path().join("other-store.json");
    f.cmd_in(f.path(), &other_store)
        .args([
            "accept",
            "keyed.kivro",
            "--identity",
            identity_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let output = f
        .cmd_in(f.path(), &other_store)
        .args(["get", "DATABASE_URL", "--show"])
        .output()
        .unwrap();
    assert_eq!(stdout(&output).trim_end(), "keyed-dsn");
}

#[test]
fn accepting_a_bundle_from_another_project_is_refused() {
    let source = Fixture::new();
    source.set("DATABASE_URL", "x");
    source.set("AUTH0_CLIENT_ID", "y");
    source
        .cmd()
        .env("KIVRO_PASSPHRASE", "pass pass pass pass")
        .args(["share", "--out", "bundle.kivro"])
        .assert()
        .success();

    let other = Fixture::empty();
    std::fs::write(
        other.path().join(".kivro.toml"),
        "[project]\nname = \"a-different-project\"\n[environment]\ndefault = \"dev\"\n",
    )
    .unwrap();
    std::fs::copy(
        source.path().join("bundle.kivro"),
        other.path().join("bundle.kivro"),
    )
    .unwrap();

    let output = other
        .cmd()
        .env("KIVRO_PASSPHRASE", "pass pass pass pass")
        .args(["accept", "bundle.kivro"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(6));
    assert!(stderr(&output).contains("a-different-project"));
}

// ---------------------------------------------------------------------------
// sync
// ---------------------------------------------------------------------------

#[test]
fn sync_reports_and_then_fetches_missing_secrets() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "synced-dsn");
    f.set("AUTH0_CLIENT_ID", "synced-id");

    std::fs::create_dir_all(f.path().join("team")).unwrap();
    f.cmd()
        .env("KIVRO_PASSPHRASE", "team passphrase here")
        .args(["share", "--out", "team/infinity-launcher.dev.kivro"])
        .assert()
        .success();

    let mut manifest = std::fs::read_to_string(f.path().join(".kivro.toml")).unwrap();
    manifest.push_str("\n[sync]\nkind = \"file\"\npath = \"team\"\n");
    std::fs::write(f.path().join(".kivro.toml"), manifest).unwrap();

    // A fresh developer with an empty store.
    let fresh = f.path().join("fresh-store.json");
    let output = f
        .cmd_in(f.path(), &fresh)
        .env("KIVRO_PASSPHRASE", "team passphrase here")
        .args(["sync", "--json"])
        .output()
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(parsed["fetchable"].as_array().unwrap().len(), 2);
    assert!(
        parsed["fetched"].as_array().unwrap().is_empty(),
        "sync must not write without --apply"
    );

    f.cmd_in(f.path(), &fresh)
        .env("KIVRO_PASSPHRASE", "team passphrase here")
        .args(["sync", "--apply"])
        .assert()
        .success();

    let output = f.cmd_in(f.path(), &fresh).arg("status").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
}

// ---------------------------------------------------------------------------
// doctor
// ---------------------------------------------------------------------------

#[test]
fn doctor_flags_an_unignored_env_file_and_passes_once_clean() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "x");
    f.set("AUTH0_CLIENT_ID", "y");

    let output = f.cmd().arg("doctor").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("no stray"));

    std::fs::write(
        f.path().join(".env"),
        "AUTH0_CLIENT_SECRET=8f3a9c2b1d4e5f60\n",
    )
    .unwrap();
    let output = f.cmd().arg("doctor").output().unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert!(stdout(&output).contains(".gitignore"));

    std::fs::write(f.path().join(".gitignore"), ".env\n").unwrap();
    let output = f.cmd().arg("doctor").output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "an ignored .env is a warning, not a failure"
    );
}

#[test]
fn doctor_recommends_ignoring_the_real_bundle_extension() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "x");
    f.set("AUTH0_CLIENT_ID", "y");
    std::fs::write(f.path().join(".env"), "PORT=3000\n").unwrap();

    let output = f.cmd().arg("doctor").output().unwrap();
    let text = stdout(&output);
    assert!(
        text.contains(&format!("*.{}", kivro_crypto::BUNDLE_EXTENSION)),
        "the recommendation must name the extension bundles are written with:\n{text}"
    );
}

#[test]
fn doctor_notices_bundles_left_in_the_project_root() {
    let f = Fixture::new();
    f.set("DATABASE_URL", "x");
    f.set("AUTH0_CLIENT_ID", "y");

    f.cmd()
        .env("KIVRO_PASSPHRASE", "correct horse battery staple")
        .args(["share"])
        .assert()
        .success();

    // `share` with no --out picks the suggested filename; doctor has to
    // recognise it as a bundle.
    let output = f.cmd().arg("doctor").output().unwrap();
    let text = stdout(&output);
    assert!(
        text.contains("encrypted bundles in the project root"),
        "a shared bundle left in the root must be flagged:\n{text}"
    );
    assert!(text.contains(&kivro_crypto::suggested_filename(
        &kivro_core::ProjectName::new("infinity-launcher").unwrap(),
        &kivro_core::EnvironmentName::new("dev").unwrap(),
    )));
}

#[test]
fn doctor_reports_a_manifest_that_needs_a_newer_cli() {
    let f = Fixture::empty();
    std::fs::write(
        f.path().join(".kivro.toml"),
        "[meta]\nformat = 99\n[project]\nname = \"future\"\n",
    )
    .unwrap();

    let output = f.cmd().arg("status").output().unwrap();
    assert_eq!(output.status.code(), Some(5));
    assert!(stderr(&output).contains("format version 99"));
}
