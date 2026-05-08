//! `keygen explain error <code>` — turn opaque keygen.sh error codes into a
//! useful diagnosis with cause + fix.
//!
//! Authored by okooo5km(十里).

pub mod catalog;
pub mod commands;

pub use catalog::{lookup, ErrorEntry};
