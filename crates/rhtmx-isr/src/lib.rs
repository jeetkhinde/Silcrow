//! # RHTMX ISR - Incremental Static Regeneration
//!
//! This crate provides caching and revalidation capabilities for RHTMX applications.
//!
//! ## Features
//!
//! - **Multiple Storage Backends**: Memory, Filesystem, Dragonfly (Redis)
//! - **Stale-While-Revalidate**: Serve cached content while updating in background
//! - **Configurable Revalidation**: Time-based and on-demand revalidation
//! - **Fallback Support**: Primary + fallback storage for reliability
//!
//! ## Example
//!
//! ```rust
//! use rhtmx_isr::{IsrEngine, IsrConfig, StorageBackend};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = IsrConfig {
//!         default_revalidate: Duration::from_secs(60),
//!         storage: StorageBackend::Memory,
//!         fallback: None,
//!     };
//!
//!     let engine = IsrEngine::new(config).await.unwrap();
//!
//!     // Use in your request handlers
//! }
//! ```

pub mod engine;
pub mod config;
pub mod cache;
pub mod storage;

pub use engine::IsrEngine;
pub use config::{IsrConfig, StorageBackend};
pub use cache::CachedPage;
