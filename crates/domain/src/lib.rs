//! # hubbub-domain
//!
//! Core domain layer for Hubbub.
//! Contains entities, value objects, ports (traits), and domain errors.
//!
//! ## Rules
//! - This crate MUST NOT depend on any I/O crates (reqwest, sqlx, keyring, notify).
//! - All external interactions are defined as port traits here, implemented by adapter crates.

pub mod entities;
pub mod errors;
pub mod events;
pub mod ports;
pub mod text;
