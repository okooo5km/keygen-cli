# Permission tiers

This file is the authoritative tier list. The TUI's command palette and
action panels enforce the same rules in `src/tui/permission.rs`; the agent
should mirror them.

There are three tiers. Pick the highest tier any single command falls into:

- **Tier 1 — auto-run.** Read-only inspection. Run without asking.
- **Tier 2 — dry-run, then confirm.** Mutating but reversible (or low-blast).
  First call must include `--dry-run --json`. Show the user the envelope
  (`request.method`, `request.url`, `request.body`) and only re-run without
  `--dry-run` after explicit user agreement.
- **Tier 3 — explicit approval, no shortcuts.** Destructive or irreversible.
  Restate the command and target id, name the blast radius, and only proceed
  on a clear "yes". Pass `--yes` only at that point.

If two clauses disagree, the higher tier wins.

## Tier 1 — auto-run

| Command | Why |
|---|---|
| `<resource> list` | Read-only listing. |
| `<resource> get` | Read-only fetch. |
| `whoami` | Identity probe. |
| `doctor` | Capability probe. No state change. |
| `schema --format json` | Static dump of the CLI surface. |
| `explain error <CODE>` | Local error dictionary. |
| `config get` | Reads `config.toml`. |
| `profile list` | Reads `config.toml`. |
| `env list` | Reads remote envs (EE). Stateless. |
| `webhook event list` / `webhook event get` | Read-only. |
| `request-log list/get` (EE) | Read-only. |
| `event-log list/get` (EE) | Read-only. |
| `release upgrade --dry-run` | Pure computation. |

Any command with `--dry-run` is automatically Tier 1 for execution purposes:
no API mutation happens. Use this freely to preview a Tier 2/3 call.

## Tier 2 — dry-run, then confirm

Default rule of thumb: any `create` / `update` on any resource, plus any
non-CRUD action that changes state but has a documented reverse action or
narrow blast radius.

| Command | Reverse / Mitigation |
|---|---|
| `<resource> create`           | `<resource> delete <id>` |
| `<resource> update`           | Re-`update` to prior values. |
| `license validate`            | Read-mostly; counts toward usage. |
| `machine activate`            | `machine deactivate <id>`. |
| `machine ping`                | Idempotent heartbeat. |
| `machine reset`               | Resets counter only. |
| `process spawn`               | `process kill <id>` (Tier 3). |
| `process ping`                | Idempotent. |
| `license usage incr`          | Pair with `decr` (Tier 2). |
| `license usage decr`          | Pair with `incr` (Tier 2). |
| `license usage reset`         | No reverse — start of cycle only. |
| `license check-out`           | `license check-in <id>`. |
| `license check-in`            | Cancels a check-out. |
| `machine check-out`           | Same blob as license check-out. |
| `artifact upload` (new file)  | `artifact yank <id>` (Tier 3). |
| `artifact upload` (overwrite) | **Promote to Tier 3** — overwrites a published binary. |
| `artifact download`           | Network/$$ on Cloud — confirm large pulls. |
| `webhook endpoint test`       | Sends a real event payload. Confirm. |
| `webhook event retry`         | Re-delivers; idempotent on the receiver if they ACK. |
| `release upgrade`             | Read-only on EE; billable on Cloud — confirm. |

### Dry-run envelope — how to read it

```bash
keygen license create --json --dry-run --policy pol_abc --user usr_def
```

Returns:

```json
{
  "ok": true,
  "data": {
    "request": {
      "method": "POST",
      "url": "https://api.keygen.sh/v1/accounts/<acct>/licenses",
      "headers": { "Content-Type": "application/vnd.api+json", "...": "..." },
      "body": { "data": { "type": "licenses", "attributes": {…},
                          "relationships": {…} } }
    }
  }
}
```

Show the `method` + `url` + `body` to the user verbatim. If anything looks
wrong (wrong account, wrong policy id, missing fields) — fix the flags and
re-run with `--dry-run` again, never the live version.

## Tier 3 — explicit approval, no shortcuts

These either delete data, disable users, or change state visible to end
customers. Do not pass `--yes` until the user has verbally agreed to the
exact command and target id.

| Command | Reversible? | Notes |
|---|---|---|
| `<resource> delete <id> --yes` | No. | Use `<resource> get <id>` first. |
| `license suspend <id>`         | Yes — `reinstate`. | But blocks customers immediately. |
| `license reinstate <id>`       | Yes — `suspend`. | Restores access. |
| `license renew <id>`           | No clean undo. | Pushes expiry forward. |
| `license revoke <id>`          | **No.** | Cannot be reinstated. |
| `license transfer <id>`        | No clean undo. | Owner change. |
| `release publish <id>`         | Yes — `yank`. | Visible to customers. |
| `release yank <id>`            | **No.** | Customers lose access. |
| `artifact yank <id>`           | **No.** | Removes a published binary. |
| `artifact upload --release <id> --file <…>` (overwrite) | **No.** | Replaces a published binary. |
| `user ban <id>`                | Yes — `unban`. | Blocks sign-in. |
| `user unban <id>`              | Yes — `ban`.   | Lifts a block. |
| `user reset-password <id>`     | n/a | Sends an email; user-visible. |
| `user update-password <id>`    | No clean undo. | Admin override. |
| `process kill <id>`            | **No.** | Aborts a running process. |
| `token regenerate <id>`        | **No.** | Old secret invalidated. |
| Any mutation on a profile whose name matches `prod` / `production`, or the unnamed default profile pointing at `api.keygen.sh`. | n/a | Treat the profile itself as the blast-radius signal. |

### Pre-flight checklist for Tier 3

Before running:

1. Run `keygen <resource> get <id> --json` and show the user the current
   state.
2. Restate the command, the id, and what will change.
3. Wait for an explicit "yes" — not a passive "ok".
4. Pass `--yes` (and `--idempotency-key <slug>` when relevant) on the live
   call.

If the user wants to act on five or more ids, ask once whether to batch with
`xargs -n1 -P1` (sequential, safer) or to script a loop.

## Risk matrix

Add the points below; any Tier 2 command whose total reaches **+2** should be
treated as Tier 3 for the duration of that command:

| Signal | Points |
|---|---|
| Active profile name contains `prod` / `production`. | +1 |
| Active host is `api.keygen.sh` and no profile is set. | +1 |
| Batch size (number of distinct ids) ≥ 5. | +1 |
| Write request lacks `--idempotency-key`. | +1 |
| User has not run `get` on the target this session. | +1 |
| EE-only command but `keygen doctor` reports CE/Cloud. | +1 (also abort and return `Capability` error). |

## When in doubt

Refuse to act and ask. The user can always escalate; you cannot un-revoke.
