/// Pattern parsing and priority calculation for routes
///
/// Pure functional parsers that transform file paths into route patterns.
/// All functions are **pure**: same input → same output, no side effects.

use std::collections::HashMap;

use crate::{ParameterConstraint, PatternSegmentType};
use super::pattern::classify_segment;

/// Internal state accumulator for functional fold-based parsing
///
/// Pure functional design: all mutations are local to the fold accumulator.
/// Each builder method returns a new/modified Self, enabling functional chaining.
#[derive(Default)]
struct ParseState {
    pattern: String,
    params: Vec<String>,
    optional_params: Vec<String>,
    dynamic_count: usize,
    has_catch_all: bool,
    param_constraints: HashMap<String, ParameterConstraint>,
}

impl ParseState {
    /// Adds a static segment to the pattern
    fn with_static_segment(mut self, segment: String) -> Self {
        self.pattern.push('/');
        self.pattern.push_str(&segment);
        self
    }

    /// Adds a required parameter segment
    fn with_required_param(mut self, param_name: String, constraint: Option<ParameterConstraint>) -> Self {
        self.pattern.push_str("/:");
        self.pattern.push_str(&param_name);
        self.params.push(param_name.clone());

        if let Some(c) = constraint {
            self.param_constraints.insert(param_name, c);
        }

        self.dynamic_count += 1;
        self
    }

    /// Adds an optional parameter segment
    fn with_optional_param(mut self, param_name: String, constraint: Option<ParameterConstraint>) -> Self {
        self.pattern.push_str("/:");
        self.pattern.push_str(&param_name);
        self.pattern.push('?');
        self.params.push(param_name.clone());
        self.optional_params.push(param_name.clone());

        if let Some(c) = constraint {
            self.param_constraints.insert(param_name, c);
        }

        self.dynamic_count += 1;
        self
    }

    /// Adds a catch-all parameter segment
    fn with_catch_all(mut self, param_name: String, constraint: Option<ParameterConstraint>) -> Self {
        self.pattern.push_str("/*");
        self.pattern.push_str(&param_name);
        self.params.push(param_name.clone());

        if let Some(c) = constraint {
            self.param_constraints.insert(param_name, c);
        }

        self.has_catch_all = true;
        self.dynamic_count += 100;
        self
    }

    /// Adds an optional catch-all parameter segment
    fn with_optional_catch_all(mut self, param_name: String, constraint: Option<ParameterConstraint>) -> Self {
        self.pattern.push_str("/*");
        self.pattern.push_str(&param_name);
        self.pattern.push('?');
        self.params.push(param_name.clone());
        self.optional_params.push(param_name.clone());

        if let Some(c) = constraint {
            self.param_constraints.insert(param_name, c);
        }

        self.has_catch_all = true;
        self.dynamic_count += 99;
        self
    }

    /// Finalizes the pattern, handling empty pattern case
    fn finalize(mut self) -> Self {
        if self.pattern.is_empty() {
            self.pattern = "/".to_string();
        }
        self
    }

    /// Deconstructs state into return tuple
    fn into_tuple(self) -> (
        String,
        Vec<String>,
        Vec<String>,
        usize,
        bool,
        HashMap<String, ParameterConstraint>,
    ) {
        (
            self.pattern,
            self.params,
            self.optional_params,
            self.dynamic_count,
            self.has_catch_all,
            self.param_constraints,
        )
    }
}

/// Checks if a segment should be skipped during parsing
///
/// Pure predicate function: segment -> bool
fn should_skip_segment(segment: &str) -> bool {
    segment.is_empty()
        || segment == "_layout"
        || segment.starts_with("_layout.") // Skip named layouts like _layout.admin
        || segment == "_error"
        || segment == "_nolayout" // Skip nolayout markers
        || segment == "loading" // Phase 4.3
        || segment == "_template" // Phase 4.4
        || segment == "not-found" // Phase 4.5
        || segment == "index"
        || matches!(segment, "(.)" | "(..)" | "(...)" | "(....)") // Intercepting route markers
        || (segment.starts_with('(') && segment.ends_with(')')) // Route groups
        || segment.starts_with('@') // Parallel route slots
}

