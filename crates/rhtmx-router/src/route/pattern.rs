/// Pattern parsing for route segments
///
/// Pure functional parsing of file-based route patterns into typed segments.
/// All functions are **pure**: same input → same output, no side effects.

use crate::ParameterConstraint;

/// Represents different types of route pattern segments
///
/// Functional sum type for pattern matching route segments.
/// Each variant carries the parameter name and optional constraint.
///
/// # Examples
///
/// ```
/// use rhtmx_router::route::pattern::{classify_segment, PatternSegmentType};
///
/// // Static segment
/// let seg = classify_segment("about");
/// assert!(matches!(seg, PatternSegmentType::Static(_)));
///
/// // Required parameter
/// let seg = classify_segment("[id]");
/// assert!(matches!(seg, PatternSegmentType::Required(_, _)));
///
/// // Optional parameter with constraint
/// let seg = classify_segment("[id:int?]");
/// assert!(matches!(seg, PatternSegmentType::Optional(_, Some(_))));
///
/// // Catch-all with constraint
/// let seg = classify_segment("[...slug:alpha]");
/// assert!(matches!(seg, PatternSegmentType::CatchAll(_, Some(_))));
///
/// // Optional catch-all
/// let seg = classify_segment("[[...slug]]");
/// assert!(matches!(seg, PatternSegmentType::OptionalCatchAll(_, _)));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum PatternSegmentType {
    /// Catch-all segment: [...slug] or [...slug:alpha]
    CatchAll(String, Option<ParameterConstraint>),
    /// Optional catch-all segment: [[...slug]] or [[...slug:alpha]] (Phase 4.1)
    OptionalCatchAll(String, Option<ParameterConstraint>),
    /// Optional parameter: [id?] or [id:int?]
    Optional(String, Option<ParameterConstraint>),
    /// Required parameter: [id] or [id:int]
    Required(String, Option<ParameterConstraint>),
    /// Static text segment
    Static(String),
}

/// Classifies a segment into a pattern type (pure function)
///
/// **Pure functional parser**: Maps string segment → PatternSegmentType
/// Uses pattern matching and functional composition for parsing.
///
/// # Parsing Rules (evaluated in order)
///
/// 1. **Optional catch-all**: `[[...name]]` or `[[...name:constraint]]`
/// 2. **Catch-all**: `[...name]` or `[...name:constraint]`
/// 3. **Optional param**: `[name?]` or `[name:constraint?]`
/// 4. **Required param**: `[name]` or `[name:constraint]`
/// 5. **Static**: Any other text
///
/// # Examples
///
/// ```
/// use rhtmx_router::route::pattern::classify_segment;
///
/// // Static
/// let seg = classify_segment("about");
///
/// // Dynamic parameters
/// let seg = classify_segment("[id]");        // Required
/// let seg = classify_segment("[id?]");       // Optional
/// let seg = classify_segment("[id:int]");    // With constraint
/// let seg = classify_segment("[id:int?]");   // Optional with constraint
///
/// // Catch-all
/// let seg = classify_segment("[...slug]");         // Required
/// let seg = classify_segment("[[...slug]]");       // Optional
/// let seg = classify_segment("[...slug:alpha]");   // With constraint
/// ```
///
/// # Performance
///
/// - O(n) where n is segment length
/// - Zero allocations for static segments (if string view available)
/// - Single allocation per parameter name
pub fn classify_segment(segment: &str) -> PatternSegmentType {
    // Check for optional catch-all: [[...name]] (double brackets)
    if segment.starts_with("[[") && segment.ends_with("]]") {
        let inner = &segment[2..segment.len() - 2]; // Strip [[ and ]]
        if let Some(param_part) = inner.strip_prefix("...") {
            let (param_name, constraint) = parse_param_with_constraint(param_part);
            return PatternSegmentType::OptionalCatchAll(param_name, constraint);
        }
    }

    match segment.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        Some(inner) => {
            // Parse catch-all: [...name] or [...name:constraint]
            if let Some(param_part) = inner.strip_prefix("...") {
                let (param_name, constraint) = parse_param_with_constraint(param_part);
                return PatternSegmentType::CatchAll(param_name, constraint);
            }

            // Parse optional: [name?] or [name:constraint?]
            if let Some(param_part) = inner.strip_suffix('?') {
                let (param_name, constraint) = parse_param_with_constraint(param_part);
                return PatternSegmentType::Optional(param_name, constraint);
            }

            // Parse required: [name] or [name:constraint]
            let (param_name, constraint) = parse_param_with_constraint(inner);
            PatternSegmentType::Required(param_name, constraint)
        }
        None => PatternSegmentType::Static(segment.to_string()),
    }
}

