---
name: keygen-cli
description: |
  keygen-cli — manage keygen.sh from the command line: products, policies, licenses,
  machines, releases, artifacts, webhooks, users, and tokens. Works against keygen.sh
  Cloud, self-hosted CE, and self-hosted EE. Use this skill whenever the user mentions:
  keygen.sh, license management, software licensing, license key, license activation,
  activate machine, suspend license, revoke license, renew license, release artifact
  upload, webhook endpoint, entitlement, policy, product token, offline license verify.
  All write operations require explicit human approval — see references/permissions.md
  for the three-tier rule set.
version: 0.3.0
license: MIT
homepage: https://github.com/okooo5km/keygen-cli
author: okooo5km
metadata:
  binary: keygen
  alias: kg
  requires:
    bins: ["keygen"]
  install:
    - kind: brew
      formula: okooo5km/tap/keygen-cli
      bins: ["keygen", "kg"]
    - kind: cargo
      crate_git: https://github.com/okooo5km/keygen-cli
  references:
    installation: ./references/installation.md
    commands:     ./references/commands.md
    permissions:  ./references/permissions.md
    envelope:     ./references/ai-envelope.md
    tui:          ./references/tui.md
---

# keygen-cli

A Rust CLI for [keygen.sh](https://keygen.sh). One binary, three deployments
(Cloud / self-hosted CE / self-hosted EE), and a stable JSON envelope built for
agents. Every command emits the same `{ ok, data, meta?, error? }` shape under
`--json`, and exit codes are documented and stable.

## Setup

```bash
brew install okooo5km/tap/keygen-cli   # installs `keygen` and the `kg` alias
keygen --version
```

Other paths (pre-built tarballs, `cargo install`, source) live in
[`references/installation.md`](./references/installation.md).

## Authenticate

```bash
keygen login                # interactive: deployment → host → account → token
keygen whoami               # confirm identity + detected capabilities
```

Tokens land in the OS keyring (Keychain on macOS, Secret Service on Linux,
Credential Manager on Windows). For CI and ephemeral shells, set
`KEYGEN_TOKEN=...` instead — that path skips the keyring entirely. Switch
deployments with `--profile <name>` or `KEYGEN_PROFILE`.

## Core invocation pattern

Every resource exposes the same CRUD surface. Resource-specific actions
(validate, suspend, publish, ...) sit on the same subcommand tree.

```bash
keygen <resource> list   [--filter k=v] [--limit N] [--page N] [--sort field]
keygen <resource> get    <id>
keygen <resource> create [--from-file body.json | --set attrs.x=y --metadata k=v]
keygen <resource> update <id> [--from-file body.json | --set ...]
keygen <resource> delete <id> --yes
```

Two examples:

```bash
keygen license list --filter status=ACTIVE --json
keygen license create --json --policy pol_abc --user usr_def --set attrs.name="Acme"
```

`--json` (a `gh`-style shortcut for `--output json`) always emits the canonical
envelope. Exit codes: `0` ok, `1` user, `2` server, `3` network, `4` auth, `5`
capability. Full envelope schema in
[`references/ai-envelope.md`](./references/ai-envelope.md).

## Self-discovery

Two commands let an agent learn the full surface area without reading docs:

```bash
keygen schema --format json     # entire command tree (subcommands + flags)
keygen explain error <CODE>     # diagnose any keygen.sh error code
```

Use `schema` when you need to verify a flag exists or look up a subcommand.
Use `explain` whenever an envelope returns `error.code` — it returns the
canonical cause, fix, and a suggested next command.

## Permission tiers

Three tiers govern which operations need human approval. The full per-command
list is in [`references/permissions.md`](./references/permissions.md); apply
the rules below verbatim.

### Tier 1 — auto-run

Read-only inspection. Run freely, no confirmation required.

- `list`, `get` on every resource.
- `whoami`, `doctor`, `schema`, `explain`, `config get`, `profile list`.
- `*-log list` / `*-log get` (request-log, event-log).
- `webhook event list` / `webhook event get`.

### Tier 2 — dry-run, then confirm

Mutating but reversible (or low-blast-radius). On the **first** invocation,
add `--dry-run --json`, show the user the request envelope (method + URL +
body), and proceed only after the user agrees.

- `create` and `update` on every resource.
- `license validate`, `machine activate`, `machine ping`, `process spawn`,
  `process ping`.
- `license usage incr|decr|reset`.
- `license check-out`, `license check-in`, `machine check-out`.
- `artifact download`, `artifact upload` (when uploading a brand-new file).
- `webhook endpoint test`, `webhook event retry`.
- `release upgrade` (read-only computation, but billable on Cloud — confirm).

### Tier 3 — explicit approval, no shortcuts

Destructive or irreversible. Never pass `--yes` until the user has verbally
agreed to the *exact* command and target id. Restate the command and its
blast radius first.

- `delete` on every resource.
- `license suspend`, `license reinstate`, `license renew`, `license revoke`,
  `license transfer`.
- `release publish`, `release yank`, `artifact yank`.
- `user ban`, `user unban`, `user reset-password`, `user update-password`.
- `process kill`.
- `token regenerate`.
- Any mutation aimed at a profile whose name contains `prod` / `production`,
  or the unnamed default profile when the host is `api.keygen.sh`.

## Risk heuristics

Before running a Tier 2 or Tier 3 operation, raise the concern *first* if any
of these hold:

- The active profile looks like production (`prod`, `production`, or the
  default profile pointing at `api.keygen.sh`).
- The batch covers five or more resource ids.
- A write request lacks `--idempotency-key` and the user has not opted into
  retries being unsafe.
- The user asked for `delete` / `revoke` without a recent `get` of the target.
- The command is EE-only (`event-log`, `request-log`, `env`) but `keygen
  doctor` reports CE/Cloud — surface the capability mismatch instead of
  hitting the API.

## Common patterns

```bash
# Issue a perpetual license for a known policy + user.
keygen license create --json \
  --policy pol_abc --user usr_def \
  --set attrs.name="Acme — Pro 2026" --metadata seat=enterprise

# Activate a machine and then ping it.
keygen machine activate --json --license lic_xyz --fingerprint $(uname -n)
keygen machine ping --json <machine_id>

# Cut a release and upload its binary.
keygen release create --json --product prod_abc --set attrs.version=1.4.0
keygen artifact upload --json --release rel_xyz --file ./dist/app-1.4.0.dmg
```

Walk-throughs:
[`examples/create-license.md`](./examples/create-license.md),
[`examples/offline-verify.md`](./examples/offline-verify.md).

## Where to dig deeper

- [`references/installation.md`](./references/installation.md) — every install path.
- [`references/commands.md`](./references/commands.md) — full resource × action matrix.
- [`references/permissions.md`](./references/permissions.md) — authoritative tier list.
- [`references/ai-envelope.md`](./references/ai-envelope.md) — JSON shape + exit codes.
- [`references/tui.md`](./references/tui.md) — `keygen tui` keybindings and panels.
