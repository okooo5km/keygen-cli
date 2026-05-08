# keygen-cli

> AI-friendly CLI for [keygen.sh](https://keygen.sh) — manage products, policies, licenses, machines, and releases from the terminal or from an LLM agent.

`keygen-cli` ships a single binary (`keygen`, plus a `kg` shortcut) that talks to:

- **keygen.sh Official Cloud** (`https://api.keygen.sh`)
- **Self-hosted Community Edition** (Singleplayer / Multiplayer)
- **Self-hosted Enterprise Edition** (with environments, event/request logs, SSO, OCI registry)

---

## Status

🚧 **Early development.** The command tree and project scaffolding are in place. Resource implementations land iteratively per the plan in `doc/plan.md`.

---

## Highlights

- **Resource + action surface.** `keygen <resource> <action>` mirrors the keygen.sh API one-to-one, which makes LLM tool-calling trivial.
- **Stable JSON envelope** for AI mode: `{ ok, data, meta?, error? }` with documented exit codes (0–5).
- **Auto mode-switching.** Pretty colored tables on a TTY, deterministic JSON on pipes / `CI=true` / `--ai`.
- **OS keyring credential storage** (Keychain / Credential Manager / Secret Service) with a chmod-600 fallback under `$XDG_DATA_HOME`.
- **TUI dashboard.** `keygen tui` opens a ratatui-powered cockpit for browsing licenses, machines, releases, and live webhook events.
- **Capability detection.** The CLI probes each deployment and refuses EE-only commands on CE with a helpful upgrade hint.

---

## Install

### Homebrew (macOS / Linux)

```bash
brew tap okooo5km/tap
brew install keygen-cli
```

The formula installs both the `keygen` binary and the `kg` short alias.

### From source

```bash
cargo install --locked --path .
```

### Pre-built binaries

Each release publishes signed tarballs (`.sha256` alongside) for:

- macOS Universal (arm64 + x86_64)
- Linux x86_64
- Linux arm64
- Windows x86_64

See the [Releases](https://github.com/okooo5km/keygen-cli/releases) page.

---

## Quick start

```bash
keygen login                     # interactive: pick deployment → host → account → token
keygen whoami                    # show identity + detected capabilities
keygen license list              # pretty table on a terminal
keygen license list --output json   # explicit JSON
keygen license create --policy <pid> --user <uid>
keygen license validate <id>
keygen tui                       # full-screen dashboard
```

### AI / agent usage

```bash
keygen --ai license create --policy <pid> --user <uid> | jq
keygen --ai schema               # full command schema for tool-calling
```

Override anything with env vars or flags:

```bash
KEYGEN_HOST=https://licensing.example.com \
KEYGEN_ACCOUNT=acme \
KEYGEN_TOKEN=…  keygen license list
```

---

## Configuration

`keygen-cli` follows the **XDG Base Directory** spec.

| Path | Purpose |
|---|---|
| `$XDG_CONFIG_HOME/keygen/config.toml` | Profiles (host / account / output defaults) |
| OS keyring entry `sh.keygen.cli:<profile>` | Tokens (preferred) |
| `$XDG_DATA_HOME/keygen/credentials.toml` | Token fallback (chmod 600) |
| `$XDG_CACHE_HOME/keygen/capabilities.json` | Capability probe cache (1d TTL) |
| `$XDG_STATE_HOME/keygen/history.log` | TUI command history |

Override resolution order:
`flag > env var > active profile > default profile`.

---

## Development

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

The full design + roadmap lives in [`doc/plan.md`](doc/plan.md).

---

## License

MIT © okooo5km(十里) — see [LICENSE](LICENSE).
