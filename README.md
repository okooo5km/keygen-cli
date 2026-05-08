# keygen-cli

> AI-friendly CLI for [keygen.sh](https://keygen.sh) — manage products, policies, licenses, machines, and releases from the terminal or from an LLM agent.

`keygen-cli` ships a single binary (`keygen`, plus a `kg` shortcut) that talks to:

- **keygen.sh Official Cloud** (`https://api.keygen.sh`)
- **Self-hosted Community Edition** (Singleplayer / Multiplayer)
- **Self-hosted Enterprise Edition** (with environments, event/request logs, SSO, OCI registry)

---

## Status

🚀 **Working v0.3.** Every resource in keygen.sh's API has a wired-up CRUD + action surface. The ratatui dashboard ships master/detail panes, a card view (`--layout cards`), an action panel with dry-run + tier-aware confirm overlays, a `:` command palette, and a live webhook-events tail. Tables align under CJK / emoji, `keygen explain error <code>` ships a 30+ entry diagnosis catalog, and `keygen license verify` does offline ED25519 / RSA signature checks. The bundled Claude Code skill at `skills/keygen/` teaches agents the full surface area in one install. See `doc/plan.md` for the full design.

---

## Highlights

- **Resource + action surface.** `keygen <resource> <action>` mirrors the keygen.sh API one-to-one, which makes LLM tool-calling trivial.
- **Stable JSON envelope** for AI mode: `{ ok, data, meta?, error? }` with documented exit codes (0–5).
- **Auto mode-switching.** Pretty colored tables on a TTY, deterministic JSON on pipes / `CI=true` / `--ai`.
- **OS keyring credential storage** (Keychain / Credential Manager / Secret Service) with a chmod-600 fallback under `$XDG_DATA_HOME`.
- **TUI dashboard.** `keygen tui` opens a ratatui-powered cockpit: browse resources in tabs, press `a` for the action panel (validate / suspend / revoke / ...), `:` for a vim-style command palette, watch webhook events tail in real time, and any Tier 2/3 op gets a destructive-banner confirm overlay before it goes live.
- **Capability detection.** The CLI probes each deployment and refuses EE-only commands on CE with a helpful upgrade hint.

---

## Install

### Homebrew (macOS / Linux)

```bash
brew install okooo5km/tap/keygen-cli
```

To upgrade an already-installed copy:

```bash
brew update
brew upgrade okooo5km/tap/keygen-cli
```

The formula installs both the `keygen` binary and the `kg` short alias.

> **Note:** the tap is updated by CI a few minutes after a new tag is pushed,
> so right after a release `brew install` may still serve the previous
> version until you run `brew update`.

### Pre-built binaries

Each release ships signed tarballs (with a `.sha256` next to each archive)
under the [Releases](https://github.com/okooo5km/keygen-cli/releases/latest)
page. Pick the file matching your platform.

**macOS (universal — arm64 + x86_64):**

```bash
VERSION=0.2.0
curl -L -O https://github.com/okooo5km/keygen-cli/releases/download/v${VERSION}/keygen-cli_${VERSION}_darwin_universal.tar.gz
curl -L -O https://github.com/okooo5km/keygen-cli/releases/download/v${VERSION}/keygen-cli_${VERSION}_darwin_universal.tar.gz.sha256
shasum -a 256 -c keygen-cli_${VERSION}_darwin_universal.tar.gz.sha256
tar xzf keygen-cli_${VERSION}_darwin_universal.tar.gz
sudo mv keygen /usr/local/bin/
sudo ln -sf /usr/local/bin/keygen /usr/local/bin/kg
# macOS Gatekeeper note: `xattr -d com.apple.quarantine /usr/local/bin/keygen`
# if you downloaded via the browser.
```

**Linux x86_64:**

```bash
VERSION=0.2.0
curl -L -O https://github.com/okooo5km/keygen-cli/releases/download/v${VERSION}/keygen-cli_${VERSION}_linux_x86_64.tar.gz
sha256sum -c keygen-cli_${VERSION}_linux_x86_64.tar.gz.sha256
tar xzf keygen-cli_${VERSION}_linux_x86_64.tar.gz
sudo install -m 0755 keygen /usr/local/bin/
sudo ln -sf /usr/local/bin/keygen /usr/local/bin/kg
```

**Linux arm64 (Raspberry Pi 4/5, Ampere, AWS Graviton):**

```bash
VERSION=0.2.0
curl -L -O https://github.com/okooo5km/keygen-cli/releases/download/v${VERSION}/keygen-cli_${VERSION}_linux_arm64.tar.gz
sha256sum -c keygen-cli_${VERSION}_linux_arm64.tar.gz.sha256
tar xzf keygen-cli_${VERSION}_linux_arm64.tar.gz
sudo install -m 0755 keygen /usr/local/bin/
sudo ln -sf /usr/local/bin/keygen /usr/local/bin/kg
```

**Windows x86_64 (PowerShell):**

```powershell
$Version = "0.2.0"
Invoke-WebRequest -Uri "https://github.com/okooo5km/keygen-cli/releases/download/v$Version/keygen-cli_${Version}_windows_x86_64.zip" -OutFile keygen.zip
Expand-Archive keygen.zip -DestinationPath "$Env:USERPROFILE\bin"
# add %USERPROFILE%\bin to PATH if it isn't already
```

### From source

If you have a Rust toolchain (`>= 1.81`):

```bash
cargo install --locked --git https://github.com/okooo5km/keygen-cli --tag v0.2.0
```

Or, after cloning the repo:

```bash
git clone https://github.com/okooo5km/keygen-cli
cd keygen-cli
cargo install --locked --path .
```

### Shell completion

```bash
keygen completion zsh  > ~/.zfunc/_keygen        # zsh
keygen completion bash > ~/.local/share/bash-completion/completions/keygen
keygen completion fish > ~/.config/fish/completions/keygen.fish
```

### Claude Code skill

If you use [Claude Code](https://docs.claude.com/claude-code) — or any agent
that loads `~/.claude/skills/` — install the bundled skill so the agent
picks up keygen-cli's command surface, JSON envelope, and permission tiers
automatically:

```bash
git clone https://github.com/okooo5km/keygen-cli   # if you haven't already
./keygen-cli/skills/keygen/install.sh
```

The script symlinks `skills/keygen/` into `~/.claude/skills/keygen` (or
`$CLAUDE_SKILLS_DIR/keygen` if set). The skill file (`SKILL.md`) tells the
agent which commands are read-only (auto-run), which require a `--dry-run`
preview before going live, and which need explicit `--yes`. The full
per-command list lives in `skills/keygen/references/permissions.md`.

---

## Quick start

```bash
keygen login                          # interactive: pick deployment → host → token
keygen whoami                         # show identity + detected capabilities
keygen license list                   # colored table on a TTY, plain table on a pipe
keygen license get <id>
keygen license validate <id>
keygen tui                            # full-screen dashboard
```

`keygen` defaults to a human-friendly table — colored on a TTY, plain ASCII on
a pipe. ANSI is suppressed automatically when stdout isn't a terminal or you
set `NO_COLOR=` / pass `--no-color`.

### AI / agent usage

Pass `--json` (mirrors `gh`) for the canonical envelope. Other formats live
behind `--output yaml|tsv|ndjson`.

```bash
keygen license list --json | jq '.data[].id'
keygen license create --json --from-file new-license.json
keygen schema --format json | jq '.data.command.subcommands'   # tool-call schema
```

Two JSON shapes:

```json
{ "ok": true, "data": { ... }, "meta": { "page": 1, "limit": 50 } }
{ "ok": false, "error": { "code": "LICENSE_SUSPENDED", "title": "...",
                          "http_status": 422, "hint": "run `keygen license reinstate <id>`",
                          "request_id": "01HZ..." } }
```

Exit codes are stable: `0` ok, `1` user error, `2` server error, `3` network, `4` auth, `5` capability not supported.

Override anything with env vars or flags:

```bash
KEYGEN_HOST=https://licensing.example.com \
KEYGEN_ACCOUNT=acme \
KEYGEN_TOKEN=…  keygen license list
```

---

## Configuration

`keygen-cli` follows the **XDG Base Directory** spec on every Unix (Linux,
macOS) — the `directories` crate's macOS-native `~/Library/Application
Support/...` is *not* used. Windows still goes through `directories`.

| Unix default | XDG override | Purpose |
|---|---|---|
| `~/.config/keygen/config.toml` | `$XDG_CONFIG_HOME/keygen/config.toml` | Profiles (host / account / output defaults) |
| OS keyring entry `sh.keygen.cli:<profile>` | — | Tokens |
| `~/.cache/keygen/capabilities.json` | `$XDG_CACHE_HOME/keygen/...` | Capability probe cache (1d TTL) |
| `~/.local/share/keygen/` | `$XDG_DATA_HOME/keygen/` | Future: credential fallback / TUI state |

Override resolution order:
`flag > env var > active profile > default profile`.

---

## Resource catalogue

| Resource | Surface |
|---|---|
| `license` | CRUD + validate / validate-key / suspend / reinstate / renew / revoke / check-out / check-in / usage incr/decr/reset / tokens / transfer |
| `machine` | activate / deactivate / list / get / update / ping / reset / check-out |
| `policy` | CRUD + entitlements attach/detach/list |
| `product` | CRUD + tokens |
| `release` | CRUD + publish / yank / upgrade / constraints / packages |
| `artifact` | list / get / upload (resumable) / download (with progress) / yank |
| `package`, `entitlement`, `component`, `token`, `group`, `user`, `process` | CRUD + resource-specific actions (token regen, user ban/groups, process spawn/kill/ping, group users/licenses, ...) |
| `webhook` | endpoints CRUD + endpoint test, events list/get + retry |
| `request-log`, `event-log` | list/get (EE-only, capability gated) |
| Top-level | `login` / `logout` / `whoami` / `doctor` / `config` / `profile` / `env` / `schema` / `completion` / `tui` |

## Development

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
./target/debug/keygen schema --format json | jq
```

The full design + roadmap lives in [`doc/plan.md`](doc/plan.md).

---

## License

MIT © okooo5km(十里) — see [LICENSE](LICENSE).
