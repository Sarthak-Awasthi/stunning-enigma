//! Core domain entities and error types for the Mod Manager.
//!
//! This crate defines all the core data structures used throughout the application,
//! including games, instances, profiles, mods, plugins, and runners.

pub mod entities;
pub mod error;

pub use entities::*;
pub use error::*;
