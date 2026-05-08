# keygen-cli — recipes

Task-oriented cookbook for `keygen`. Every recipe pairs a real-user scenario
with a copy-pastable shell block. Tier reminders (1 = read-only, 2 = dry-run
first, 3 = explicit approval) point back to
[`permissions.md`](./permissions.md).

Authored by okooo5km(十里).

---

## 1. Customer support

### "My license stopped working"

```bash
# Tier 1 — inspect the license + its policy + recent machines.
keygen license get <lid> --include policy --json | jq '.data'
keygen machine list --filter license=<lid> --json | jq '.data[] | {id, attributes:.attributes}'
keygen license validate <lid> --json | jq '.data.attributes'   # canonical reason
```

`license validate` returns the precise `code` (e.g. `EXPIRED`,
`SUSPENDED`, `NO_MACHINES`); pipe it into `keygen explain error <CODE>` for
the canonical fix.

### "I can't activate — too many machines"

```bash
keygen machine list --filter license=<lid> --json \
  | jq '.data[] | {id, name:.attributes.name, lastHeartbeat:.attributes.lastHeartbeat}'

# Tier 3 — only after the user names the stale machine to retire.
keygen machine deactivate <mid>
```

### "The license is gone"

```bash
keygen license list --filter user=<uid> --json | jq '.data[].id'
keygen license list --filter product=<pid> --filter status=INACTIVE --json
```

Note: relation filters (`license`, `user`, `product`, ...) may be ignored
on self-hosted CE — see "Filter behavior differences" below.

### Refund / takedown

```bash
# Tier 3 — irreversible.
keygen license suspend <lid>            # soft pause
keygen license revoke  <lid>            # hard kill, validation will fail
```

---

## 2. Audit / reconciliation

### Active license count by policy

```bash
keygen license list --filter status=ACTIVE --limit 200 --json \
  | jq '[.data[] | .relationships.policy.data.id] | group_by(.) | map({policy:.[0], count:length})'
```

### Expiring next 30 days

```bash
keygen license list --json --limit 200 \
  | jq --arg cutoff "$(date -v +30d -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null \
                       || date -u -d '+30 days' +%Y-%m-%dT%H:%M:%SZ)" \
       '[.data[] | select(.attributes.expiry != null and .attributes.expiry <= $cutoff)
                 | {id, expiry:.attributes.expiry}]'
```

### "Which licenses include entitlement X?"

```bash
# 1. Find policies that carry the entitlement.
keygen policy list --include entitlements --limit 200 --json \
  | jq --arg e <eid> '.data[] | select(.relationships.entitlements.data[].id == $e) | .id'

# 2. Then list licenses on each matching policy.
keygen license list --filter policy=<pid> --json
```

### Webhook delivery health

```bash
keygen webhook event list --filter status=FAILED --limit 50 --json \
  | jq '.data[] | {id, endpoint:.relationships.endpoint.data.id, last:.attributes.lastResponseCode}'
```

---

## 3. Release operations

### Cut a release + upload binary + publish

```bash
# Tier 2 — dry-run, then commit.
keygen release create --dry-run --json --product <pid> --set attrs.version=1.4.0
keygen release create            --json --product <pid> --set attrs.version=1.4.0
keygen artifact upload --release <rel_id> --file ./dist/app-1.4.0.dmg --json

# Tier 3 — public, gated visibility flip.
keygen release publish <rel_id>
```

### Yank a bad release

```bash
keygen release yank <rel_id>            # Tier 3
keygen webhook event list --filter event=release.yanked --limit 5 --json
```

### Replay a failed webhook

```bash
keygen webhook event list --filter status=FAILED --json | jq '.data[].id'
keygen webhook event retry <eid>        # Tier 2
```

---

## 4. Batch operations

### Issue N licenses from a CSV of users

```bash
# Tier 2 — dry-run on the first row, then loop.
while IFS=, read -r email user_id; do
  keygen license create --json \
    --policy <pid> --user "$user_id" \
    --set attrs.name="$email — auto" \
    --metadata source=batch \
    --idempotency-key "batch-$(date +%s)-$user_id"
done < users.csv
```

Always pass `--idempotency-key` on bulk writes — the same key replayed
returns the original create rather than producing a duplicate.

### Bulk renew

```bash
keygen license list --filter status=EXPIRED --limit 200 --json \
  | jq -r '.data[].id' \
  | while read lid; do
      keygen license renew "$lid" --json   # Tier 3 — confirm the count first
    done
```

### Transfer licenses to a new user

```bash
keygen license transfer <lid> --user <new_uid> --json    # Tier 3
```

---

## 5. Offline / air-gapped

### Check-out, distribute, verify

```bash
# On a connected admin machine.
keygen license check-out <lid> --ttl 7d --output json > license-checkout.json

# Ship license-checkout.json to the air-gapped box. Then on the device:
keygen license verify --offline --file license-checkout.json --json
```

`license verify --offline` validates the embedded signature without
contacting keygen.sh — works against `ed25519` and `rsa-pss-sha256`.

### Re-check before TTL expiry

```bash
keygen license check-out <lid> --ttl 7d --output json > license-checkout.json
# Replace the file on the device; verify on next launch.
```

---

## 6. Troubleshooting

### Decode an error envelope

```bash
keygen <anything> --json | jq '.error'
keygen explain error <CODE>             # canonical cause + fix
```

### See what the server saw

```bash
# EE only — request log records the most recent API calls.
keygen request-log list --limit 20 --json
keygen request-log get <rid> --json | jq '.data.attributes'
```

### Tail webhook events

```bash
keygen webhook event list --limit 20 --json     # snapshot
keygen tui                                       # live tail (events panel)
```

### Capability mismatch

```bash
keygen doctor --json | jq '.data.checks'
```

`doctor` runs `client → auth → capabilities → filters_relation`. If a
check fails it returns exit 1 with a structured envelope so an agent can
branch on the specific gap rather than re-running blind.

---

## Filter behavior

Keygen.sh's filter contract is **top-level query params**, not JSON:API's
`filter[<key>]` namespace:

```bash
# correct — what the docs and the CLI emit
GET /v1/accounts/<acct>/licenses?policy=<pid>&status=ACTIVE
GET /v1/accounts/<acct>/licenses?metadata[seat]=enterprise
GET /v1/accounts/<acct>/licenses?expires[in]=7d
```

CLI < 0.3.1 wrapped every `--filter k=v` in `filter[]`; the server
silently dropped the unknown key and returned the unfiltered collection
(the bug behind issue #1). 0.3.1+ sends keys verbatim and adds two
defenses on top:

- **Post-fetch audit** — for relation filters
  (`license, user, product, policy, group, owner, machine, environment`),
  every returned row's `relationships.<key>.data.id` must equal the
  filter value. Mismatches surface as `error.code = FILTER_UNSUPPORTED`
  (exit 1).
- **Doctor probe** — `keygen doctor` runs `filters_relation`; reports
  whether the active deployment honors relation filters at all.

Attribute filters (`status`, `expires`, `key`) are not audited — their
semantics (range, boolean, substring) make post-hoc verification
ambiguous. If audit ever rejects a filter you trust, fall back to
client-side filtering:

```bash
keygen machine list --limit 200 --json \
  | jq --arg lid <lid> '[.data[] | select(.relationships.license.data.id == $lid)]'
```
