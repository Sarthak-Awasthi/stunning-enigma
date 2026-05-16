//! Mod ingestion pipeline for the Mod Manager.
//!
//! This crate handles the complete mod ingestion workflow:
//! - Archive format detection (ZIP, 7z)
//! - SHA-256 hashing for deduplication
//! - Archive extraction with wrapper normalization
//! - FOMOD install path resolution
//! - File indexing and database registration

pub mod archive;
pub mod fomod;
pub mod hasher;
pub mod ingest;

pub use ingest::{IngestResult, ingest_mod};
