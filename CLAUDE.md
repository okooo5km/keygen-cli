# CLAUDE.md — `keygen-cli`

Working notes for AI agents (Claude / FRIDAY) collaborating on this repo.

## What this is

A Rust CLI for [keygen.sh](https://keygen.sh). The single binary is `keygen` (with a `kg` shortcut symlink installed by Homebrew). It supports keygen.sh Official, self-hosted CE, and self-hosted EE.

The full design + step-by-step roadmap lives in `doc/plan.md`. **Read it before making non-trivial changes.** It is the source of truth for: resource+action matrix, AI/human mode rules, XDG paths, CI/CD targets, and the 0→15 implementation order.

## Repo layout

```
src/
├── main.rs / lib.rs            # entry + top-level dispatch
├── cli/                        # clap derive — Cli, Command, GlobalArgs, ResourceCommand
├── api/                        # reqwest + JSON:API plumbing
├── auth/                       # login flow + keyring storage
├── capability/                 # CE/EE/Official feature probe + `doctor`
├── config/                     # profile model + on-disk config.toml
├── output/                     # mode resolver + json/yaml/ndjson/table renderers
├── render/                     # status colors / time / progress
├── resources/                  # one module per keygen.sh resource (Cmd + dispatch)
├── schema/                     # `keygen schema` — emits CLI tree as JSON
└── tui/                        # ratatui dashboard (`keygen tui`)
```

## Conventions

- **Binary name is `keygen`**, not `keygen-cli`. The crate is `keygen-cli`; only the package metadata / Homebrew formula uses the long form.
- **Do not break the AI envelope.** `--output json` must always emit `{ "ok": <bool>, "data": ... }` or `{ "ok": false, "error": {...} }`. Adding fields is fine; renaming or removing them is not.
- **Stable exit codes** (`src/exit.rs`): 0 ok / 1 user / 2 server / 3 network / 4 auth / 5 capability. Don't repurpose them.
- **CRUD shape** (`src/resources/common.rs`): every resource exposes `list / get / create / update / delete` with the same flag surface. Resource-specific actions go on the same `Cmd` enum.
- **Capability gates.** EE-only / Official-only commands must check `crate::capability` before talking to the API and return `Error::Capability` with a hint, never a raw HTTP error.
- **No `unsafe`.** Forbidden at the crate level (`#![forbid(unsafe_code)]` via `Cargo.toml`).
- **Author tag.** New files attribute to `okooo5km(十里)` when a top-of-file author line is needed.

## When you implement a resource

1. Add the `Cmd` variants in `src/resources/<name>.rs` (extend, don't replace existing CRUD).
2. Implement `dispatch` to call the shared API client (`crate::api::Client`).
3. Add a `render` module if the resource has a custom status / column layout.
4. If the resource is EE-only, gate it in `dispatch` via `Capability` checks.
5. Make sure both **AI** (json) and **human** (table) outputs are exercised.
6. Add wiremock-based integration tests under `tests/integration/<name>.rs`.
7. Update `doc/plan.md` only if the API contract changed; otherwise keep notes in this file.

## CI / release

- `.github/workflows/ci.yml` runs test + clippy + fmt on PRs.
- `.github/workflows/release.yml` triggers on `v*` tags: builds macOS universal + Linux x64/arm64 + Windows, creates a GitHub Release, and pushes a Homebrew formula to `okooo5km/homebrew-tap` (skipped for pre-release tags containing `-`).
- Bump version in `Cargo.toml`, commit, then `git tag vX.Y.Z && git push --tags`.

## Quick checks

```bash
cargo check
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
./target/debug/keygen --help
./target/debug/keygen license --help
```

## Pointers

- API reference: <https://keygen.sh/docs/api/>
- Self-hosting: <https://keygen.sh/docs/self-hosting/>
- Internal plan: `doc/plan.md`
