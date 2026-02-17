use crate::template_loader::TemplateLoader;
use crate::value::Value;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;

/// Result of a rendering operation
#[derive(Debug, Clone)]
pub struct RenderResult {
    pub html: String,
}

impl RenderResult {
    pub fn new(html: String) -> Self {
        Self { html }
    }
}

/// Template renderer with variable interpolation
pub struct Renderer {
    variables: HashMap<String, Value>,
    _loader: Option<Arc<TemplateLoader>>,
}

impl Renderer {
    pub fn new() -> Self {
        Self { variables: HashMap::new(), _loader: None }
    }

    pub fn with_loader(loader: Arc<TemplateLoader>) -> Self {
        Self { variables: HashMap::new(), _loader: Some(loader) }
    }

    pub fn set_var(&mut self, name: impl Into<String>, value: Value) {
        self.variables.insert(name.into(), value);
    }

    pub fn render(&self, content: &str) -> Result<RenderResult> {
        Ok(RenderResult::new(self.interpolate(content)))
    }

    pub fn render_partial(&self, content: &str) -> Result<RenderResult> {
        self.render(content)
    }

    pub fn render_with_layout(&self, layout: &str, page: &str) -> Result<RenderResult> {
        let page_result = self.render(page)?;
        let combined = layout.replace("{slots.content}", &page_result.html);
        Ok(RenderResult::new(self.interpolate(&combined)))
    }

    fn interpolate(&self, content: &str) -> String {
        static VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_\.]*)\}").unwrap()
        });

        VAR_REGEX.replace_all(content, |caps: &regex::Captures| {
            let name = &caps[1];
            if name.contains('.') {
                self.get_nested(name)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| format!("{{{}}}", name))
            } else {
                self.variables.get(name)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| format!("{{{}}}", name))
            }
        }).to_string()
    }

    fn get_nested(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = self.variables.get(parts[0])?;
        for part in &parts[1..] {
            match current {
                Value::Object(map) => current = map.get(*part)?,
                _ => return None,
            }
        }
        Some(current)
    }
}

impl Default for Renderer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_interpolation() {
        let mut renderer = Renderer::new();
        renderer.set_var("name", Value::String("Alice".to_string()));
        renderer.set_var("age", Value::Number(30.0));
        let result = renderer.render("<p>Hello, {name}! Age: {age}</p>").unwrap();
        assert_eq!(result.html, "<p>Hello, Alice! Age: 30</p>");
    }

    #[test]
    fn test_nested_value() {
        let mut user_map = HashMap::new();
        user_map.insert("name".to_string(), Value::String("Bob".to_string()));
        let mut renderer = Renderer::new();
        renderer.set_var("user", Value::Object(user_map));
        let result = renderer.render("<p>{user.name}</p>").unwrap();
        assert_eq!(result.html, "<p>Bob</p>");
    }

    #[test]
    fn test_layout_rendering() {
        let mut renderer = Renderer::new();
        renderer.set_var("title", Value::String("Test".to_string()));
        let layout = "<html><title>{title}</title><body>{slots.content}</body></html>";
        let page = "<h1>Welcome</h1>";
        let result = renderer.render_with_layout(layout, page).unwrap();
        assert!(result.html.contains("<title>Test</title>"));
        assert!(result.html.contains("<h1>Welcome</h1>"));
    }

    #[test]
    fn test_missing_variable() {
        let renderer = Renderer::new();
        let result = renderer.render("<p>{missing}</p>").unwrap();
        assert_eq!(result.html, "<p>{missing}</p>");
    }
}
