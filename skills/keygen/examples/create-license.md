# Example — issue a license end to end

End-to-end recipe for the most common ask: "create a license for this
customer and activate it on their machine." Every step shows the dry-run
preview an agent should display before running the live command.

## Prerequisites

```bash
keygen whoami         # confirm identity + active profile
keygen doctor         # confirm host reachable + capabilities
```

If `whoami` fails with exit code 4, run `keygen login` first.

## 1. Pick the policy and user

```bash
keygen policy list --filter product=prod_abc --json | jq '.data[] | { id, attrs: .attributes.name }'
keygen user   list --filter email=alice@example.com --json | jq '.data[] | { id, attrs: .attributes }'
```

These are Tier 1 — no confirmation needed. Capture `pol_xyz` and `usr_def`.

## 2. Dry-run the create

```bash
keygen license create --json --dry-run \
  --policy pol_xyz \
  --user   usr_def \
  --set attrs.name="Acme — Pro 2026" \
  --metadata seat=enterprise \
  --metadata renewal=annual
```

Shows the request envelope. Verify `url` ends in `/licenses`, `body.data.relationships.policy.data.id` is `pol_xyz`, and the metadata is what
you want.

## 3. Live create

After the user agrees:

```bash
keygen license create --json \
  --policy pol_xyz \
  --user   usr_def \
  --set attrs.name="Acme — Pro 2026" \
  --metadata seat=enterprise \
  --metadata renewal=annual \
  --idempotency-key license-acme-2026-01
```

Capture `data.id` (the license id, `lic_…`) and `data.attributes.key` (the
license key your customer types into your app).

## 4. Activate a machine

`activate` is Tier 2 — dry-run first:

```bash
keygen machine activate --json --dry-run \
  --license     lic_abc123 \
  --fingerprint 8e2f-…-acme-laptop-01 \
  --set attrs.name="Alice's MacBook Pro"
```

Then live:

```bash
keygen machine activate --json \
  --license     lic_abc123 \
  --fingerprint 8e2f-…-acme-laptop-01 \
  --set attrs.name="Alice's MacBook Pro" \
  --idempotency-key machine-acme-laptop-01
```

## 5. Validate from the customer's side

For the offline-style validation that ships with most apps, hand the
customer the license **key** (not the id). They validate without auth:

```bash
keygen license validate-key <key>
```

Returns `meta.valid: true` and `meta.code: "VALID"` when healthy.

## 6. (Optional) check out an offline blob

For air-gapped customers, mint a signed `.lic`:

```bash
keygen license check-out lic_abc123 --json --out alice.lic
```

The customer ships `alice.lic` with their machine and verifies it offline:

```bash
keygen license verify --key "$(cat alice.lic)" \
  --public-key-file ./product-public.pem
```

See [`offline-verify.md`](./offline-verify.md) for the full offline path.

## Reverse / cleanup

If something went wrong:

| Action | Tier |
|---|---|
| `keygen machine deactivate <id>`      | 2 (reversible). |
| `keygen license suspend <id>`         | 3 — pair with `reinstate`. |
| `keygen license delete <id> --yes`    | 3 — irreversible. |
