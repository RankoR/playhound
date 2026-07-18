#![forbid(unsafe_code)]
//! Maintenance helpers that are intentionally excluded from the published crate.

mod sanitize;

pub use sanitize::{FixtureKind, sanitize};
