# AI envelope and exit codes

`keygen` emits a stable JSON envelope under `--json` (or `--output json`).
Every command — CRUD, action verb, `doctor`, `schema`, `explain` — uses the
same shape. Adding fields is allowed; renaming or removing them is not.

## Envelope shapes

There are exactly three shapes. Detect them by inspecting `ok` and the type
of `data`.

### 1. Single resource

```json
{
  "ok": true,
  "data": {
    "id": "lic_abc",
    "type": "licenses",
    "attributes": { "name": "...", "status": "ACTIVE", "...": "..." },
    "relationships": { "policy": { "data": { "type": "policies", "id": "pol_xyz" } } }
  },
  "meta": { "valid": true, "code": "VALID" }   /* optional */
}
```

`meta` appears for endpoints whose verdict lives there (e.g. `license
validate` returns `meta.valid` + `meta.code`).

### 2. List

```json
{
  "ok": true,
  "data": [
    { "id": "lic_a", "type": "licenses", "attributes": { "..." : "..." } },
    { "id": "lic_b", "type": "licenses", "attributes": { "..." : "..." } }
  ],
  "meta": { "page": 1, "limit": 50, "count": 2 }
}
```

`meta.count` may be absent when the API doesn't supply a total (filtered
endpoints). Pagination is via `meta.page` + `meta.limit`; ask for the next
page with `--page N`.

### 3. Free-form bag

Returned by commands that don't map to a JSON:API resource (e.g.
`<resource> delete`, `keygen explain error <code>`, `keygen doctor`).

```json
{ "ok": true, "data": { "deleted": "lic_abc" } }
{ "ok": true, "data": { "code": "LICENSE_SUSPENDED", "title": "...", "fix": "..." } }
```

### Error shape

```json
{
  "ok": false,
  "error": {
    "kind": "api",
    "http_status": 422,
    "code": "LICENSE_SUSPENDED",
    "title": "license is suspended",
    "detail": "must reinstate before validating",
    "source": "/data/attributes/status",
    "request_id": "req_abc",
    "hint": "run `keygen license reinstate <id>`"
  }
}
```

Field reference:

| Field | Meaning |
|---|---|
| `kind` | One of: `api` (server-side error), `auth`, `network`, `capability`, `user`, `config`, `serde`, `io`, `other`. |
| `http_status` | Original HTTP status. Present for `kind: "api"`. |
| `code` | Stable keygen.sh error code (e.g. `LICENSE_NOT_FOUND`). |
| `title` / `detail` | Server-supplied human-readable strings. |
| `source` | JSON pointer into the offending request body, when available. |
| `request_id` | Server request id for support. |
| `hint` | CLI-supplied hint, often a "next command" suggestion. |

When the error has a `code`, run `keygen explain error <CODE>` (Tier 1) for
a structured cause + fix entry.

## Dry-run envelope

`--dry-run` short-circuits before the network call and returns the request
that would have been sent. The shape is always:

```json
{
  "ok": true,
  "data": {
    "request": {
      "method": "POST",
      "url":    "https://api.keygen.sh/v1/accounts/<acct>/licenses",
      "headers": { "Authorization": "Bearer …", "Content-Type": "application/vnd.api+json" },
      "body":   { "data": { "type": "licenses", "attributes": {…} } }
    }
  }
}
```

Always `ok: true`. The API was not contacted.

## Exit codes

Stable across the major version. Branch on these instead of parsing strings.

| Code | `ExitKind`     | Meaning |
|------|----------------|---------|
| `0`  | `Ok`           | Command succeeded. |
| `1`  | `UserError`    | Bad flags, missing args, 4xx that wasn't auth. |
| `2`  | `ServerError`  | 5xx from the API. Retry-able. |
| `3`  | `NetworkError` | DNS, TCP, TLS, or timeout. Retry-able. |
| `4`  | `AuthError`    | 401, 403, or no usable credential. |
| `5`  | `Capability`   | Command requires EE/Cloud and the host doesn't offer it. |

## Decision flow for agents

1. Parse stdout JSON. If `ok: true`, use `data` (and `meta`, if present).
2. If `ok: false`, capture `error.code`. Run `keygen explain error <code>`
   for a fix recipe before retrying.
3. If exit code is `3` (network) or `2` (server), the request can usually be
   retried — pass `--idempotency-key <slug>` on the retry to avoid duplicate
   writes.
4. If exit code is `4` (auth), prompt the user to run `keygen login` (or set
   `KEYGEN_TOKEN`) before retrying.
5. If exit code is `5` (capability), surface the mismatch instead of retrying
   — the host genuinely doesn't support this command.

## Getting the schema programmatically

```bash
keygen schema --format json | jq '.data.command.subcommands[] | { name, about }'
```

The schema is generated from the `clap` derive tree, so it stays in sync
with the binary at all times. Use it to verify a flag exists before
recommending it to the user.
