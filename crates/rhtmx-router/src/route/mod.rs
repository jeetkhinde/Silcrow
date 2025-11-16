/// Route module for file-based routing
///
/// Contains pure functional components for route parsing and matching.
/// All modules follow functional programming principles:
/// - Pure functions (same input → same output)
/// - Immutable data structures
/// - Pattern matching for control flow
/// - Zero-copy optimizations where possible

pub mod pattern;

// Re-export commonly used types
pub use pattern::{classify_segment, parse_param_with_constraint, PatternSegmentType};
