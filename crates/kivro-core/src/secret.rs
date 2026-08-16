//! The SecretString type.
//!
//! # Security design
//!
//! Secret values are never represented as a plain `String` anywhere in this
//! workspace. SecretString exists to make the accidental paths for leaking a
//! secret (logging, Debug, {}, serde) hard to hit by accident:
//!
//! Debug renders a fixed redacted placeholder
//! std::fmt::Display is deliberately not implemented
//! serde::Serialize is deliberately not implemented
//! serde::Deserialize is implemented, because moving data from a less protected representation into a more protected one is the safe direction
//! The inner buffer is wrapped in zeroize::Zeroizing, so the heap allocation is overwritten when the value is dropped

use std::fmt;

use zeroize::Zeroizing;

/// secret value held in memory
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(Zeroizing<String>);

impl SecretString {
    /// Create a secret from value
    pub fn new(value: impl Into<String>) -> Self {
        Self(Zeroizing::new(value.into()))
    }

    /// Expose a secret, we can keep this name verbose so you can grep it easily
    pub fn expose_secret(&self) -> &str {
        self.0.as_str()
    }

    /// length of the secret in bytes
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// is secret empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// safe way to determine what a secret is
    pub fn describe(&self) -> String {
        format!("<{} bytes, redacted>", self.0.len())
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString(<redacted>)")
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl<'de> serde::Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(SecretString::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_is_redacted() {
        let s = SecretString::new("hunter2");
        let rendered = format!("{:?}", s);
        assert!(!rendered.contains("hunter2"));
        assert_eq!(rendered, "SecretString(<redacted>)");
    }

    #[test]
    fn debug_of_containing_structures_is_redacted() {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Holder {
            name: &'static str,
            value: SecretString,
        }
        let h = Holder {
            name: "AUTH0_CLIENT_SECRET",
            value: SecretString::new("hunter2"),
        };
        assert!(!format!("{:?}", h).contains("hunter2"));
    }

    #[test]
    fn describe_leaks_only_length() {
        assert_eq!(SecretString::new("abc").describe(), "<3 bytes, redacted>");
    }

    #[test]
    fn round_trips_through_expose() {
        assert_eq!(SecretString::new("abc").expose_secret(), "abc");
    }
}