/// Processes a single segment and updates the parse state
///
/// Pure function: (state, segment) -> new state
fn process_segment(state: ParseState, segment: &str) -> ParseState {
    if should_skip_segment(segment) {
        return state;
    }

    match classify_segment(segment) {
        PatternSegmentType::CatchAll(param_name, constraint) => {
            state.with_catch_all(param_name, constraint)
        }
        PatternSegmentType::OptionalCatchAll(param_name, constraint) => {
            state.with_optional_catch_all(param_name, constraint)
        }
        PatternSegmentType::Optional(param_name, constraint) => {
            state.with_optional_param(param_name, constraint)
        }
        PatternSegmentType::Required(param_name, constraint) => {
            state.with_required_param(param_name, constraint)
        }
        PatternSegmentType::Static(seg) => {
            state.with_static_segment(seg)
        }
    }
}

/// Parses a file path pattern into route components (pure function)
///
/// **Pure functional parser**: Maps file path → (pattern, params, constraints, ...)
///
/// Uses functional composition:
/// - `split('/') → iter` - break path into segments
/// - `fold` - accumulate state through ParseState accumulator
/// - `process_segment` - pure transformation for each segment
/// - `finalize` - handle edge cases (empty pattern)
///
/// # Returns
///
/// Tuple of:
/// 1. `pattern` - URL pattern like "/users/:id"
/// 2. `params` - Parameter names ["id"]
/// 3. `optional_params` - Optional parameter names
/// 4. `dynamic_count` - Priority scoring for dynamic segments
/// 5. `has_catch_all` - Whether route has catch-all parameter
/// 6. `param_constraints` - Parameter validation constraints
///
/// # Examples
///
/// ```
/// use rhtmx_router::route::parser::parse_pattern;
///
/// // Static route
/// let (pattern, params, _, _, _, _) = parse_pattern("pages/about.rhtml");
/// assert_eq!(pattern, "/about");
/// assert_eq!(params.len(), 0);
///
/// // Dynamic route
/// let (pattern, params, _, _, _, _) = parse_pattern("pages/users/[id].rhtml");
/// assert_eq!(pattern, "/users/:id");
/// assert_eq!(params, vec!["id".to_string()]);
///
/// // Catch-all route
/// let (pattern, params, _, _, has_catch_all, _) = parse_pattern("pages/docs/[...slug].rhtml");
/// assert_eq!(pattern, "/docs/*slug");
/// assert_eq!(has_catch_all, true);
/// ```
///
/// # Special Segments (Skipped)
///
/// - `_layout`, `_layout.*` - Layout files
/// - `_error` - Error pages
/// - `_nolayout` - No-layout markers
/// - `loading` - Loading UI
/// - `_template` - Template files
/// - `not-found` - 404 pages
/// - `index` - Index pages
/// - `(.)`, `(..)`, `(...)`, `(....)` - Intercepting route markers
/// - `(folder)` - Route groups
/// - `@slot` - Parallel route slots
///
/// # Performance
///
/// - O(n) where n is number of segments
/// - Functional pipeline with fold combinator
/// - Single pass, no backtracking
pub fn parse_pattern(
    path: &str,
) -> (
    String,
    Vec<String>,
    Vec<String>,
    usize,
    bool,
    HashMap<String, ParameterConstraint>,
) {
    path.split('/')
        .fold(ParseState::default(), process_segment)
        .finalize()
        .into_tuple()
}

