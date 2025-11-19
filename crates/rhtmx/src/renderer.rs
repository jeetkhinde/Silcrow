// File: src/renderer.rs
// Purpose: Render rhtmx templates with directive support (Functional Programming Style)

use crate::template_loader::TemplateLoader;
use anyhow::Result;
use regex::Regex;
use rhtmx_parser::{DirectiveParser, ExpressionEvaluator, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Result of a rendering operation
///
/// # Fields
/// * `html` - The rendered HTML string
/// * `collected_css` - Set of scoped CSS styles collected during rendering
#[derive(Debug, Clone)]
pub struct RenderResult {
    pub html: String,
    pub collected_css: HashSet<String>,
}

impl RenderResult {
    /// Creates a new RenderResult with the given HTML content
    ///
    /// # Arguments
    /// * `html` - The HTML content as a String
    ///
    /// # Returns
    /// A new RenderResult instance with empty CSS collection
    pub fn new(html: String) -> Self {
        Self {
            html,
            collected_css: HashSet::new(),
        }
    }

    /// Adds a CSS string to the collected styles
    ///
    /// # Arguments
    /// * `css` - CSS string to add to the collection
    ///
    /// # Returns
    /// Self with the CSS added to collected_css
    pub fn with_css(mut self, css: String) -> Self {
        self.collected_css.insert(css);
        self
    }

    /// Merges CSS from another RenderResult into this one
    ///
    /// # Arguments
    /// * `other` - Reference to another RenderResult whose CSS to merge
    ///
    /// # Returns
    /// Self with merged CSS collections
    pub fn merge_css(mut self, other: &RenderResult) -> Self {
        self.collected_css.extend(other.collected_css.clone());
        self
    }
}

/// Immutable rendering context that flows through the rendering pipeline
///
/// # Fields
/// * `variables` - HashMap of template variables available during rendering
/// * `template_loader` - Optional template loader for loading components
#[derive(Clone)]
pub struct RenderContext {
    variables: HashMap<String, Value>,
    template_loader: Option<Arc<TemplateLoader>>,
}

impl RenderContext {
    /// Creates a new empty RenderContext
    ///
    /// # Returns
    /// A new RenderContext with no variables and no template loader
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            template_loader: None,
        }
    }

    /// Creates a new RenderContext with a template loader
    ///
    /// # Arguments
    /// * `template_loader` - Arc-wrapped TemplateLoader for loading components
    ///
    /// # Returns
    /// A new RenderContext with the specified template loader
    pub fn with_loader(template_loader: Arc<TemplateLoader>) -> Self {
        Self {
            variables: HashMap::new(),
            template_loader: Some(template_loader),
        }
    }

    /// Pure function: Returns a new context with an additional variable
    ///
    /// # Arguments
    /// * `name` - Variable name (can be String or &str)
    /// * `value` - Variable value of type Value
    ///
    /// # Returns
    /// A new RenderContext with the added variable
    pub fn with_var(mut self, name: impl Into<String>, value: Value) -> Self {
        self.variables.insert(name.into(), value);
        self
    }

    /// Pure function: Returns a new context with multiple variables
    ///
    /// # Arguments
    /// * `vars` - HashMap of variable names to Values
    ///
    /// # Returns
    /// A new RenderContext with all variables added
    pub fn with_vars(mut self, vars: HashMap<String, Value>) -> Self {
        self.variables.extend(vars);
        self
    }

    /// Pure function: Returns a new context with all variables from another context
    ///
    /// # Arguments
    /// * `other` - Reference to another RenderContext to copy variables from
    ///
    /// # Returns
    /// A new RenderContext with merged variables
    pub fn with_context_vars(mut self, other: &RenderContext) -> Self {
        self.variables.extend(other.variables.clone());
        self
    }

    /// Pure function: Create an evaluator from the context's variables
    ///
    /// # Returns
    /// An ExpressionEvaluator initialized with this context's variables
    fn create_evaluator(&self) -> ExpressionEvaluator {
        ExpressionEvaluator::from_variables(self.variables.clone())
    }

    /// Gets a reference to the template loader if one exists
    ///
    /// # Returns
    /// Option containing a reference to the Arc<TemplateLoader>
    fn get_template_loader(&self) -> Option<&Arc<TemplateLoader>> {
        self.template_loader.as_ref()
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

/// HTML renderer with directive support (Immutable, Functional Programming Style)
///
/// Supports rendering templates with RHTMX directives like:
/// - `r-if`, `r-else-if`, `r-else` - Conditional rendering
/// - `r-for` - Loop rendering
/// - `r-match`, `r-when`, `r-default` - Pattern matching
///
/// # Macros Supported
/// - `html! {}` - HTML content
/// - `css! {}` - Scoped CSS styles
/// - `maud! {}` - Maud syntax templates
///
/// # File-based Routing
/// Uses file-based layouts with `_layout.rhtmx` format
pub struct Renderer {
    context: RenderContext,
}

impl Renderer {
    /// Creates a new Renderer with an empty context
    ///
    /// # Returns
    /// A new Renderer instance with no variables or template loader
    pub fn new() -> Self {
        Self {
            context: RenderContext::new(),
        }
    }

    /// Creates a new renderer with access to components via TemplateLoader
    ///
    /// # Arguments
    /// * `template_loader` - Arc-wrapped TemplateLoader for loading components
    ///
    /// # Returns
    /// A new Renderer instance with the specified template loader
    pub fn with_loader(template_loader: Arc<TemplateLoader>) -> Self {
        Self {
            context: RenderContext::with_loader(template_loader),
        }
    }

    /// Creates a renderer from an existing RenderContext
    ///
    /// # Arguments
    /// * `context` - A RenderContext to use for rendering
    ///
    /// # Returns
    /// A new Renderer instance using the provided context
    pub fn from_context(context: RenderContext) -> Self {
        Self { context }
    }

    /// Pure function: Returns a new renderer with an additional variable
    ///
    /// # Arguments
    /// * `name` - Variable name (can be String or &str)
    /// * `value` - Variable value of type Value
    ///
    /// # Returns
    /// A new Renderer with the added variable
    pub fn with_var(mut self, name: impl Into<String>, value: Value) -> Self {
        self.context = self.context.with_var(name, value);
        self
    }

    /// Deprecated: Use with_var() instead for FP style
    #[deprecated(note = "Use with_var() instead for functional programming style")]
    pub fn set_var(&mut self, name: impl Into<String>, value: Value) {
        self.context = self.context.clone().with_var(name, value);
    }

    /// Deprecated: No longer needed in FP style
    #[deprecated(note = "CSS collection is automatic in FP style")]
    pub fn collect_template_css(&mut self, _scoped_css: &Option<()>) {
        // CSS collection happens automatically during render
    }

    /// Pure function: Render a template to HTML (returns RenderResult)
    ///
    /// Processes all directives, interpolations, and returns rendered HTML with collected CSS.
    ///
    /// # Arguments
    /// * `template_content` - The template content as a string slice
    ///
    /// # Returns
    /// * `Result<RenderResult>` - Ok with rendered HTML and CSS, or Err on failure
    pub fn render(&self, template_content: &str) -> Result<RenderResult> {
        let result = self.process_directives(template_content);
        let evaluator = self.context.create_evaluator();
        let interpolated = Self::process_interpolations_with_evaluator(&result.html, &evaluator);
        Ok(RenderResult {
            html: interpolated,
            collected_css: result.collected_css,
        })
    }

    /// Pure function: Process r-if, r-else-if, r-else, r-for, r-match directives
    ///
    /// # Arguments
    /// * `html` - HTML template content to process
    ///
    /// # Returns
    /// * `RenderResult` - Processed HTML with collected CSS
    fn process_directives(&self, html: &str) -> RenderResult {
        let mut result = String::new();
        let mut chars = html.chars().peekable();
        let mut buffer = String::new();
        let mut all_css = HashSet::new();

        while let Some(ch) = chars.next() {
            buffer.push(ch);

            if ch == '<' && chars.peek() != Some(&'/') && chars.peek() != Some(&'!') {
                let tag_start = buffer.len() - 1;
                while let Some(&next_ch) = chars.peek() {
                    buffer.push(chars.next().unwrap());
                    if next_ch == '>' {
                        break;
                    }
                }

                let tag = &buffer[tag_start..];

                if DirectiveParser::has_match_directive(tag) {
                    let (element, _consumed) = Self::extract_element(tag, &mut chars);
                    let match_result = self.process_match(&element);
                    buffer.truncate(tag_start);
                    result.push_str(&buffer);
                    result.push_str(&match_result.html);
                    all_css.extend(match_result.collected_css);
                    buffer.clear();
                    continue;
                }

                if DirectiveParser::has_for_directive(tag) {
                    let (element, _consumed) = Self::extract_element(tag, &mut chars);
                    let loop_result = self.process_loop(&element);
                    buffer.truncate(tag_start);
                    result.push_str(&buffer);
                    result.push_str(&loop_result.html);
                    all_css.extend(loop_result.collected_css);
                    buffer.clear();
                    continue;
                }

                if DirectiveParser::has_if_directive(tag)
                    || DirectiveParser::has_else_if_directive(tag)
                    || DirectiveParser::has_else_directive(tag)
                {
                    let (element, _consumed) = Self::extract_element(tag, &mut chars);
                    let cond_result = self.process_conditional(&element);
                    buffer.truncate(tag_start);
                    result.push_str(&buffer);
                    result.push_str(&cond_result.html);
                    all_css.extend(cond_result.collected_css);
                    buffer.clear();
                    continue;
                }
            }
        }

        result.push_str(&buffer);
        RenderResult {
            html: result,
            collected_css: all_css,
        }
    }

    /// Pure function: Extract a complete HTML element
    ///
    /// # Arguments
    /// * `opening_tag` - The opening HTML tag string
    /// * `chars` - Mutable peekable iterator of remaining characters
    ///
    /// # Returns
    /// * `(String, usize)` - Tuple of (complete element HTML, characters consumed)
    fn extract_element(
        opening_tag: &str,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> (String, usize) {
        let mut element = opening_tag.to_string();
        let mut consumed = 0;
        let tag_name = Self::get_tag_name(opening_tag);

        if opening_tag.trim_end().ends_with("/>") {
            return (element, consumed);
        }

        let mut depth = 1;

        while let Some(ch) = chars.next() {
            consumed += 1;
            element.push(ch);

            if ch == '<' {
                let mut tag_buffer = String::from('<');
                while let Some(&next_ch) = chars.peek() {
                    chars.next();
                    consumed += 1;
                    tag_buffer.push(next_ch);
                    element.push(next_ch);
                    if next_ch == '>' {
                        break;
                    }
                }

                if tag_buffer.starts_with("</") {
                    let closing_name = Self::get_tag_name(&tag_buffer);
                    if closing_name == tag_name {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                } else if !tag_buffer.ends_with("/>") && !tag_buffer.starts_with("<!") {
                    let opening_name = Self::get_tag_name(&tag_buffer);
                    if opening_name == tag_name {
                        depth += 1;
                    }
                }
            }
        }

        (element, consumed)
    }

    /// Pure function: Get tag name from an HTML tag
    ///
    /// # Arguments
    /// * `tag` - HTML tag string (e.g., "<div class='foo'>")
    ///
    /// # Returns
    /// * `String` - The tag name (e.g., "div")
    fn get_tag_name(tag: &str) -> String {
        let tag = tag.trim_start_matches('<').trim_start_matches('/');
        tag.split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('>')
            .to_string()
    }

    /// Pure function: Process a match block (r-match, r-when, r-default)
    ///
    /// # Arguments
    /// * `element` - Complete HTML element with r-match directive
    ///
    /// # Returns
    /// * `RenderResult` - Rendered matched case with CSS
    fn process_match(&self, element: &str) -> RenderResult {
        let tag_end = element.find('>').unwrap_or(element.len());
        let opening_tag = &element[..=tag_end];

        let match_var = match DirectiveParser::extract_match_variable(opening_tag) {
            Some(var) => var,
            None => return RenderResult::new(String::new()),
        };

        let evaluator = self.context.create_evaluator();
        let match_value = evaluator.eval_string(&match_var);
        let cleaned_tag = DirectiveParser::remove_directives(opening_tag);

        let content_start = tag_end + 1;
        let content_end = element
            .rfind(&format!("</{}", Self::get_tag_name(opening_tag)))
            .unwrap_or(element.len());
        let content = &element[content_start..content_end];

        let mut matched_element = None;
        let mut default_element = None;

        let mut chars = content.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '<' && chars.peek() != Some(&'/') && chars.peek() != Some(&'!') {
                let mut tag_buffer = String::from('<');
                while let Some(&next_ch) = chars.peek() {
                    tag_buffer.push(chars.next().unwrap());
                    if next_ch == '>' {
                        break;
                    }
                }

                if DirectiveParser::has_when_directive(&tag_buffer) {
                    let (when_element, _) = Self::extract_element_from_tag(&tag_buffer, &mut chars);

                    if let Some(pattern) = DirectiveParser::extract_when_pattern(&tag_buffer) {
                        if evaluator.eval_string(&pattern) == match_value
                            && matched_element.is_none()
                        {
                            matched_element = Some(when_element);
                        }
                    }
                } else if DirectiveParser::has_default_directive(&tag_buffer) {
                    let (default_elem, _) = Self::extract_element_from_tag(&tag_buffer, &mut chars);
                    default_element = Some(default_elem);
                }
            }
        }

        let selected = matched_element.or(default_element).unwrap_or_default();

        if selected.is_empty() {
            return RenderResult::new(String::new());
        }

        let tag_end_pos = selected.find('>').unwrap_or(selected.len());
        let elem_tag = &selected[..=tag_end_pos];
        let cleaned_elem_tag = DirectiveParser::remove_directives(elem_tag);
        let processed_element = selected.replacen(elem_tag, &cleaned_elem_tag, 1);

        let processed = self.process_directives(&processed_element);
        let interpolated = Self::process_interpolations_with_evaluator(&processed.html, &evaluator);

        let result = format!(
            "{}{}{}",
            cleaned_tag,
            interpolated,
            format!("</{}>", Self::get_tag_name(opening_tag))
        );

        RenderResult {
            html: result,
            collected_css: processed.collected_css,
        }
    }

    /// Pure function: Extract element when we already have the opening tag
    ///
    /// # Arguments
    /// * `opening_tag` - The opening HTML tag string
    /// * `chars` - Mutable peekable iterator of remaining characters
    ///
    /// # Returns
    /// * `(String, usize)` - Tuple of (complete element HTML, characters consumed)
    fn extract_element_from_tag(
        opening_tag: &str,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> (String, usize) {
        let mut element = opening_tag.to_string();
        let mut consumed = 0;
        let tag_name = Self::get_tag_name(opening_tag);

        if opening_tag.trim_end().ends_with("/>") {
            return (element, consumed);
        }

        let mut depth = 1;

        while let Some(ch) = chars.next() {
            consumed += 1;
            element.push(ch);

            if ch == '<' {
                let mut tag_buffer = String::from('<');
                while let Some(&next_ch) = chars.peek() {
                    chars.next();
                    consumed += 1;
                    tag_buffer.push(next_ch);
                    element.push(next_ch);
                    if next_ch == '>' {
                        break;
                    }
                }

                if tag_buffer.starts_with("</") {
                    let closing_name = Self::get_tag_name(&tag_buffer);
                    if closing_name == tag_name {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                } else if !tag_buffer.ends_with("/>") && !tag_buffer.starts_with("<!") {
                    let opening_name = Self::get_tag_name(&tag_buffer);
                    if opening_name == tag_name {
                        depth += 1;
                    }
                }
            }
        }

        (element, consumed)
    }

    /// Pure function: Process a loop element (r-for)
    ///
    /// # Arguments
    /// * `element` - Complete HTML element with r-for directive
    ///
    /// # Returns
    /// * `RenderResult` - Rendered loop iterations with CSS
    fn process_loop(&self, element: &str) -> RenderResult {
        let tag_end = element.find('>').unwrap_or(element.len());
        let opening_tag = &element[..=tag_end];

        let (item_var, index_var, collection) = match DirectiveParser::extract_for_loop(opening_tag)
        {
            Some(info) => info,
            None => return RenderResult::new(String::new()),
        };

        let evaluator = self.context.create_evaluator();
        let items = match evaluator.get_array(&collection) {
            Some(arr) => arr,
            None => return RenderResult::new(String::new()),
        };

        let cleaned_tag = DirectiveParser::remove_directives(opening_tag);

        let content_start = tag_end + 1;
        let content_end = element
            .rfind(&format!("</{}", Self::get_tag_name(opening_tag)))
            .unwrap_or(element.len());
        let content = &element[content_start..content_end];

        let mut result = String::new();
        let mut all_css = HashSet::new();

        for (index, item) in items.iter().enumerate() {
            let mut item_context = self.context.clone();
            item_context = item_context.with_context_vars(&self.context);
            item_context = item_context.with_var(&item_var, item.clone());

            if let Some(idx_var) = &index_var {
                item_context = item_context.with_var(idx_var, Value::Number(index as f64));
            }

            let item_renderer = Renderer::from_context(item_context);
            let processed_content = item_renderer.process_directives(content);
            let item_evaluator = item_renderer.context.create_evaluator();
            let interpolated = Self::process_interpolations_with_evaluator(&processed_content.html, &item_evaluator);

            result.push_str(&cleaned_tag);
            result.push_str(&interpolated);
            result.push_str(&format!("</{}>", Self::get_tag_name(opening_tag)));

            all_css.extend(processed_content.collected_css);
        }

        RenderResult {
            html: result,
            collected_css: all_css,
        }
    }

    /// Pure function: Process a conditional element (r-if, r-else-if, r-else)
    ///
    /// # Arguments
    /// * `element` - Complete HTML element with conditional directive
    ///
    /// # Returns
    /// * `RenderResult` - Rendered element if condition is true, empty otherwise
    fn process_conditional(&self, element: &str) -> RenderResult {
        let tag_end = element.find('>').unwrap_or(element.len());
        let opening_tag = &element[..=tag_end];

        let evaluator = self.context.create_evaluator();
        let should_render = if DirectiveParser::has_if_directive(opening_tag) {
            if let Some(condition) = DirectiveParser::extract_if_condition(opening_tag) {
                evaluator.eval_bool(&condition)
            } else {
                false
            }
        } else if DirectiveParser::has_else_if_directive(opening_tag) {
            if let Some(condition) = DirectiveParser::extract_else_if_condition(opening_tag) {
                evaluator.eval_bool(&condition)
            } else {
                false
            }
        } else if DirectiveParser::has_else_directive(opening_tag) {
            true
        } else {
            false
        };

        if should_render {
            let cleaned_tag = DirectiveParser::remove_directives(opening_tag);
            let processed = element.replacen(opening_tag, &cleaned_tag, 1);
            RenderResult::new(processed)
        } else {
            RenderResult::new(String::new())
        }
    }

    /// Pure function: Process {expression} interpolations with an evaluator
    ///
    /// # Arguments
    /// * `html` - HTML content with {expression} interpolations
    /// * `evaluator` - ExpressionEvaluator for evaluating expressions
    ///
    /// # Returns
    /// * `String` - HTML with all interpolations replaced with evaluated values
    fn process_interpolations_with_evaluator(html: &str, evaluator: &ExpressionEvaluator) -> String {
        let re = Regex::new(r"\{([^}]+)\}").unwrap();

        re.replace_all(html, |caps: &regex::Captures| {
            let expr = &caps[1];
            evaluator.eval_string(expr)
        })
        .to_string()
    }

    /// Renders a template without a layout wrapper (for HTMX partial responses)
    ///
    /// # Arguments
    /// * `content` - Template content to render
    ///
    /// # Returns
    /// * `Result<RenderResult>` - Rendered HTML without layout wrapper
    pub fn render_partial(&self, content: &str) -> Result<RenderResult> {
        self.render(content)
    }

    /// Renders a page with layout content
    ///
    /// Used by file-based router with `_layout.rhtmx` files.
    /// Replaces {slots.content} in layout with rendered page content.
    ///
    /// # Arguments
    /// * `layout_content` - Layout template content (from `_layout.rhtmx`)
    /// * `page_content` - Page template content to inject into layout
    ///
    /// # Returns
    /// * `Result<RenderResult>` - Rendered HTML with layout and injected CSS
    pub fn render_with_layout(
        &self,
        layout_content: &str,
        page_content: &str,
    ) -> Result<RenderResult> {
        let layout_result = self.process_directives(layout_content);
        let page_result = self.render(page_content)?;

        let result = layout_result.html.replace("{slots.content}", &page_result.html);

        let evaluator = self.context.create_evaluator();
        let interpolated = Self::process_interpolations_with_evaluator(&result, &evaluator);

        let mut combined_css = layout_result.collected_css.clone();
        combined_css.extend(page_result.collected_css);

        let final_html = Self::inject_css(&interpolated, &combined_css);

        Ok(RenderResult {
            html: final_html,
            collected_css: combined_css,
        })
    }

    /// Pure function: Inject collected CSS into the HTML <head>
    ///
    /// # Arguments
    /// * `html` - HTML document to inject CSS into
    /// * `collected_css` - Set of CSS strings to inject
    ///
    /// # Returns
    /// * `String` - HTML with CSS injected in <head> section
    fn inject_css(html: &str, collected_css: &HashSet<String>) -> String {
        if collected_css.is_empty() {
            return html.to_string();
        }

        let combined_css = collected_css
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let style_tag = format!("<style data-rhtmx-scoped>\n{}\n</style>", combined_css);

        if let Some(head_close) = html.find("</head>") {
            let mut result = html.to_string();
            result.insert_str(head_close, &style_tag);
            result.insert(head_close, '\n');
            return result;
        }

        if let Some(head_open) = html.find("<head>") {
            let insert_pos = head_open + 6;
            let mut result = html.to_string();
            result.insert(insert_pos, '\n');
            result.insert_str(insert_pos + 1, &style_tag);
            return result;
        }

        html.to_string()
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}