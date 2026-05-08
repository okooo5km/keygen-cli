# Command reference

Every resource exposes the same five CRUD verbs (where the API supports them)
plus its own action verbs. Flags below the matrix apply globally to every
command. Resource-specific examples follow at the end of each section — copy
them when you need to look up a concrete invocation.

## Global flags

| Flag | Purpose |
|---|---|
| `--profile <name>` / `KEYGEN_PROFILE` | Pick a profile from `config.toml`. |
| `--host <url>` / `KEYGEN_HOST` | Override the API host. |
| `--account <id>` / `KEYGEN_ACCOUNT` | Override the account id (Cloud / multiplayer EE). |
| `--token <…>` / `KEYGEN_TOKEN` | Inject a token, skip the keyring. |
| `--env <id>` / `KEYGEN_ENV` | EE: pick the active environment. |
| `--output table\|json\|yaml\|tsv\|ndjson` | Output format. Default `table`. |
| `--json` | `gh`-style shortcut for `--output json`. |
| `--layout table\|cards` / `--cards` | Human layout when not JSON. |
| `--no-color` / `NO_COLOR=` | Disable ANSI colors. |
| `--quiet` / `-q` | Print only the key result (id / key). |
| `-v` / `-vv` / `-vvv` | Increase log level. |
| `--dry-run` | Print the request that would be sent and exit. |
| `--idempotency-key <…>` | Idempotency key for write operations. |
| `--timeout <secs>` | Request timeout (default 30). |
| `--retry <n>` | Retries for idempotent requests (default 2). |

## Top-level commands

| Command | Purpose |
|---|---|
| `keygen login` / `keygen logout` | Manage credentials in the OS keyring. |
| `keygen whoami` | Identity + detected capabilities. |
| `keygen doctor` | Probe host, token, and capability matrix. |
| `keygen config <get\|set\|list\|...>` | Inspect / mutate `config.toml`. |
| `keygen profile <list\|use\|...>` | Switch named host + account combos. |
| `keygen env <list\|use\|...>` | EE: switch the active environment. |
| `keygen explain error <CODE>` | Diagnose an API error code. |
| `keygen schema --format json` | Full command tree as JSON (agent self-discovery). |
| `keygen completion <shell>` | Generate completion scripts. |
| `keygen tui` | Launch the full-screen dashboard. |

## CRUD shape

```bash
keygen <resource> list      [--filter k=v]... [--limit N] [--page N] [--sort field] [--include rel]
keygen <resource> get       <id> [--include rel]
keygen <resource> create    [--from-file body.json|-] [--set path=val]... [--metadata k=v]...
keygen <resource> update    <id> [--from-file body.json|-] [--set ...]
keygen <resource> delete    <id> --yes
```

`--set path=value` is JSONPath-flavored: `--set attrs.maxMachines=5`,
`--set relationships.policy.data.id=pol_abc`. `--from-file -` reads a
JSON:API payload from stdin.

## Resource matrix

Every line below is a real subcommand. Items marked **EE** require self-hosted
EE (`event-log`, `request-log`, `env`); items marked **Cloud/EE** require a
multiplayer account.

### `token` — API tokens

CRUD: `list / get / create / update / delete`. Plus:

| Action | Notes |
|---|---|
| `keygen token regenerate <id>` | Rotates the secret. **Tier 3 — irreversible.** |

### `product` — product definitions

CRUD: `list / get / create / update / delete`. Plus:

| Action | Notes |
|---|---|
| `keygen product tokens <id>` | List tokens scoped to the product. |

### `policy` — license policies

CRUD: `list / get / create / update / delete`. Plus:

| Action | Notes |
|---|---|
| `keygen policy entitlements attach <id> <entitlement>` | Attach an entitlement. |
| `keygen policy entitlements detach <id> <entitlement>` | Detach an entitlement. |
| `keygen policy entitlements list   <id>` | Currently-attached entitlements. |

### `license` — issued licenses

CRUD: `list / get / create / update / delete`. Plus:

