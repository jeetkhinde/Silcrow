// File: src/renderer.rs
// Purpose: Simple template renderer with variable interpolation (no r-directives)

use crate::template_loader::TemplateLoader;
use crate::value::Value;
use anyhow::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use once_cell::sync::Lazy;

/// Result of a rendering operation
#[derive(Debug, Clone)]
pub struct RenderResult {
    pub html: String,
    pub collected_css: HashSet<String>,
}

impl RenderResult {
    pub fn new(html: String) -> Self {
        Self {
            html,
            collected_css: HashSet::new(),
        }
    }

    pub fn with_css(mut self, css: String) -> Self {
        self.collected_css.insert(css);
        self
    }

    pub fn merge_css(mut self, other: &RenderResult) -> Self {
        self.collected_css.extend(other.collected_css.clone());
        self
    }
}

/// Rendering context (immutable)
#[derive(Clone)]
pub struct RenderContext {
    variables: HashMap<String, Value>,
    template_loader: Option<Arc<TemplateLoader>>,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            template_loader: None,
        }
    }

    pub fn with_loader(template_loader: Arc<TemplateLoader>) -> Self {
        Self {
            variables: HashMap::new(),
            template_loader: Some(template_loader),
        }
    }

    pub fn with_var(mut self, name: impl Into<String>, value: Value) -> Self {
        self.variables.insert(name.into(), value);
        self
    }

    pub fn with_vars(mut self, vars: HashMap<String, Value>) -> Self {
        self.variables.extend(vars);
        self
    }

    pub fn with_context_vars(mut self, other: &RenderContext) -> Self {
        self.variables.extend(other.variables.clone());
        self
    }

    fn get_var(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Mutable renderer (for backwards compatibility)
pub struct Renderer {
    context: RenderContext,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            context: RenderContext::new(),
        }
    }

    pub fn with_loader(template_loader: Arc<TemplateLoader>) -> Self {
        Self {
            context: RenderContext::with_loader(template_loader),
        }
    }

    pub fn from_context(context: RenderContext) -> Self {
        Self { context }
    }

    pub fn with_var(mut self, name: impl Into<String>, value: Value) -> Self {
        self.context = self.context.with_var(name, value);
        self
    }

    pub fn set_var(&mut self, name: impl Into<String>, value: Value) {
        self.context.variables.insert(name.into(), value);
    }

    /// Placeholder for scoped CSS collection (for backwards compatibility)
    pub fn collect_template_css(&mut self, _scoped_css: &Option<()>) {
        // No-op: CSS is handled by Maud now
    }

    /// Render template content with variable interpolation
    pub fn render(&self, template_content: &str) -> Result<RenderResult> {
        let html = self.interpolate_variables(template_content);
        Ok(RenderResult::new(html))
    }

    /// Render as partial (just calls render)
    pub fn render_partial(&self, content: &str) -> Result<RenderResult> {
        self.render(content)
    }

    /// Render page with layout
    pub fn render_with_layout(
        &self,
        layout_content: &str,
        page_content: &str,
    ) -> Result<RenderResult> {
        // Render the page content first
        let page_result = self.render(page_content)?;

        // Replace {slots.content} in layout with rendered page
        let layout_html = layout_content.replace("{slots.content}", &page_result.html);

        // Interpolate variables in the combined HTML
        let final_html = self.interpolate_variables(&layout_html);

        Ok(RenderResult::new(final_html))
    }

    /// Interpolate {variable} placeholders with actual values
    fn interpolate_variables(&self, content: &str) -> String {
        static VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_\.]*)\}").unwrap()
        });

        VAR_REGEX.replace_all(content, |caps: &regex::Captures| {
            let var_name = &caps[1];

            // Handle dotted paths (e.g., user.name, slots.content)
            if var_name.contains('.') {
                self.get_nested_value(var_name)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| format!("{{{}}}", var_name))
            } else {
                self.context.get_var(var_name)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| format!("{{{}}}", var_name))
            }
        }).to_string()
    }

    /// Get nested value using dot notation (e.g., "user.name")
    fn get_nested_value(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let mut current = self.context.get_var(parts[0])?;

        for part in &parts[1..] {
            match current {
                Value::Object(map) => {
                    current = map.get(*part)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_interpolation() {
        let renderer = Renderer::new()
            .with_var("name", Value::String("Alice".to_string()))
            .with_var("age", Value::Number(30.0));

        let result = renderer.render("<p>Hello, {name}! Age: {age}</p>").unwrap();
        assert_eq!(result.html, "<p>Hello, Alice! Age: 30</p>");
    }

    #[test]
    fn test_nested_value() {
        let mut user_map = HashMap::new();
        user_map.insert("name".to_string(), Value::String("Bob".to_string()));
        user_map.insert("email".to_string(), Value::String("bob@example.com".to_string()));

        let renderer = Renderer::new()
            .with_var("user", Value::Object(user_map));

        let result = renderer.render("<p>{user.name} - {user.email}</p>").unwrap();
        assert_eq!(result.html, "<p>Bob - bob@example.com</p>");
    }

    #[test]
    fn test_layout_rendering() {
        let renderer = Renderer::new()
            .with_var("title", Value::String("Test Page".to_string()));

        let layout = "<html><head><title>{title}</title></head><body>{slots.content}</body></html>";
        let page = "<h1>Welcome</h1>";

        let result = renderer.render_with_layout(layout, page).unwrap();
        assert!(result.html.contains("<title>Test Page</title>"));
        assert!(result.html.contains("<h1>Welcome</h1>"));
    }

    #[test]
    fn test_missing_variable() {
        let renderer = Renderer::new();
        let result = renderer.render("<p>{missing}</p>").unwrap();
        assert_eq!(result.html, "<p>{missing}</p>"); // Keeps placeholder
    }
}
