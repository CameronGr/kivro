use std::io::{IsTerminal, Write};

use kivro_core::{Error, SecretString};

#[derive(Debug, Clone, Copy)]
pub struct Ui {
    color: bool,
    pub json: bool,
    pub quiet: bool,
}

impl Ui {
    pub fn new(color_preference: bool, no_color_flag: bool, json: bool, quiet: bool) -> Self {
        let color = color_preference
            && !no_color_flag
            && std::env::var_os("NO_COLOR").is_none()
            && std::io::stderr().is_terminal();
        Self { color, json, quiet }
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    // ✓ ✗ •

    pub fn present(&self) -> String {
        self.paint("32", "✓")
    }

    pub fn absent(&self) -> String {
        self.paint("31", "✗")
    }

    pub fn note_mark(&self) -> String {
        self.paint("33", "•")
    }

    pub fn bold(&self, text: &str) -> String {
        self.paint("1", text)
    }

    pub fn dim(&self, text: &str) -> String {
        self.paint("2", text)
    }

    pub fn info(&self, text: impl AsRef<str>) {
        if !self.quiet && !self.json {
            write_line(text.as_ref());
        }
    }

    pub fn blank(&self) {
        self.info("");
    }

    pub fn warn(&self, text: impl AsRef<str>) {
        eprintln!("{} {}", self.paint("33;1", "warning:"), text.as_ref());
    }

    pub fn error(&self, error: &Error) {
        eprintln!("{} {error}", self.paint("31;1", "error:"));
        if let Some(hint) = error.hint() {
            eprintln!();
            if hint.contains('\n') {
                eprintln!("{hint}");
            } else {
                eprintln!("{} {hint}", self.dim("hint:"));
            }
        }
    }

    pub fn json_value(&self, value: &serde_json::Value) {
        write_line(&serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into()));
    }

    pub fn confirm(&self, question: &str, default_yes: bool) -> bool {
        if !std::io::stdin().is_terminal() {
            return false;
        }
        let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
        eprint!("{question} {suffix} ");
        let _ = std::io::stderr().flush();

        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_err() {
            return false;
        }
        match answer.trim().to_ascii_lowercase().as_str() {
            "" => default_yes,
            "y" | "yes" => true,
            _ => false,
        }
    }
}

fn write_line(text: &str) {
    let mut out = std::io::stdout().lock();
    if let Err(e) = writeln!(out, "{text}") {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
    }
}

pub fn prompt_secret(prompt: &str) -> Result<SecretString, Error> {
    let value = rpassword::prompt_password(prompt)
        .map_err(|e| Error::Other(format!("cannot read from the terminal: {e}")))?;
    Ok(SecretString::new(value))
}

pub fn prompt_secret_confirmed(prompt: &str) -> Result<SecretString, Error> {
    let first = prompt_secret(prompt)?;
    let second = prompt_secret("Confirm: ")?;
    if first != second {
        return Err(Error::Other("the values did not match".into()));
    }
    Ok(first)
}

pub const PASSPHRASE_ENV: &str = "KIVRO_PASSPHRASE";

pub fn passphrase(prompt: &str) -> Result<SecretString, Error> {
    if let Some(value) = std::env::var_os(PASSPHRASE_ENV) {
        return Ok(SecretString::new(value.to_string_lossy().into_owned()));
    }
    if !stdin_is_tty() {
        return Err(Error::Other(format!(
            "a passphrase is required but this is not a terminal; set {PASSPHRASE_ENV} or pass an age identity"
        )));
    }
    prompt_secret(prompt)
}

pub fn stdin_is_tty() -> bool {
    std::io::stdin().is_terminal()
}

pub fn read_secret_from_stdin() -> Result<SecretString, Error> {
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(Error::RawIo)?;
    if buffer.ends_with('\n') {
        buffer.pop();
        if buffer.ends_with('\r') {
            buffer.pop();
        }
    }
    Ok(SecretString::new(buffer))
}
