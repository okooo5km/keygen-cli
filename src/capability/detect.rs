//! Capability detection: probes `/v1/profile` (or `/v1/whoami`) and infers
//! which optional features the deployment exposes. Cached under
//! `$XDG_CACHE_HOME/keygen/capabilities.json` with a 1-day TTL.
//!
//! Implemented in step 4.
