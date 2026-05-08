# `keygen tui`

`keygen tui` opens a full-screen dashboard built on ratatui. It pulls the
same data the CLI does and uses the same permission tiers, so any write
action triggered from the TUI obeys the rules in
[`permissions.md`](./permissions.md).

## Layout

```
┌──────────────────────────────────────────────────────────────────────────┐
│ resources tabs   ·   profile · host · capabilities                       │
├──────────────────────────────┬───────────────────────────────────────────┤
│ master list (rows)           │ detail panel (k/v table)                  │
│   filter / sort indicators   │                                           │
│                              ├───────────────────────────────────────────┤
│                              │ events panel (auto-poll, see Stage c)     │
├──────────────────────────────┴───────────────────────────────────────────┤
│ status bar · keys hint                                                   │
└──────────────────────────────────────────────────────────────────────────┘
```

A `--cards` shortcut (or pressing `c` inside the TUI) flips the master list
into a card layout for resources whose `view_for_jsonapi_type` defines a
card form.

## Browsing keys

| Key | Action |
|---|---|
| `Tab` / `Shift-Tab` | Cycle through resource tabs. |
| `↑` / `↓` / `j` / `k` | Move row selection. |
| `g` / `G` | Jump to first / last row. |
| `Enter` | Open detail for the selected row. |
| `Esc` | Close detail / dismiss overlay. |
| `c` | Toggle table ↔ cards layout. |
| `r` | Reload the current resource. |
| `?` | Help overlay (lists every binding). |
| `q` / `Ctrl-C` | Quit. |

## Action panel

Press `a` on a selected row to open an action menu populated from the
resource's non-CRUD verbs (license: `validate`, `suspend`, `reinstate`,
`renew`, `revoke`, `usage incr/decr/reset`; machine: `ping`, `reset`,
`deactivate`; webhook endpoint: `test`).

| Key | Action |
|---|---|
| `a` | Open action panel for the selected row. |
| `↑` / `↓` / `j` / `k` | Move within the panel. |
| `Enter` | Run the selected action. |
| `?` | Show full descriptions for every action. |

Tier 2 / Tier 3 actions trigger a confirm overlay first. The overlay
displays the request envelope (method + URL + body) the API would receive,
plus a red banner for Tier 3.

| Key (in confirm) | Action |
|---|---|
| `y` | Execute. |
| `n` / `Esc` | Cancel. |

## Events panel

A right-bottom panel polls `/webhook-events` every five seconds, newest
first. New rows render with a yellow timestamp pill until the next redraw,
then settle into the muted style. Status colours come from the event's
`status` attribute (`DELIVERED` green / `FAILED` red / `RETRYING` yellow);
fetch errors surface as a one-line red placeholder rather than blocking
the UI.

| Key | Action |
|---|---|
| `e` | Toggle full-screen events view. |

CE deployments without webhook events show a `(no webhook events yet —
wired? configure an endpoint to see deliveries here)` placeholder, no
error.

## Command palette

Press `:` to drop into a vim-style palette. Any `keygen` subcommand
(without the `keygen` prefix) is parsed by the same clap tree the binary
uses; the command runs by forking the current binary so it reuses the
keyring lookup, idempotency-key handling, and JSON output pipeline.

| Key (in palette) | Action |
|---|---|
| `Tab` | Auto-complete the current token (resource names, then per-resource subcommands). |
| `Enter` | Parse and run — Tier 1 executes immediately; Tier 2/3 stages a confirm banner first. |
| `y` | Confirm a staged Tier 2/3 command. |
| `n` / `Esc` | Cancel the staged command, or close the palette. |

The palette honours the same Tier 1/2/3 rules as the action panel — typing
`license revoke abc` stages a destructive-banner confirm before sending
anything to the API. An in-process `OutputSink` refactor (avoiding the
fork) is tracked for v2.

## Implementation notes

- The action panel and the command palette share
  `src/tui/permission.rs`, which is the single source for the tier
  classification used in this skill.
- The dry-run envelope shown by the confirm overlay is exactly the JSON the
  CLI would print under `--dry-run --json` — see
  [`ai-envelope.md`](./ai-envelope.md).
- `keygen tui` never bypasses the keyring: it reads the active profile's
  token the same way the CLI does. `KEYGEN_TOKEN` overrides if set.
