# Encrypted bundle specification

A bundle moves a project's secrets between developers. It is a single text file,
conventionally named `<project>.<environment>.kivro`, encrypted with
[age](https://age-encryption.org/v1).

## File format

```json
{
  "magic": "infinity-secrets-bundle",
  "format": 1,
  "cipher": "age-v1-scrypt",
  "hint": {
    "project": "infinity-launcher",
    "environment": "dev",
    "created_at": "2026-01-14T09:31:00Z",
    "created_by": "cameron"
  },
  "payload": "-----BEGIN AGE ENCRYPTED FILE-----\n…\n-----END AGE ENCRYPTED FILE-----\n"
}
```

| Field     | Authenticated | Meaning                                                       |
| --------- | ------------- | ------------------------------------------------------------- |
| `magic`   | no            | Fixed string identifying the file type.                       |
| `format`  | no            | Envelope version. Above the supported version → refuse.       |
| `cipher`  | no            | `age-v1-scrypt` (passphrase) or `age-v1-x25519` (recipients). |
| `hint`    | **no**        | Advisory routing metadata. See below.                         |
| `payload` | yes           | ASCII-armored age file.                                       |

## The payload

Inside the age ciphertext is a JSON document:

```json
{
  "format": 1,
  "project": "infinity-launcher",
  "environment": "dev",
  "created_at": "2026-01-14T09:31:00Z",
  "created_by": "cameron",
  "secrets": { "DATABASE_URL": "…", "AUTH0_CLIENT_SECRET": "…" }
}
```

This is the **authoritative** copy of every field. age authenticates the whole
ciphertext (ChaCha20-Poly1305 per chunk, plus an HMAC-SHA-256 over the header),
so the payload is tamper-evident.

## Why metadata is duplicated

age has no additional-authenticated-data input, so there is no way to bind an
outer plaintext header to the ciphertext. Rather than present an unauthenticated
header as if it were trustworthy, the format states the split explicitly:

- the `hint` exists so tooling can say _"this looks like a bundle for
  infinity-launcher/dev"_ before a passphrase is available;
- on decryption, every hint field is compared against the authenticated payload,
  and any disagreement is a hard `bundle_mismatch` error.

`kivro accept` additionally checks the payload's project against the local
manifest, so a bundle for another project is refused even if its filename and
hint both claim otherwise.

Secret _names_ are withheld from the hint by default — a list of variable names
discloses which vendors and services a project depends on. `--hint-names`
includes them for tooling that needs to plan before decrypting.

## Cryptography

| Concern                    | Provided by                                         |
| -------------------------- | --------------------------------------------------- |
| Confidentiality            | age: ChaCha20-Poly1305 over the STREAM construction |
| Integrity / authentication | age: per-chunk AEAD tags + header HMAC-SHA-256      |
| Password-based KDF         | age scrypt recipient (work factor in the header)    |
| Public-key mode            | age X25519 + HKDF-SHA-256                           |
| Versioning                 | `format` (envelope) and `cipher` (scheme)           |

No cryptography is implemented in this project. age was chosen over a bespoke
format because it is specified, widely reviewed, and interoperable — a bundle's
payload can be decrypted with the standard `age` CLI, which matters for a format
that will hold production credentials for years.

Decryption bounds the accepted scrypt work factor (2^20), so a hostile bundle
cannot turn `kivro accept` into hours of KDF work.

## Algorithm agility

`cipher` names the scheme; an unrecognised value is refused with a message
naming what the build supports. Adding a scheme (a future age version, a
post-quantum recipient type) means adding a `cipher` value — old builds refuse
cleanly rather than misreading. `format` versions the envelope itself, for
changes to the JSON structure.

Migration path for a hypothetical `age-v2`: new CLIs write `age-v1-*` until a
release date, then switch; `kivro accept` reads both indefinitely. Bundles are
short-lived transfer artefacts, not storage, which keeps this cheap.

## Handling

Bundles are encrypted, but they are still a copy of your credentials in a file:

- send the file and the passphrase over **different** channels;
- prefer several random words to a short passphrase — scrypt buys time, not
  miracles, and a bundle in a chat log is available to an attacker forever;
- prefer `--recipient` (age public keys) for anything automated;
- delete the file after accepting it. `kivro doctor` warns about bundles left
  in a project root.
