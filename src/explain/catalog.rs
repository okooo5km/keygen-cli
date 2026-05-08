//! Static catalogue of keygen.sh error codes — expanded version of the hint
//! map used by the API client. Used by both `keygen explain error <code>` and
//! the per-error hints surfaced inside the `Error::Api` variant.
//!
//! Authored by okooo5km(十里).

#[derive(Debug, Clone, Copy)]
pub struct ErrorEntry {
    pub code: &'static str,
    pub title: &'static str,
    pub cause: &'static [&'static str],
    pub fix: &'static [&'static str],
    pub see_also: &'static [&'static str],
}

pub const CATALOG: &[ErrorEntry] = &[
    ErrorEntry {
        code: "LICENSE_SUSPENDED",
        title: "License has been administratively suspended.",
        cause: &[
            "An admin called `keygen license suspend <id>`.",
            "A webhook integration auto-suspended on a billing event.",
        ],
        fix: &["keygen license reinstate <id>"],
        see_also: &["license.suspended attribute"],
    },
    ErrorEntry {
        code: "LICENSE_EXPIRED",
        title: "License is past its expiry date.",
        cause: &["The license's `expiry` is in the past."],
        fix: &[
            "keygen license renew <id>          # if the policy allows renewal",
            "issue a new license under a longer-duration policy",
        ],
        see_also: &["license.expiry, policy.duration"],
    },
    ErrorEntry {
        code: "LICENSE_NOT_ACTIVATED",
        title: "License has no active machines.",
        cause: &["The user has not yet activated a machine for this license."],
        fix: &[
            "keygen machine activate --license <lid> --fingerprint <fp>",
        ],
        see_also: &["machine activate, machine fingerprint"],
    },
    ErrorEntry {
        code: "LICENSE_TOO_MANY_MACHINES",
        title: "License has reached its machine cap.",
        cause: &[
            "policy.maxMachines reached.",
            "Old hardware is still registered (deactivation needed).",
        ],
        fix: &[
            "keygen machine list --filter license=<lid>",
            "keygen machine deactivate <id>          # remove old hardware",
            "raise policy.maxMachines on the license's policy",
        ],
        see_also: &["policy.maxMachines, machine deactivate"],
    },
    ErrorEntry {
        code: "MACHINE_LIMIT_EXCEEDED",
        title: "Machine cap exceeded for this license / policy.",
        cause: &["A new activation would exceed policy.maxMachines."],
        fix: &[
            "deactivate an old machine: keygen machine deactivate <id>",
            "raise policy.maxMachines",
        ],
        see_also: &["policy.maxMachines"],
    },
    ErrorEntry {
        code: "MACHINE_HEARTBEAT_DEAD",
        title: "Machine missed its heartbeat window.",
        cause: &[
            "policy.requireHeartbeat=true and policy.heartbeatDuration elapsed without a `machine ping`.",
        ],
        fix: &[
            "keygen machine ping <id>",
            "set policy.requireHeartbeat=false if heartbeats are not desired",
        ],
        see_also: &["machine ping, policy.heartbeatDuration"],
    },
    ErrorEntry {
        code: "FINGERPRINT_TAKEN",
        title: "Fingerprint already registered to a different machine.",
        cause: &["Two activations using the same hardware fingerprint."],
        fix: &[
            "deactivate the previous machine before re-activating",
            "use a unique fingerprint per device",
        ],
        see_also: &["machine.fingerprint"],
    },
    ErrorEntry {
        code: "TOKEN_INVALID",
        title: "Bearer token is malformed or unknown.",
        cause: &[
            "Token was revoked or rotated.",
            "Wrong account / environment context (token was issued for another account).",
        ],
        fix: &[
            "keygen login",
            "ensure KEYGEN_TOKEN, KEYGEN_ACCOUNT, KEYGEN_HOST match the deployment",
        ],
        see_also: &["keygen login, profile config"],
    },
    ErrorEntry {
        code: "TOKEN_EXPIRED",
        title: "Bearer token has expired.",
        cause: &["Token reached its `expiry` timestamp."],
        fix: &["keygen login", "keygen token regenerate <id>"],
        see_also: &["token.expiry"],
    },
    ErrorEntry {
        code: "TOKEN_FORBIDDEN",
        title: "Token does not have permission for this action.",
        cause: &[
            "Product token used for an admin-only endpoint.",
            "User token without sufficient permissions.",
        ],
        fix: &["use an admin or product token with the right scopes"],
        see_also: &["token.permissions, role"],
    },
    ErrorEntry {
        code: "FORBIDDEN",
        title: "Active credentials cannot perform this operation.",
        cause: &["Token role / permissions don't allow this operation."],
        fix: &["log in with an admin token", "request the operator to grant the missing scope"],
        see_also: &["token, role, permissions"],
    },
    ErrorEntry {
        code: "UNAUTHORIZED",
        title: "Request did not include valid credentials.",
        cause: &["Missing Authorization header.", "Wrong scheme (Bearer vs License)."],
        fix: &["keygen login", "set KEYGEN_TOKEN before invoking"],
        see_also: &["auth"],
    },
    ErrorEntry {
        code: "NOT_FOUND",
        title: "Resource id does not exist (or is in another environment).",
        cause: &["Wrong id.", "Wrong environment for an EE deployment."],
        fix: &[
            "keygen <resource> list  # find the right id",
            "keygen env switch <id>  # if on EE",
        ],
        see_also: &["env, account"],
    },
    ErrorEntry {
        code: "VALIDATION_FAILED",
        title: "Request body failed schema validation.",
        cause: &["Required attribute missing.", "Wrong type for a field."],
        fix: &[
            "keygen <resource> <action> --json --dry-run  # to inspect what's sent",
            "consult the keygen.sh docs for the resource's required fields",
        ],
        see_also: &["request body, --set"],
    },
    ErrorEntry {
        code: "RATE_LIMIT_EXCEEDED",
        title: "Too many requests in a short window.",
        cause: &["The keygen.sh per-account or per-IP rate limit was hit."],
        fix: &["back off and retry after the `Retry-After` window", "batch operations where possible"],
        see_also: &["rate limit headers"],
    },
    ErrorEntry {
        code: "POLICY_PROTECTED",
        title: "Cannot modify a protected policy.",
        cause: &["policy.protected=true blocks attribute changes."],
        fix: &["set protected=false on the policy first", "use a different policy"],
        see_also: &["policy.protected"],
    },
    ErrorEntry {
        code: "USER_BANNED",
        title: "User account is banned.",
        cause: &["An admin banned the user with `keygen user ban`."],
        fix: &["keygen user unban <id>"],
        see_also: &["user ban / unban"],
    },
    ErrorEntry {
        code: "USER_NOT_VERIFIED",
        title: "User has not verified their email.",
        cause: &["The user signed up but never confirmed via email."],
        fix: &["resend the verification email; complete the verification flow"],
        see_also: &["user.verified"],
    },
    ErrorEntry {
        code: "GROUP_USER_LIMIT_EXCEEDED",
        title: "Group has reached its maxUsers cap.",
        cause: &["group.maxUsers reached."],
        fix: &["increase group.maxUsers", "remove an existing member"],
        see_also: &["group.maxUsers"],
    },
    ErrorEntry {
        code: "ENTITLEMENT_CONSTRAINT_FAILED",
        title: "License doesn't satisfy required entitlement constraints.",
        cause: &["Validation scope listed an entitlement the license lacks."],
        fix: &[
            "keygen policy entitlements list <pid>",
            "keygen policy entitlements attach <pid> --entitlement <eid>",
        ],
        see_also: &["entitlements, validate scope"],
    },
    ErrorEntry {
        code: "ARTIFACT_NOT_UPLOADED",
        title: "Release artifact is missing its binary.",
        cause: &["The artifact resource exists but `keygen artifact upload` was never run."],
        fix: &["keygen artifact upload <id> --file <path>"],
        see_also: &["artifact upload"],
    },
    ErrorEntry {
        code: "RELEASE_YANKED",
        title: "Release was yanked and cannot be re-published.",
        cause: &["A yanked release is read-only."],
        fix: &["create a new release", "consult: keygen release get <id>"],
        see_also: &["release.yanked"],
    },
    ErrorEntry {
        code: "PRODUCT_NOT_FOUND",
        title: "Referenced product does not exist in this account.",
        cause: &["Wrong product id.", "Wrong account context."],
        fix: &[
            "keygen product list",
            "verify KEYGEN_ACCOUNT or active profile",
        ],
        see_also: &["account, product list"],
    },
    ErrorEntry {
        code: "ACCOUNT_BILLING_DELINQUENT",
        title: "Account has an outstanding billing issue.",
        cause: &["Card on file failed.", "Subscription lapsed."],
        fix: &["update billing info on keygen.sh dashboard"],
        see_also: &["billing"],
    },
    ErrorEntry {
        code: "WEBHOOK_DELIVERY_FAILED",
        title: "Webhook endpoint did not return 2xx.",
        cause: &["Endpoint URL is down or returning errors."],
        fix: &[
            "keygen webhook events retry <eid>",
            "verify the endpoint is reachable and accepting POST",
        ],
        see_also: &["webhook events retry"],
    },
    ErrorEntry {
        code: "FILESIZE_EXCEEDS_LIMIT",
        title: "Uploaded file exceeds the artifact size cap.",
        cause: &["The binary is larger than your plan allows."],
        fix: &["compress / split the binary", "upgrade plan"],
        see_also: &["artifact upload"],
    },
    ErrorEntry {
        code: "INVALID_FILTER",
        title: "Filter key not recognized for this resource.",
        cause: &["Misspelled filter param.", "Filter not supported on this resource."],
        fix: &[
            "consult the keygen.sh docs for valid filter keys",
            "remove the offending --filter k=v",
        ],
        see_also: &["list filters"],
    },
    ErrorEntry {
        code: "FILTER_UNSUPPORTED",
        title: "Server ignored a filter parameter and returned an unfiltered collection.",
        cause: &[
            "Keygen.sh uses top-level query params (e.g. `?policy=<id>`), not the JSON:API \
             `filter[<key>]` namespace; CLI < 0.3.1 wrapped every `--filter k=v` in `filter[]`, \
             which the server silently dropped.",
            "An older CLI is talking to a current Keygen deployment, or a typo / unknown key \
             slipped through, so the server returned the full collection.",
        ],
        fix: &[
            "upgrade `keygen-cli` to 0.3.1+ — it sends `?<key>=<value>` directly",
            "verify the filter key against the resource's docs (`keygen <res> list --help`)",
            "`keygen doctor` shows whether the active deployment honors relation filters",
        ],
        see_also: &["doctor", "list filters"],
    },
    ErrorEntry {
        code: "VALIDATION_FINGERPRINT_REQUIRED",
        title: "License validation requires a fingerprint scope.",
        cause: &["policy.requireFingerprintScope=true."],
        fix: &["keygen license validate <id> --fingerprint <fp>"],
        see_also: &["license validate"],
    },
    ErrorEntry {
        code: "PROCESS_HEARTBEAT_DEAD",
        title: "Process missed its heartbeat window.",
        cause: &["Process was not pinged within heartbeat duration."],
        fix: &["keygen process ping <id>"],
        see_also: &["process heartbeat"],
    },
    ErrorEntry {
        code: "ENV_FORBIDDEN",
        title: "Active token does not have access to this environment.",
        cause: &["EE-only: token was issued for a different environment."],
        fix: &["keygen env list", "keygen env switch <id>"],
        see_also: &["env (EE)"],
    },
    ErrorEntry {
        code: "CHECKOUT_REQUIRED",
        title: "Policy requires a check-out before validation.",
        cause: &["policy.requireCheckIn=true and last check-out has expired."],
        fix: &["keygen license check-out <id>"],
        see_also: &["license check-out"],
    },
];

#[must_use]
pub fn lookup(code: &str) -> Option<&'static ErrorEntry> {
    let upper = code.to_ascii_uppercase();
    CATALOG.iter().find(|e| e.code == upper)
}

#[must_use]
pub fn list_codes() -> Vec<&'static str> {
    CATALOG.iter().map(|e| e.code).collect()
}
