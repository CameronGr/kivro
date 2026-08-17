use std::collections::BTreeMap;

use age::secrecy::ExposeSecret;
use kivro_core::{EnvironmentName, ProjectName, SecretName, SecretString};
use kivro_crypto::{
    BUNDLE_FORMAT, Bundle, CIPHER_SCRYPT, OpenKey, SealKey, SealOptions, open, peek, seal,
};

fn sample() -> Bundle {
    let mut secrets = BTreeMap::new();
    secrets.insert(
        SecretName::new("DATABASE_URL").unwrap(),
        SecretString::new("postgres://a"),
    );
    secrets.insert(
        SecretName::new("AUTH0_CLIENT_SECRET").unwrap(),
        SecretString::new("s3cr3t"),
    );
    Bundle::new(
        ProjectName::new("infinity-launcher").unwrap(),
        EnvironmentName::new("dev").unwrap(),
        secrets,
    )
    .with_metadata(Some("2026-01-01T00:00:00Z".into()), Some("cameron".into()))
}

fn passphrase() -> SecretString {
    SecretString::new("correct horse battery staple")
}

#[test]
fn passphrase_round_trip() {
    let text = seal(
        &sample(),
        &SealKey::Passphrase(passphrase()),
        SealOptions::default(),
    )
    .unwrap();
    let opened = open(&text, &OpenKey::Passphrase(passphrase())).unwrap();

    assert_eq!(opened.project.as_str(), "infinity-launcher");
    assert_eq!(opened.environment.as_str(), "dev");
    assert_eq!(opened.created_by.as_deref(), Some("cameron"));
    assert_eq!(
        opened.secrets[&SecretName::new("DATABASE_URL").unwrap()].expose_secret(),
        "postgres://a"
    );
}

#[test]
fn ciphertext_never_contains_plaintext() {
    let text = seal(
        &sample(),
        &SealKey::Passphrase(passphrase()),
        SealOptions::default(),
    )
    .unwrap();
    assert!(!text.contains("postgres://a"));
    assert!(!text.contains("s3cr3t"));
    // Default options withhold names as well.
    assert!(!text.contains("AUTH0_CLIENT_SECRET"));
    assert!(text.contains(CIPHER_SCRYPT));
    assert!(text.contains("BEGIN AGE ENCRYPTED FILE"));
}

#[test]
fn name_hints_are_opt_in() {
    let options = SealOptions {
        hint_identity: true,
        hint_names: true,
    };
    let text = seal(&sample(), &SealKey::Passphrase(passphrase()), options).unwrap();
    assert!(text.contains("AUTH0_CLIENT_SECRET"));
    assert!(!text.contains("s3cr3t"));

    let hint = peek(&text).unwrap();
    assert_eq!(hint.project.as_deref(), Some("infinity-launcher"));
    assert!(hint.is_supported());
    assert!(hint.needs_passphrase());
    assert_eq!(hint.names.as_ref().unwrap().len(), 2);
}

#[test]
fn wrong_passphrase_is_rejected_clearly() {
    let text = seal(
        &sample(),
        &SealKey::Passphrase(passphrase()),
        SealOptions::default(),
    )
    .unwrap();
    let err = open(&text, &OpenKey::Passphrase(SecretString::new("wrong"))).unwrap_err();
    assert_eq!(err.kind(), "crypto_error");
    assert!(err.to_string().contains("wrong passphrase"));
}

#[test]
fn tampering_with_the_ciphertext_is_detected() {
    let text = seal(
        &sample(),
        &SealKey::Passphrase(passphrase()),
        SealOptions::default(),
    )
    .unwrap();
    let mut envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    let payload = envelope["payload"].as_str().unwrap().to_string();

    // Flip a character inside the armored body (not the header lines).
    let lines: Vec<&str> = payload.lines().collect();
    let target = lines.len() / 2;
    let mutated: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            if i == target && !l.is_empty() {
                let mut chars: Vec<char> = l.chars().collect();
                chars[0] = if chars[0] == 'A' { 'B' } else { 'A' };
                chars.into_iter().collect()
            } else {
                (*l).to_string()
            }
        })
        .collect();
    envelope["payload"] = serde_json::Value::String(mutated.join("\n"));

    let err = open(&envelope.to_string(), &OpenKey::Passphrase(passphrase())).unwrap_err();
    assert!(matches!(err.kind(), "crypto_error" | "bundle_format"));
}

#[test]
fn tampering_with_the_unauthenticated_hint_is_detected() {
    let options = SealOptions {
        hint_identity: true,
        hint_names: true,
    };
    let text = seal(&sample(), &SealKey::Passphrase(passphrase()), options).unwrap();

    let mut envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    envelope["hint"]["project"] = serde_json::Value::String("some-other-project".into());
    let err = open(&envelope.to_string(), &OpenKey::Passphrase(passphrase())).unwrap_err();
    assert_eq!(err.kind(), "bundle_mismatch");

    let mut envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    envelope["hint"]["names"] = serde_json::json!(["DATABASE_URL"]);
    let err = open(&envelope.to_string(), &OpenKey::Passphrase(passphrase())).unwrap_err();
    assert_eq!(err.kind(), "bundle_mismatch");
}

#[test]
fn unknown_ciphers_and_formats_are_refused_not_guessed() {
    let text = seal(
        &sample(),
        &SealKey::Passphrase(passphrase()),
        SealOptions::default(),
    )
    .unwrap();

    let mut envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    envelope["cipher"] = serde_json::Value::String("age-v9-pqc".into());
    let err = open(&envelope.to_string(), &OpenKey::Passphrase(passphrase())).unwrap_err();
    assert_eq!(err.kind(), "bundle_format");
    assert!(err.to_string().contains("unsupported cipher"));

    let mut envelope: serde_json::Value = serde_json::from_str(&text).unwrap();
    envelope["format"] = serde_json::json!(BUNDLE_FORMAT + 1);
    let err = open(&envelope.to_string(), &OpenKey::Passphrase(passphrase())).unwrap_err();
    assert!(err.to_string().contains("upgrade"));
}

#[test]
fn malformed_input_is_rejected() {
    for bad in [
        "",
        "{}",
        "not json",
        r#"{"magic":"nope","format":1,"cipher":"x","payload":""}"#,
    ] {
        assert!(open(bad, &OpenKey::Passphrase(passphrase())).is_err());
        assert!(peek(bad).is_err());
    }
}

#[test]
fn empty_passphrases_are_refused() {
    let err = seal(
        &sample(),
        &SealKey::Passphrase(SecretString::new("")),
        SealOptions::default(),
    )
    .unwrap_err();
    assert_eq!(err.kind(), "crypto_error");
}

#[test]
fn public_key_round_trip() {
    // Generated per run: the point is the format round-trip, and a hard-coded
    // private key in a repository is a bad habit even when it guards nothing.
    let identity = age::x25519::Identity::generate();
    let recipient = identity.to_public().to_string();
    let identity = identity.to_string();

    let text = seal(
        &sample(),
        &SealKey::Recipients(vec![recipient]),
        SealOptions::default(),
    )
    .unwrap();
    let opened = open(
        &text,
        &OpenKey::Identities(vec![SecretString::new(identity.expose_secret().to_owned())]),
    )
    .unwrap();
    assert_eq!(opened.secrets.len(), 2);

    // A passphrase must not open a recipient bundle.
    let err = open(&text, &OpenKey::Passphrase(passphrase())).unwrap_err();
    assert!(err.to_string().contains("age identity"));
}
