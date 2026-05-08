# Example — offline license verification

`keygen license verify` performs offline signature verification: it splits
the license key into `<encoded>.<sig>`, recovers the bytes that were
signed (`key/<encoded>` per keygen.sh's convention), and runs the matching
verifier. No network call.

This is Tier 1: read-only, no API contact.

## Prerequisites

You need three things from the customer:

1. The license **key** (the `<encoded>.<sig>` blob, sometimes wrapped in a
   `.lic` file as plain text).
2. The product's **verifying key** — either the hex Ed25519 public key from
   the keygen.sh dashboard, or a PEM-encoded RSA key.
3. The signing **scheme**, which keygen.sh embeds in the policy:
   `ED25519_SIGN` (default) or `RSA_2048_PKCS1_SIGN_V2`.

## 1. Verify a key directly

```bash
keygen license verify \
  --key "abc.def...sig...==" \
  --public-key 8e2f...                    # 64-hex-char Ed25519 key
```

Output (table):

```text
✓ signature valid
key:        abc.def…
scheme:     ED25519_SIGN
algorithm:  Ed25519
```

Or under `--json`:

```json
{ "ok": true, "data": { "valid": true, "scheme": "ED25519_SIGN", "algorithm": "Ed25519" } }
```

## 2. Verify with a key file

For RSA, point at a PEM:

```bash
keygen license verify \
  --key "abc.def...sig...==" \
  --public-key-file ./product-public.pem \
  --scheme RSA_2048_PKCS1_SIGN_V2
```

For raw / hex Ed25519 the same flag works (raw 32-byte file, hex string in
a file, or PEM are all accepted).

## 3. Verify a checked-out `.lic` blob

`license check-out` produces a `.lic` file whose contents are exactly the
signed key. Verify it the same way:

```bash
keygen license verify \
  --key "$(cat alice.lic)" \
  --public-key-file ./product-public.pem
```

## 4. Failure modes

```json
{ "ok": false,
  "error": { "kind": "user", "code": "LICENSE_KEY_MALFORMED",
             "detail": "key is not in <encoded>.<sig> form" } }
```

Common causes:

| Symptom | Cause |
|---|---|
| `LICENSE_KEY_MALFORMED` | Truncated key, missing `.sig`, or wrong file. |
| `SIGNATURE_INVALID`     | Wrong public key, wrong scheme, or tampered key. |
| `INVALID_PUBLIC_KEY`    | Hex string isn't 64 chars, PEM doesn't parse, etc. |

When verification fails, do **not** try to "fix" the key — fetch a fresh
copy and the matching verifying key from the dashboard.

## 5. Embedding in your app

For most use cases, ship the verifying key compiled into the binary and
call the same verification code keygen-cli uses. The scheme selector and
the `key/<encoded>` byte convention are the only product-specific bits.