| Action | Notes |
|---|---|
| `keygen license validate <id> [--fingerprint <fp>]` | Server-side validation. |
| `keygen license validate-key <key>` | Validate by key (no auth). |
| `keygen license verify --key <…> --public-key <hex\|PEM>` | **Offline** signature verification. |
| `keygen license suspend <id>` | **Tier 3.** Reversible via `reinstate`. |
| `keygen license reinstate <id>` | Lift a suspension. **Tier 3.** |
| `keygen license renew <id>` | Extend the expiry. **Tier 3.** |
| `keygen license revoke <id>` | **Tier 3 — irreversible.** |
| `keygen license check-out <id>` | Mint a signed `.lic` blob for offline use. |
| `keygen license check-in <id>` | Cancel an outstanding check-out. |
| `keygen license usage incr\|decr <id> [--by N]` | Adjust the usage counter. |
| `keygen license usage reset <id>` | Reset usage to zero. |
| `keygen license tokens <id>` | Tokens scoped to the license. |
| `keygen license transfer <id> --user <…> \| --policy <…> \| --group <…>` | **Tier 3.** |

`license create` accepts shortcut flags (`--policy`, `--user`, `--group`)
that fold into `relationships.*`.

### `entitlement` — feature flags attached to policies

CRUD: `list / get / create / update / delete`. No extra actions.

### `user` — account users

CRUD: `list / get / create / update / delete`. Plus:

| Action | Notes |
|---|---|
| `keygen user ban   <id>` | Block sign-in. **Tier 3.** |
| `keygen user unban <id>` | Lift a ban. **Tier 3.** |
| `keygen user reset-password   <id>` | Send a reset email. **Tier 3.** |
| `keygen user update-password  <id> --password <…>` | Admin reset. **Tier 3.** |
| `keygen user groups attach\|detach\|list <id> [<group>]` | Group membership. |
| `keygen user tokens <id>` | Tokens scoped to the user. |

### `group` — user groups (Cloud/EE)

CRUD: `list / get / create / update / delete`. Plus:

| Action | Notes |
|---|---|
| `keygen group users attach\|detach\|list <id> [<user>]` | Membership. |
| `keygen group licenses <id>` | Licenses owned by the group. |

### `machine` — license activations

CRUD: `list / get / create / update / delete`. Plus:

| Action | Notes |
|---|---|
| `keygen machine activate --license <…> --fingerprint <…>` | Create a machine. |
| `keygen machine deactivate <id>` | Same as `delete`. |
| `keygen machine ping <id>` | Send a heartbeat. |
| `keygen machine reset <id>` | Reset heartbeat counter. |
| `keygen machine check-out <id>` | Mint a signed activation blob. |

### `component` — hardware fingerprints

CRUD only.

### `process` — process heartbeats

CRUD: `list / get / create / update / delete`. Plus:

| Action | Notes |
|---|---|
| `keygen process spawn --machine <…>` | Spawn a process. |
| `keygen process kill  <id>` | **Tier 3 — irreversible.** |
| `keygen process ping  <id>` | Heartbeat. |

### `release` — published releases

CRUD: `list / get / create / update / delete`. Plus:

| Action | Notes |
|---|---|
| `keygen release publish <id>` | **Tier 3 — visible to customers.** |
| `keygen release yank    <id>` | **Tier 3 — irreversible.** |
| `keygen release upgrade --product <…> --version <semver>` | Compute upgrade target. |
| `keygen release constraints attach\|detach\|list <id> [<entitlement>]` | |
| `keygen release packages    attach\|detach\|list <id> [<package>]` | |

### `artifact` — release binaries

CRUD: `list / get / create / update / delete`. Plus:

| Action | Notes |
|---|---|
| `keygen artifact upload   --release <…> --file <path>` | Upload a binary. |
| `keygen artifact download <id> [--out <path>]` | Download. |
| `keygen artifact yank     <id>` | **Tier 3 — irreversible.** |

### `package` — release package groupings

CRUD only.

### `webhook` — endpoints + delivery events

```text
keygen webhook endpoint  list / get / create / update / delete
keygen webhook endpoint  test  <id>            # send a test event
keygen webhook event     list / get
keygen webhook event     retry <id>            # re-deliver a failed event
```

### `request-log` (EE)

CRUD-style read only: `list / get`. Empty payload on Cloud/CE.

### `event-log` (EE)

CRUD-style read only: `list / get`. Empty payload on Cloud/CE. Pair with the
TUI events panel for live tailing.

## Cross-cutting flags worth knowing

- `--dry-run` works on every write command and prints the request envelope
  (method + URL + body) without sending it.
- `--idempotency-key <key>` is forwarded as the `Idempotency-Key` header for
  any POST/PATCH/DELETE.
- `--include rel1,rel2` follows JSON:API include semantics on `list` / `get`.
- `--filter status=ACTIVE --filter user=usr_abc` builds the `filter[…]` query
  string. Each `--filter` may be repeated.
- `--sort -created` sorts descending by `created` (`-` prefix).
