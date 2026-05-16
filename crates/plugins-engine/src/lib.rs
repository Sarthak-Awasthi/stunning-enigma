//! Plugin management for the Mod Manager.
//!
//! This crate provides plugin (ESM/ESP/ESL) handling:
//! - Binary parsing of plugin headers to extract metadata
//! - Master dependency validation
//! - Load order synchronization and persistence
//! - LOOT integration for automatic sorting

pub mod load_order;
pub mod parser;
pub mod validate;

pub use load_order::{sync_plugins, write_load_order};
pub use parser::{PluginHeader, parse_plugin_header};
pub use validate::validate_masters;
