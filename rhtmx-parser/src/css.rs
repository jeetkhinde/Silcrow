// File: src/parser/css.rs
// Purpose: Parse and scope CSS from RHTML templates

use regex::Regex;

/// Represents extracted and scoped CSS
#[derive(Debug, Clone)]
pub struct ScopedCss {
    pub scope_name: String,
    pub original_css: String,
    pub scoped_css: String,
}

/// CSS Parser for RHTML templates
pub struct CssParser;

impl CssParser {
    /// Extract CSS block from RHTML content
    /// Format: css <name> { ... }
    pub fn extract_css(content: &str) -> Option<(String, String)> {
        // Match: css ComponentName { ... }
        let re = Regex::new(r"css\s+(\w+)\s*\{").ok()?;

        let caps = re.captures(content)?;
        let scope_name = caps.get(1)?.as_str().to_string();
        let css_start = caps.get(0)?.end();

        // Find the matching closing brace
        let css_content = Self::extract_css_content(&content[css_start..])?;

        Some((scope_name, css_content))
    }

    /// Extract CSS content between braces with proper nesting handling
    fn extract_css_content(content: &str) -> Option<String> {
        let mut depth = 1;
        let mut end_pos = None;

        for (i, ch) in content.chars().enumerate() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    end_pos = Some(i);
                    break;
                }
            }
        }

        end_pos.map(|end| content[..end].trim().to_string())
    }

    /// Scope CSS by adding data attribute selectors
    /// Example: .card => [data-rhtml="Button"] .card
    pub fn scope_css(scope_name: &str, css: &str) -> String {
        let scope_attr = format!("[data-rhtml=\"{}\"]", scope_name);
        let mut scoped = String::new();

        // Split CSS into rules
        let rules = Self::parse_css_rules(css);

        for rule in rules {
            if let Some((selectors, declarations)) = Self::split_rule(&rule) {
                // Scope each selector
                let scoped_selectors = selectors
                    .split(',')
                    .map(|sel| Self::scope_selector(&scope_attr, sel.trim()))
                    .collect::<Vec<_>>()
                    .join(", ");

                scoped.push_str(&format!("{} {{\n{}\n}}\n\n", scoped_selectors, declarations));
            }
        }

        scoped.trim().to_string()
    }

    /// Parse CSS into individual rules
    fn parse_css_rules(css: &str) -> Vec<String> {
        let mut rules = Vec::new();
        let mut current_rule = String::new();
        let mut depth = 0;

        for ch in css.chars() {
            current_rule.push(ch);

            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 && !current_rule.trim().is_empty() {
                    rules.push(current_rule.trim().to_string());
                    current_rule.clear();
                }
            }
        }

        rules
    }

    /// Split a CSS rule into selectors and declarations
    fn split_rule(rule: &str) -> Option<(String, String)> {
        let open_brace = rule.find('{')?;
        let close_brace = rule.rfind('}')?;

        let selectors = rule[..open_brace].trim().to_string();
        let declarations = rule[open_brace + 1..close_brace].trim().to_string();

        Some((selectors, declarations))
    }

    /// Scope a single CSS selector
    fn scope_selector(scope_attr: &str, selector: &str) -> String {
        let selector = selector.trim();

        // Handle special cases
        if selector.is_empty() {
            return scope_attr.to_string();
        }

        // If selector starts with :, it's a pseudo-class on the component itself
        if selector.starts_with(':') {
            return format!("{}{}", scope_attr, selector);
        }

        // If selector contains &, replace it with the scope attribute
        if selector.contains('&') {
            return selector.replace('&', scope_attr);
        }

        // For descendant selectors, add scope attribute at the beginning
        format!("{} {}", scope_attr, selector)
    }

    /// Remove CSS blocks from RHTML content
    pub fn remove_css_blocks(content: &str) -> String {
        // Use a simple approach: find css blocks and remove them
        let mut result = content.to_string();

        // Keep removing css blocks until none are found
        loop {
            if let Some((scope_name, _)) = Self::extract_css(&result) {
                // Find and remove this css block
                let pattern = format!(r"css\s+{}\s*\{{", regex::escape(&scope_name));
                if let Ok(re) = Regex::new(&pattern) {
                    if let Some(m) = re.find(&result) {
                        let start = m.start();
                        let content_after = &result[m.end()..];
                        if let Some(css_content) = Self::extract_css_content(content_after) {
                            let end = m.end() + css_content.len() + 1; // +1 for closing brace
                            result = format!("{}{}", &result[..start], &result[end..]);
                            continue;
                        }
                    }
                }
            }
            break;
        }

        result
    }

    /// Process RHTML content and return (content without CSS, scoped CSS, partials)
    pub fn process_template(content: &str) -> (String, Option<ScopedCss>, Vec<String>) {
        // First, process function components to standard syntax
        let processed = crate::function_component::FunctionComponentParser::process_content(content);
        let content = processed.content;
        let partials = processed.partials;

        if let Some((scope_name, css)) = Self::extract_css(&content) {
            let scoped_css = Self::scope_css(&scope_name, &css);
            let content_without_css = Self::remove_css_blocks(&content);

            (
                content_without_css,
                Some(ScopedCss {
                    scope_name: scope_name.clone(),
                    original_css: css,
                    scoped_css,
                }),
                partials,
            )
        } else {
            (content.to_string(), None, partials)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_css() {
        let content = r#"
            cmp Button {
                <button>Click</button>
            }

            css Button {
                .btn {
                    color: blue;
                }
            }
        "#;

        let result = CssParser::extract_css(content);
        assert!(result.is_some());

        let (scope_name, css) = result.unwrap();
        assert_eq!(scope_name, "Button");
        assert!(css.contains(".btn"));
    }

    #[test]
    fn test_scope_css() {
        let css = r#"
            .btn {
                color: blue;
                padding: 10px;
            }
            .btn:hover {
                color: red;
            }
        "#;

        let scoped = CssParser::scope_css("Button", css);
        assert!(scoped.contains("[data-rhtml=\"Button\"]"));
        assert!(scoped.contains(".btn"));
    }

    #[test]
    fn test_scope_selector() {
        let scope = "[data-rhtml=\"Button\"]";

        assert_eq!(
            CssParser::scope_selector(scope, ".btn"),
            "[data-rhtml=\"Button\"] .btn"
        );

        assert_eq!(
            CssParser::scope_selector(scope, ":hover"),
            "[data-rhtml=\"Button\"]:hover"
        );
    }

    #[test]
    fn test_process_template() {
        let content = r#"
            cmp Button {
                <button class="btn">Click</button>
            }

            css Button {
                .btn {
                    color: blue;
                }
            }
        "#;

        let (content_without_css, scoped_css, partials) = CssParser::process_template(content);

        assert!(!content_without_css.contains("css Button"));
        assert!(scoped_css.is_some());
        assert!(partials.is_empty()); // No @partial attribute

        let scoped = scoped_css.unwrap();
        assert_eq!(scoped.scope_name, "Button");
        assert!(scoped.scoped_css.contains("[data-rhtml=\"Button\"]"));
    }
}
