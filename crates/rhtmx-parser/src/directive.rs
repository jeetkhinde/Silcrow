// File: src/parser/directive.rs
// Purpose: Parse and identify RHTML directives (r-if, r-else, etc.)

use regex::Regex;

/// Represents a parsed directive
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    If(String),     // r-if="condition"
    ElseIf(String), // r-else-if="condition"
    Else,           // r-else
    For {           // r-for="item in items" or r-for="(index, item) in items"
        item_var: String,
        index_var: Option<String>,
        collection: String,
    },
    Match(String),  // r-match="variable"
    When(String),   // r-when="value"
    Default,        // r-default
}

/// Parser for RHTML directives
pub struct DirectiveParser;

impl DirectiveParser {
    /// Check if an HTML tag has an r-if directive
    pub fn has_if_directive(tag: &str) -> bool {
        tag.contains("r-if=")
    }

    /// Check if an HTML tag has an r-else-if directive
    pub fn has_else_if_directive(tag: &str) -> bool {
        tag.contains("r-else-if=")
    }

    /// Check if an HTML tag has an r-else directive
    pub fn has_else_directive(tag: &str) -> bool {
        tag.contains("r-else") && !tag.contains("r-else-if")
    }

    /// Check if an HTML tag has an r-for directive
    pub fn has_for_directive(tag: &str) -> bool {
        tag.contains("r-for=")
    }

    /// Check if an HTML tag has an r-match directive
    pub fn has_match_directive(tag: &str) -> bool {
        tag.contains("r-match=")
    }

    /// Check if an HTML tag has an r-when directive
    pub fn has_when_directive(tag: &str) -> bool {
        tag.contains("r-when=")
    }

    /// Check if an HTML tag has an r-default directive
    pub fn has_default_directive(tag: &str) -> bool {
        tag.contains("r-default") && !tag.contains("r-default=")
    }

    /// Extract r-if condition from a tag
    pub fn extract_if_condition(tag: &str) -> Option<String> {
        Self::extract_directive_value(tag, "r-if")
    }

    /// Extract r-else-if condition from a tag
    pub fn extract_else_if_condition(tag: &str) -> Option<String> {
        Self::extract_directive_value(tag, "r-else-if")
    }

    /// Extract r-for loop information from a tag
    /// Supports: "item in items" or "(index, item) in items"
    pub fn extract_for_loop(tag: &str) -> Option<(String, Option<String>, String)> {
        let value = Self::extract_directive_value(tag, "r-for")?;

        // Split by " in "
        let parts: Vec<&str> = value.split(" in ").collect();
        if parts.len() != 2 {
            return None;
        }

        let left = parts[0].trim();
        let collection = parts[1].trim().to_string();

        // Check if it's "(index, item)" format
        if left.starts_with('(') && left.ends_with(')') {
            // Parse (index, item)
            let inner = &left[1..left.len() - 1];
            let vars: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

            if vars.len() == 2 {
                return Some((
                    vars[1].to_string(), // item
                    Some(vars[0].to_string()), // index
                    collection,
                ));
            }
        }

        // Simple "item in items" format
        Some((left.to_string(), None, collection))
    }

    /// Extract r-match variable from a tag
    pub fn extract_match_variable(tag: &str) -> Option<String> {
        Self::extract_directive_value(tag, "r-match")
    }

    /// Extract r-when pattern from a tag
    pub fn extract_when_pattern(tag: &str) -> Option<String> {
        Self::extract_directive_value(tag, "r-when")
    }

    /// Extract directive value using regex
    fn extract_directive_value(tag: &str, directive: &str) -> Option<String> {
        // Match: r-if="condition" or r-if='condition'
        let pattern = format!(r#"{}=["']([^"']+)["']"#, directive);
        let re = Regex::new(&pattern).ok()?;

        re.captures(tag)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Remove directive attributes from a tag
    pub fn remove_directives(tag: &str) -> String {
        let patterns = [
            r#"r-if=["'][^"']*["']"#,
            r#"r-else-if=["'][^"']*["']"#,
            r#"r-for=["'][^"']*["']"#,
            r#"r-match=["'][^"']*["']"#,
            r#"r-when=["'][^"']*["']"#,
            r#"r-else\s*"#,
            r#"r-else="#,
            r#"r-default\s*"#,
            r#"r-default="#,
        ];

        patterns
            .iter()
            .fold(tag.to_string(), |acc, pattern| {
                Regex::new(pattern)
                    .ok()
                    .map(|re| re.replace_all(&acc, "").to_string())
                    .unwrap_or(acc)
            })
            .trim()
            .replace("  ", " ")
    }

    /// Parse all directives from a tag
    pub fn parse_directives(tag: &str) -> Vec<Directive> {
        [
            // r-if directive
            Self::has_if_directive(tag)
                .then(|| Self::extract_if_condition(tag))
                .flatten()
                .map(Directive::If),

            // r-else-if directive
            Self::has_else_if_directive(tag)
                .then(|| Self::extract_else_if_condition(tag))
                .flatten()
                .map(Directive::ElseIf),

            // r-else directive
            Self::has_else_directive(tag).then_some(Directive::Else),

            // r-for directive
            Self::has_for_directive(tag)
                .then(|| Self::extract_for_loop(tag))
                .flatten()
                .map(|(item_var, index_var, collection)| Directive::For {
                    item_var,
                    index_var,
                    collection,
                }),

            // r-match directive
            Self::has_match_directive(tag)
                .then(|| Self::extract_match_variable(tag))
                .flatten()
                .map(Directive::Match),

            // r-when directive
            Self::has_when_directive(tag)
                .then(|| Self::extract_when_pattern(tag))
                .flatten()
                .map(Directive::When),

            // r-default directive
            Self::has_default_directive(tag).then_some(Directive::Default),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_if_condition() {
        let tag = r#"<div r-if="user.is_active" class="active">"#;
        assert_eq!(
            DirectiveParser::extract_if_condition(tag),
            Some("user.is_active".to_string())
        );
    }

    #[test]
    fn test_remove_directives() {
        let tag = r#"<div r-if="true" class="test">"#;
        let cleaned = DirectiveParser::remove_directives(tag);
        assert!(!cleaned.contains("r-if"));
        assert!(cleaned.contains("class=\"test\""));
    }

    #[test]
    fn test_extract_for_loop() {
        let tag = r#"<div r-for="item in items">"#;
        let result = DirectiveParser::extract_for_loop(tag);
        assert_eq!(
            result,
            Some(("item".to_string(), None, "items".to_string()))
        );

        let tag_with_index = r#"<div r-for="(i, item) in items">"#;
        let result_with_index = DirectiveParser::extract_for_loop(tag_with_index);
        assert_eq!(
            result_with_index,
            Some(("item".to_string(), Some("i".to_string()), "items".to_string()))
        );
    }

    #[test]
    fn test_extract_match_and_when() {
        let match_tag = r#"<div r-match="status">"#;
        assert_eq!(
            DirectiveParser::extract_match_variable(match_tag),
            Some("status".to_string())
        );

        let when_tag = r#"<div r-when="active">"#;
        assert_eq!(
            DirectiveParser::extract_when_pattern(when_tag),
            Some("active".to_string())
        );

        let default_tag = r#"<div r-default>"#;
        assert!(DirectiveParser::has_default_directive(default_tag));
    }
}
