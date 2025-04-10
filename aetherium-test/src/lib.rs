//! This crate contains mocks and utilities for testing Aetherium agents.

#![forbid(unsafe_code)]
#![cfg_attr(test, warn(missing_docs))]
#![allow(unknown_lints)] // TODO: `rustc` 1.80.1 clippy issue
#![forbid(where_clauses_object_safety)]

/// Mock contracts
pub mod mocks;
