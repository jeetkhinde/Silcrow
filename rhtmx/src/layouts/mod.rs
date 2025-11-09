// RHTMX Layouts Module
// Simple, type-safe layout system using plain Rust functions

pub mod root;
pub mod admin;

// Re-export for convenience
pub use root::layout as root_layout;
pub use admin::layout as admin_layout;