/// Calculates route priority for matching order (pure function)
///
/// **Pure function**: Maps (has_catch_all, dynamic_count, depth, optional_params) → priority
///
/// Lower number = higher priority (matched first).
///
/// # Priority Order
///
/// 1. **Static routes** → 0
/// 2. **Dynamic routes** → 1-999
/// 3. **Required catch-all** → 1000+
/// 4. **Optional catch-all** → 2000+
///
/// # Examples
///
/// ```
/// use rhtmx_router::route::parser::calculate_priority;
///
/// // Static route: highest priority
/// assert_eq!(calculate_priority(false, 0, 2, &[]), 0);
///
/// // Dynamic route with one param
/// assert_eq!(calculate_priority(false, 1, 2, &[]), 4); // 1 + 2 + 1
///
/// // Required catch-all
/// assert_eq!(calculate_priority(true, 100, 2, &[]), 1002); // 1000 + 2
///
/// // Optional catch-all
/// assert_eq!(calculate_priority(true, 99, 2, &["slug".to_string()]), 2002); // 2000 + 2
/// ```
///
/// # Performance
///
/// - O(n) where n is number of optional params (for checking if catch-all is optional)
/// - Functional predicate: `any(|p| p.len() > 0)`
/// - Constant time for non-catch-all routes
pub fn calculate_priority(
    has_catch_all: bool,
    dynamic_count: usize,
    depth: usize,
    optional_params: &[String],
) -> usize {
    if has_catch_all {
        // Check if catch-all is optional (present in optional_params)
        // Functional predicate: any
        if optional_params.iter().any(|p| !p.is_empty()) {
            // Optional catch-all: lower priority (higher number)
            2000 + depth
        } else {
            // Required catch-all
            1000 + depth
        }
    } else if dynamic_count > 0 {
        let optional_bonus = if optional_params.is_empty() { 1 } else { 0 };
        dynamic_count + depth + optional_bonus
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pattern_static() {
        let (pattern, params, opt, dyn_count, has_catch, _) = parse_pattern("about");
        assert_eq!(pattern, "/about");
        assert_eq!(params.len(), 0);
        assert_eq!(opt.len(), 0);
        assert_eq!(dyn_count, 0);
        assert_eq!(has_catch, false);
    }

    #[test]
    fn test_parse_pattern_dynamic() {
        let (pattern, params, opt, dyn_count, has_catch, _) = parse_pattern("users/[id]");
        assert_eq!(pattern, "/users/:id");
        assert_eq!(params, vec!["id".to_string()]);
        assert_eq!(opt.len(), 0);
        assert_eq!(dyn_count, 1);
        assert_eq!(has_catch, false);
    }

    #[test]
    fn test_parse_pattern_optional_param() {
        let (pattern, params, opt, _, _, _) = parse_pattern("posts/[id?]");
        assert_eq!(pattern, "/posts/:id?");
        assert_eq!(params, vec!["id".to_string()]);
        assert_eq!(opt, vec!["id".to_string()]);
    }

    #[test]
    fn test_parse_pattern_catch_all() {
        let (pattern, params, _, _, has_catch, _) = parse_pattern("docs/[...slug]");
        assert_eq!(pattern, "/docs/*slug");
        assert_eq!(params, vec!["slug".to_string()]);
        assert_eq!(has_catch, true);
    }

    #[test]
    fn test_parse_pattern_optional_catch_all() {
        let (pattern, params, opt, _, has_catch, _) = parse_pattern("docs/[[...slug]]");
        assert_eq!(pattern, "/docs/*slug?");
        assert_eq!(params, vec!["slug".to_string()]);
        assert_eq!(opt, vec!["slug".to_string()]);
        assert_eq!(has_catch, true);
    }

    #[test]
    fn test_parse_pattern_skips_special_files() {
        let (pattern, params, _, _, _, _) = parse_pattern("users/_layout/[id]/index");
        assert_eq!(pattern, "/users/:id");
        assert_eq!(params, vec!["id".to_string()]);
    }

    #[test]
    fn test_parse_pattern_skips_route_groups() {
        let (pattern, _, _, _, _, _) = parse_pattern("(admin)/users");
        assert_eq!(pattern, "/users");
    }

    #[test]
    fn test_parse_pattern_skips_parallel_slots() {
        let (pattern, _, _, _, _, _) = parse_pattern("dashboard/@analytics/page");
        assert_eq!(pattern, "/dashboard/page");
    }

    #[test]
    fn test_parse_pattern_skips_intercept_markers() {
        let (pattern, _, _, _, _, _) = parse_pattern("feed/(.)/photo/[id]");
        assert_eq!(pattern, "/feed/photo/:id");
    }

    #[test]
    fn test_calculate_priority_static() {
        assert_eq!(calculate_priority(false, 0, 2, &[]), 0);
    }

    #[test]
    fn test_calculate_priority_dynamic() {
        assert_eq!(calculate_priority(false, 1, 2, &[]), 4); // 1 + 2 + 1
    }

    #[test]
    fn test_calculate_priority_required_catch_all() {
        assert_eq!(calculate_priority(true, 100, 2, &[]), 1002);
    }

    #[test]
    fn test_calculate_priority_optional_catch_all() {
        assert_eq!(
            calculate_priority(true, 99, 2, &["slug".to_string()]),
            2002
        );
    }
}
