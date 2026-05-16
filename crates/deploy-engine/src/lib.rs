//! Deploy engine for the Mod Manager.
//!
//! This crate handles mod deployment through symlink-based file management:
//! - Building deployment plans with conflict resolution
//! - Applying deployments by creating symlinks
//! - Rolling back deployments when needed
//!
//! # Architecture
//!
//! The deploy engine uses a three-phase approach:
//! 1. **Plan**: Query enabled mods, resolve conflicts based on priority
//! 2. **Apply**: Create symlinks from mod files to game Data directory
//! 3. **Rollback**: Remove symlinks and mark manifest as rolled back

pub mod apply;
pub mod conflict;
pub mod plan;

pub use apply::{apply_plan, rollback};
pub use plan::{DeployPlan, build_plan};
