//! child process execution while inecting secrets

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::process::Command;

use kivro_core::{Error, Result};

use crate::SecretSet;

/// construction of child env
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// pass all parent env through if true
    pub inherit_environment: bool,
    /// extra, non-secret variables to set
    pub extra: BTreeMap<String, String>,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            inherit_environment: true,
            extra: BTreeMap::new(),
        }
    }
}

/// Variables preserved when `inherit_environment` is `false`.
const MINIMAL_PASSTHROUGH: &[&str] = &[
    "PATH",
    "HOME",
    "USER",
    "SHELL",
    "TMPDIR",
    "TEMP",
    "TMP",
    "LANG",
    "LC_ALL",
    "TERM",
    "SYSTEMROOT",
    "WINDIR",
    "PATHEXT",
    "COMSPEC",
    "USERPROFILE",
];

/// build the subprocess cammand
pub fn command(
    program: &str,
    args: &[String],
    secrets: &SecretSet,
    options: &RunOptions,
) -> Command {
    let mut cmd = Command::new(program);
    cmd.args(args);

    if !options.inherit_environment {
        let preserved: Vec<(OsString, OsString)> = MINIMAL_PASSTHROUGH
            .iter()
            .filter_map(|k| std::env::var_os(k).map(|v| (OsString::from(k), v)))
            .collect();
        cmd.env_clear();
        cmd.envs(preserved);
    }

    for (key, value) in &options.extra {
        cmd.env(key, value);
    }

    cmd.envs(secrets.environment());
    cmd
}

/// spawn the program
pub fn run(
    program: &str,
    args: &[String],
    secrets: &SecretSet,
    options: &RunOptions,
) -> Result<i32> {
    let mut child = command(program, args, secrets, options)
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::Other(format!("command not found: `{program}`"))
            } else {
                Error::Other(format!("cannot run `{program}`: {e}"))
            }
        })?;

    let status = child
        .wait()
        .map_err(|e| Error::Other(format!("waiting for `{program}`: {e}")))?;

    if let Some(code) = status.code() {
        return Ok(code);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Ok(128 + signal);
        }
    }
    Ok(1)
}
