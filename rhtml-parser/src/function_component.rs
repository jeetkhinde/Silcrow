// File: rhtml-parser/src/function_component.rs
// Purpose: Parse #[webpage] attribute syntax for pages

use regex::Regex;

/// Result of processing webpage content
#[derive(Debug, Clone)]
pub struct ProcessedContent {
    pub content: String,
    pub partials: Vec<String>, // Names of components marked as @partial (currently unused)
}

/// Parser for #[webpage] syntax
pub struct FunctionComponentParser;

impl FunctionComponentParser {
    /// Check if content has #[webpage] attribute
    pub fn has_webpage_attribute(content: &str) -> bool {
        content.contains("#[webpage]")
    }

    /// Extract Rust functions with #[webpage] attribute
    /// Parses: #[webpage] pub fn name(props: Type) { <html> }
    pub fn extract_webpage_function(content: &str) -> Option<String> {
        // Pattern: #[webpage] followed by function definition
        let re = Regex::new(r"#\[webpage\]\s+(?:pub\s+)?fn\s+\w+\s*\([^)]*\)\s*\{").unwrap();

        if let Some(mat) = re.find(content) {
            let body_start = mat.end();

            // Extract function body
            if let Some(body) = Self::extract_braced_content(&content[body_start..]) {
                return Some(body.trim().to_string());
            }
        }

        None
    }

    /// Extract content within braces with proper nesting
    fn extract_braced_content(content: &str) -> Option<String> {
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

    /// Remove struct definitions from content
    pub fn remove_structs(content: &str) -> String {
        let mut result = content.to_string();

        loop {
            let re = Regex::new(r"struct\s+\w+\s*\{").unwrap();

            if let Some(mat) = re.find(&result) {
                let start = mat.start();
                let body_start = mat.end();

                if let Some(body) = Self::extract_braced_content(&result[body_start..]) {
                    let end = body_start + body.len() + 1; // +1 for closing brace
                    result = format!("{}{}", &result[..start], &result[end..]);
                    continue;
                }
            }

            break;
        }

        result
    }

    /// Process content: convert #[webpage] functions to WebPage { body } format
    /// Returns processed content
    pub fn process_content(content: &str) -> ProcessedContent {
        let mut result = content.to_string();

        // If no #[webpage] attribute, return as-is
        if !Self::has_webpage_attribute(&result) {
            return ProcessedContent {
                content: result,
                partials: Vec::new(),
            };
        }

        // Find and replace the entire #[webpage] function with WebPage { body }
        let re = Regex::new(r"#\[webpage\]\s+(?:pub\s+)?fn\s+\w+\s*\([^)]*\)\s*\{").unwrap();

        if let Some(mat) = re.find(&result) {
            let start = mat.start();
            let body_start = mat.end();

            if let Some(body_content) = Self::extract_braced_content(&result[body_start..]) {
                let end = body_start + body_content.len() + 1;

                // Replace with WebPage { body } format
                let replacement = format!("WebPage {{\n{}\n}}", body_content.trim());
                result = format!("{}{}{}", &result[..start], replacement, &result[end..]);
            }
        }

        // Remove struct definitions (we don't need them at runtime)
        result = Self::remove_structs(&result);

        ProcessedContent {
            content: result,
            partials: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webpage_attribute_detection() {
        let content = r#"
#[webpage]
pub fn users(props: UsersProps) {
    <div>Users</div>
}
        "#;

        assert!(FunctionComponentParser::has_webpage_attribute(content));
    }

    #[test]
    fn test_extract_webpage_function() {
        let content = r#"
#[webpage]
pub fn users(props: UsersProps) {
    <div class="container">
        <h1>Users</h1>
        <div r-for="user in props.data">
            <user_card user={user} />
        </div>
    </div>
}
        "#;

        let body = FunctionComponentParser::extract_webpage_function(content);
        assert!(body.is_some());
        assert!(body.unwrap().contains("<h1>Users</h1>"));
    }

    #[test]
    fn test_process_webpage_attribute() {
        let content = r#"
slots {
    title: "Users",
}

#[webpage]
pub fn users(props: UsersProps) {
    <div class="container">
        <h1>Users</h1>
        <div r-for="user in props.data">
            <user_card user={user} />
        </div>
    </div>
}
        "#;

        let processed = FunctionComponentParser::process_content(content);

        // Should contain WebPage {
        assert!(
            processed.content.contains("WebPage {"),
            "Content does not contain 'WebPage {{': {}",
            processed.content
        );

        // Should preserve HTML
        assert!(processed.content.contains("<h1>Users</h1>"));
        assert!(processed.content.contains("r-for="));

        // Should not contain #[webpage] anymore
        assert!(!processed.content.contains("#[webpage]"));
        assert!(!processed.content.contains("pub fn users"));
    }

    #[test]
    fn test_webpage_attribute_without_pub() {
        let content = r#"
#[webpage]
fn home(props: PageProps) {
    <div>Home</div>
}
        "#;

        let body = FunctionComponentParser::extract_webpage_function(content);
        assert!(body.is_some());
    }

    #[test]
    fn test_remove_structs() {
        let content = r#"
struct UsersProps {
    data: Vec<User>,
}

Some other content
        "#;

        let result = FunctionComponentParser::remove_structs(content);
        assert!(!result.contains("struct UsersProps"));
        assert!(result.contains("Some other content"));
    }

    #[test]
    fn test_process_content_without_webpage() {
        let content = r#"
<div>Just HTML content</div>
        "#;

        let processed = FunctionComponentParser::process_content(content);
        assert!(processed.content.contains("<div>Just HTML content</div>"));
    }
}
