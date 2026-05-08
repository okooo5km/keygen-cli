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

## Keybindings — Stage a (already shipping)

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

## Stage b — Action panel *(planned)*

Once Stage b lands, pressing `a` on a selected row opens an action menu
populated from the resource's non-CRUD verbs (license: `validate`,
`suspend`, `reinstate`, `renew`, `revoke`, `check-out`, `check-in`,
`transfer`, `usage incr/decr/reset`; machine: `activate`, `ping`, `reset`,
`check-out`; etc.).

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

## Stage c — Events panel *(planned)*

A right-bottom panel polls webhook events (Cloud + CE with webhooks
configured) or the EE event log every five seconds, newest first. New rows
pulse a single colour beat for ~200 ms before settling into the normal
status colour.

| Key | Action |
|---|---|
| `e` | Toggle full-screen events view. |
| `End` | Jump to the latest event (auto-follow). |
| `PgUp` / `PgDn` | Scroll without dropping auto-follow. |

CE deployments without webhook events show a placeholder, not an error.

## Stage d — Command palette *(planned)*

Press `:` to drop into a command-palette modal. Any `keygen` subcommand
(without the `keygen` prefix) is parsed by the same clap tree the binary
uses, run in-process, and the resulting envelope is rendered into the
result pane.

| Key (in palette) | Action |
|---|---|
| `Tab` | Auto-complete the current token from the in-memory schema. |
| `Enter` | Run the command (Tier 2/3 still gated by the confirm overlay). |
| `Esc` | Close without running. |
| `Ctrl-P` / `Ctrl-N` | Recall previous / next command. |

The palette honours the same Tier 1/2/3 rules as the action panel — typing
`license revoke abc` triggers the Tier 3 confirm overlay before sending
anything to the API.

## Implementation notes

- All three planned stages share `src/tui/permission.rs`, which is the
  single source for the tier classification used in this skill.
- The dry-run envelope shown by the confirm overlay is exactly the JSON the
  CLI would print under `--dry-run --json` — see
  [`ai-envelope.md`](./ai-envelope.md).
- `keygen tui` never bypasses the keyring: it reads the active profile's
  token the same way the CLI does. `KEYGEN_TOKEN` overrides if set.
