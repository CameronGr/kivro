# Changelog

## 0.1.0

First release.

- OS credential store backends: Windows Credential Manager, macOS Keychain,
  Linux Secret Service, behind a `SecretStore` port.
- `.kivro.toml` manifest with versioned, forward-compatible parsing;
  discovery by walking up from the working directory.
- Deterministic `app:project:environment:NAME` storage namespace.
- `kivro` CLI: `init`, `set`, `get`, `list`, `remove`, `status`, `doctor`,
  `run`, `import`, `export`, `sync`, `share`, `accept`.
- age-based encrypted bundles for developer-to-developer sharing, passphrase or
  X25519 recipients.
- `SyncSource` abstraction with a file-based bundle source.
- Library API (`kivro` crate) that the CLI is built on.
- 97 tests, none requiring a real credential store.

### Compatibility

Manifest `format = 1` and bundle `format = 1` will be readable by future
releases. See `docs/MANIFEST.md` and `docs/BUNDLE.md`.