/// Parses parameter name and optional constraint (pure function)
///
/// **Functional parser**: Maps "name" or "name:constraint" → (name, Option<Constraint>)
///
/// Uses functional composition:
/// - `split_once(':')` → Option splitting
/// - `map()` → transform if Some
/// - `unwrap_or_else()` → default if None
///
/// # Examples
///
/// ```
/// use rhtmx_router::route::pattern::parse_param_with_constraint;
/// use rhtmx_router::ParameterConstraint;
///
/// // No constraint
/// let (name, constraint) = parse_param_with_constraint("id");
/// assert_eq!(name, "id");
/// assert_eq!(constraint, None);
///
/// // With constraint
/// let (name, constraint) = parse_param_with_constraint("id:int");
/// assert_eq!(name, "id");
/// assert_eq!(constraint, Some(ParameterConstraint::Int));
///
/// // Multiple colons (only first matters)
/// let (name, constraint) = parse_param_with_constraint("id:int:extra");
/// assert_eq!(name, "id");
/// // Constraint parsing handles "int:extra"
/// ```
///
/// # Performance
///
/// - O(n) where n is param length
/// - Zero-copy for constraint parsing (delegates to ParameterConstraint::from_str)
/// - Single allocation for parameter name
pub fn parse_param_with_constraint(param: &str) -> (String, Option<ParameterConstraint>) {
    param
        .split_once(':')
        .map(|(name, constraint_str)| {
            (
                name.to_string(),
                Some(ParameterConstraint::from_str(constraint_str)),
            )
        })
        .unwrap_or_else(|| (param.to_string(), None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_static() {
        let seg = classify_segment("about");
        assert_eq!(seg, PatternSegmentType::Static("about".to_string()));
    }

    #[test]
    fn test_classify_required() {
        let seg = classify_segment("[id]");
        assert_eq!(seg, PatternSegmentType::Required("id".to_string(), None));
    }

    #[test]
    fn test_classify_required_with_constraint() {
        let seg = classify_segment("[id:int]");
        assert_eq!(
            seg,
            PatternSegmentType::Required("id".to_string(), Some(ParameterConstraint::Int))
        );
    }

    #[test]
    fn test_classify_optional() {
        let seg = classify_segment("[id?]");
        assert_eq!(seg, PatternSegmentType::Optional("id".to_string(), None));
    }

    #[test]
    fn test_classify_optional_with_constraint() {
        let seg = classify_segment("[id:int?]");
        assert_eq!(
            seg,
            PatternSegmentType::Optional("id".to_string(), Some(ParameterConstraint::Int))
        );
    }

    #[test]
    fn test_classify_catch_all() {
        let seg = classify_segment("[...slug]");
        assert_eq!(
            seg,
            PatternSegmentType::CatchAll("slug".to_string(), None)
        );
    }

    #[test]
    fn test_classify_catch_all_with_constraint() {
        let seg = classify_segment("[...slug:alpha]");
        assert_eq!(
            seg,
            PatternSegmentType::CatchAll("slug".to_string(), Some(ParameterConstraint::Alpha))
        );
    }

    #[test]
    fn test_classify_optional_catch_all() {
        let seg = classify_segment("[[...slug]]");
        assert_eq!(
            seg,
            PatternSegmentType::OptionalCatchAll("slug".to_string(), None)
        );
    }

    #[test]
    fn test_classify_optional_catch_all_with_constraint() {
        let seg = classify_segment("[[...slug:alphanum]]");
        assert_eq!(
            seg,
            PatternSegmentType::OptionalCatchAll(
                "slug".to_string(),
                Some(ParameterConstraint::AlphaNum)
            )
        );
    }

    #[test]
    fn test_parse_param_no_constraint() {
        let (name, constraint) = parse_param_with_constraint("id");
        assert_eq!(name, "id");
        assert_eq!(constraint, None);
    }

    #[test]
    fn test_parse_param_with_constraint() {
        let (name, constraint) = parse_param_with_constraint("id:int");
        assert_eq!(name, "id");
        assert_eq!(constraint, Some(ParameterConstraint::Int));
    }

    #[test]
    fn test_parse_param_uuid_constraint() {
        let (name, constraint) = parse_param_with_constraint("user:uuid");
        assert_eq!(name, "user");
        assert_eq!(constraint, Some(ParameterConstraint::Uuid));
    }
}
