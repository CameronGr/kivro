//! validated identifers

use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

fn validate(
    kind: &'static str,
    value: &str,
    max: usize,
    first: fn(char) -> bool,
    rest: fn(char) -> bool,
) -> Result<()> {
    if value.is_empty() {
        return Err(Error::InvalidName {
            kind,
            value: value.to_string(),
            reason: "must not be empty".into(),
        });
    }
    if value.len() > max {
        return Err(Error::InvalidName {
            kind,
            value: value.to_string(),
            reason: format!("must be at most {max} characters"),
        });
    }
    let mut chars = value.chars();
    let head = chars.next().expect("non-empty");
    if !first(head) {
        return Err(Error::InvalidName {
            kind,
            value: value.to_string(),
            reason: format!("must not start with `{head}`"),
        });
    }

    for c in chars {
        if !rest(c) {
            return Err(Error::InvalidName {
                kind,
                value: value.to_string(),
                reason: format!("contains unsupported characcter `{c}`"),
            });
        }
    }
    Ok(())
}

macro_rules! name_type {
    ($name:ident, $kind:literal, $max:literal, $first:expr, $rest:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// construct
            pub fn new(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                validate($kind, &value, $max, $first, $rest)?;
                Ok(Self(value))
            }

            /// a validated string
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = Error;
            fn from_str(s: &str) -> Result<Self> {
                Self::new(s)
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;
            fn try_from(s: String) -> Result<Self> {
                Self::new(s)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(
                &self,
                s: S,
            ) -> std::result::Result<S::Ok, S::Error> {
                s.serialize_str(&self.0)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(
                d: D,
            ) -> std::result::Result<Self, D::Error> {
                let raw = String::deserialize(d)?;
                Self::new(raw).map_err(serde::de::Error::custom)
            }
        }
    };
}

fn alnum(c: char) -> bool {
    c.is_ascii_alphanumeric()
}
fn alnum_punct(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')
}
fn upper_start(c: char) -> bool {
    c.is_ascii_uppercase() || c == '_'
}
fn upper_rest(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'
}

name_type!(
    ProjectName,
    "project name",
    64,
    alnum,
    alnum_punct,
    "A project identifier, e.g. `infinity-launcher`."
);
name_type!(
    EnvironmentName,
    "environment name",
    32,
    alnum,
    alnum_punct,
    "An environment identifier, e.g. `dev`."
);
name_type!(
    SecretName,
    "secret name",
    128,
    upper_start,
    upper_rest,
    "An environment-variable style secret name, e.g. `AUTH0_CLIENT_SECRET`.\n\nSecret names are restricted to `[A-Z_][A-Z0-9_]*`. Beyond matching POSIX\nenvironment variable conventions, the uppercase rule is what lets the manifest\nformat distinguish variable declarations from (lowercase) settings keys inside\nan `[environments.<name>]` table, and guarantees user data can never collide\nwith the lowercase internal keys used by the keyring backend."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_typical_names() {
        assert!(ProjectName::new("infinity-launcher").is_ok());
        assert!(ProjectName::new("md10.systems").is_ok());
        assert!(EnvironmentName::new("production").is_ok());
        assert!(SecretName::new("AUTH0_CLIENT_SECRET").is_ok());
        assert!(SecretName::new("_PRIVATE").is_ok());
        assert!(SecretName::new("S3_KEY_2").is_ok());
    }

    #[test]
    fn rejects_namespace_separator() {
        // no name may use the ':' separator
        assert!(ProjectName::new("a:b").is_err());
        assert!(EnvironmentName::new("dev:prod").is_err());
        assert!(SecretName::new("A:B").is_err());
    }

    #[test]
    fn rejects_path_and_space_characters() {
        for bad in ["../etc", "a/b", "a b", "a\\b", "a\nb", ""] {
            assert!(ProjectName::new(bad).is_err(), "accepted {bad:?}");
        }
    }

    #[test]
    fn rejects_lowercase_and_leading_digit_secret_names() {
        assert!(SecretName::new("lowercase").is_err());
        assert!(SecretName::new("1ABC").is_err());
        assert!(SecretName::new("A-B").is_err());
    }

    #[test]
    fn rejects_overlong_names() {
        assert!(ProjectName::new("a".repeat(65)).is_err());
        assert!(ProjectName::new("a".repeat(64)).is_ok());
    }
}
