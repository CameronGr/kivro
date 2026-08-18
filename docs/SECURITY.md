# Security model

This document states what the tool actually protects against, and what it does
not. Where a guarantee is weaker than it might appear, that is said plainly
rather than glossed.

## What the tool does

- Stores secret values in the OS credential store (Credential Manager, Keychain,
  Secret Service) instead of plaintext files.
- Keeps values out of the repository: the manifest is declarations only.
- Injects values directly into a child process environment — no intermediate
  file, no shell argument.
- Encrypts values with age when they need to move between machines.
- Fails loudly rather than degrading: an unavailable keyring is an error, never
  a silent fallback to something weaker.

## Threat model

### In scope

| Threat                               | Mitigation                                                                                                                                                                      |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Secrets committed to git             | Values are never in a file the repository tracks; `doctor` flags a stray `.env` and missing `.gitignore` entries.                                                               |
| Secrets in shell history             | `set` prompts; a value cannot be passed as an argument. `--stdin` exists for CI.                                                                                                |
| Secrets in `ps` output / argv        | Values are only ever passed through the environment block, never argv.                                                                                                          |
| Secrets in logs, errors, crash dumps | `SecretString` has no `Display` and no `Serialize`; `Debug` is redacted. No error variant carries a value. Keyring errors that could carry credential bytes are mapped by hand. |
| Secrets in terminal scrollback       | Nothing prints a value except `get --show`, which warns when stdout is a terminal.                                                                                              |
| Secrets left on disk after sharing   | Bundles are encrypted at rest; `doctor` warns about bundles left in the project root.                                                                                           |
| A tampered bundle                    | age authenticates the ciphertext; the unauthenticated header is cross-checked against the authenticated payload and mismatches are refused.                                     |
| A bundle from the wrong project      | `accept` compares the authenticated payload's project against the local manifest.                                                                                               |
| Cross-project collision              | Names are validated so the `app:project:environment` namespace is injective.                                                                                                    |
| A hostile bundle burning CPU         | The accepted scrypt work factor is bounded.                                                                                                                                     |

### Out of scope

| Threat                                        | Why                                                                                                                                                                            |
| --------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| A compromised developer machine               | Anything that can run code as you can ask the credential store for the same secrets this tool can. That is inherent to any local secret manager.                               |
| Malicious child processes                     | `secrets run` hands the secrets to the command you named. It cannot police what that command does with them. Supply-chain risk in your dependencies is unchanged by this tool. |
| Another process reading `/proc/<pid>/environ` | On Linux this is restricted to the same user (and root). Environment variables are the standard interface; that is the trade-off they carry.                                   |
| Memory forensics, swap, core dumps            | Best-effort only. See below.                                                                                                                                                   |
| Malicious `.kivro.toml`                       | The manifest is code-reviewed content in your repository. It cannot carry values, but a hostile edit could add declarations. Review it like any other file.                    |
| Insider access                                | There is no per-secret access control in 0.1. Anyone you hand a bundle to has the values.                                                                                      |

## Memory handling — honest limits

`SecretString` wraps a `zeroize::Zeroizing<String>`, so the heap buffer is
overwritten when the value is dropped, and the serialised bundle payload is
zeroized after encryption.

**This is not a guarantee that a secret exists in exactly one place in memory.**
It cannot be, in a garbage-free language with a moving allocator's freedom to
copy:

- `String` growth reallocates and leaves the old buffer's contents behind.
- Moves and clones copy bytes we do not track.
- The OS may page memory to swap or write it to a core dump.
- Between `expose_secret()` and the kernel receiving the child's environment
  block, the value exists as an ordinary allocation.
- The `keyring` and `age` crates hold their own copies while working.

Zeroization here reduces the window and the number of stale copies. Treat it as
defence in depth, not as a boundary you can rely on.

## Cryptography

No cryptographic primitive or protocol is implemented in this project. Bundles
use the `age` crate: ChaCha20-Poly1305 over the STREAM construction, HMAC-SHA-256
over the header, scrypt for passphrases, X25519 + HKDF for public keys. See
[BUNDLE.md](BUNDLE.md) for the format and its versioning strategy.

Passphrases are never used as keys directly — age's scrypt recipient handles
stretching, with the work factor carried in the file and bounded on read.

## The insecure file store

`KIVRO_STORE=file` selects a plaintext JSON store. It exists because CI
containers have no D-Bus session, Keychain or Credential Manager, and testing
the real binary end to end is worth more than testing a mock.

Safeguards:

- it is never a default and never a fallback — an unavailable keyring is an
  error, and an unrecognised `KIVRO_STORE` value is rejected rather than
  guessed;
- the CLI prints a warning on **every** command that uses it;
- `doctor` reports it as a finding;
- files are created `0600` on Unix.

It provides no confidentiality against anything but other users on the same
machine. Do not use it for real credentials.

## `KIVRO_PASSPHRASE`

Supplies a bundle passphrase without a terminal, for CI. A passphrase in the
environment is visible to child processes and anything that dumps the
environment. Prefer `--recipient` with age public keys for automation, and the
interactive prompt for humans.

## `kivro export`

Writing a `.env` re-creates exactly the problem this tool exists to solve. It is
therefore explicit, never implicit: it requires confirmation (or `--yes`), warns
about plaintext on disk, refuses to overwrite without `--force`, and creates the
file `0600`. Use it only for tools that genuinely cannot read anything else, and
delete it afterwards.

## Reporting

Report suspected vulnerabilities privately to the maintainers rather than in a
public issue.
