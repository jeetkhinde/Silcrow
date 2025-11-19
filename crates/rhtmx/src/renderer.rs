// File: src/renderer.rs
// Purpose: Render rhtmx templates with directive support (Functional Programming Style)

use crate::template_loader::TemplateLoader;
use anyhow::Result;
use regex::Regex;
use rhtmx_parser::{DirectiveParser, ExpressionEvaluator, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Layout directive parsed from @layout(...) decorator
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutDirective {
    /// @layout(false) - No layout should be applied
    None,
    /// @layout("name") - Use custom layout by name
    Custom(String),
}

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

/// Immutable rendering context that flows through the rendering pipeline
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

    /// Pure function: Returns a new context with an additional variable
    pub fn with_var(mut self, name: impl Into<String>, value: Value) -> Self {
        self.variables.insert(name.into(), value);
        self
    }

    /// Pure function: Returns a new context with multiple variables
    pub fn with_vars(mut self, vars: HashMap<String, Value>) -> Self {
        self.variables.extend(vars);
        self
    }

    /// Pure function: Returns a new context with all variables from another context
    pub fn with_context_vars(mut self, other: &RenderContext) -> Self {
        self.variables.extend(other.variables.clone());
        self
    }

    /// Pure function: Create an evaluator from the context's variables
    fn create_evaluator(&self) -> ExpressionEvaluator {
        ExpressionEvaluator::from_variables(self.variables.clone())
    }

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
pub struct Renderer {
    context: RenderContext,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            context: RenderContext::new(),
        }
    }

    /// Create a new renderer with access to components
    pub fn with_loader(template_loader: Arc<TemplateLoader>) -> Self {
        Self {
            context: RenderContext::with_loader(template_loader),
        }
    }

    /// Create a renderer from a context
    pub fn from_context(context: RenderContext) -> Self {
        Self { context }
    }

    /// Pure function: Returns a new renderer with an additional variable
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
    pub fn render(&self, template_content: &str) -> Result<RenderResult> {
        let html = Self::extract_html(template_content);
        let result = self.process_directives(&html);
        let evaluator = self.context.create_evaluator();
        let interpolated = Self::process_interpolations_with_evaluator(&result.html, &evaluator);
        Ok(RenderResult {
            html: interpolated,
            collected_css: result.collected_css,
        })
    }

    /// Pure function: Find the position of slots block
    fn find_slots_block(content: &str) -> Option<usize> {
        content
            .find("__rhtmx_slots__ {")
            .or_else(|| content.find("slots {"))
    }

    /// Pure function: Check if content has a WebPage component
    fn has_component(content: &str) -> bool {
        let search_start = if let Some(slots_pos) = Self::find_slots_block(content) {
            let mut depth = 0;
            let mut found_opening = false;
            let mut slots_end = slots_pos;

            for (byte_idx, ch) in content[slots_pos..].char_indices() {
                if ch == '{' {
                    depth += 1;
                    found_opening = true;
                } else if ch == '}' {
                    depth -= 1;
                    if found_opening && depth == 0 {
                        slots_end = slots_pos + byte_idx + ch.len_utf8();
                        break;
                    }
                }
            }
            slots_end
        } else {
            0
        };

        content[search_start..].contains("WebPage {")
    }

    /// Pure function: Extract HTML content from rhtmx template
    fn extract_html(content: &str) -> String {
        let search_start = if let Some(slots_pos) = Self::find_slots_block(content) {
            let mut depth = 0;
            let mut found_opening = false;
            let mut slots_end = slots_pos;

            for (byte_idx, ch) in content[slots_pos..].char_indices() {
                if ch == '{' {
                    depth += 1;
                    found_opening = true;
                } else if ch == '}' {
                    depth -= 1;
                    if found_opening && depth == 0 {
                        slots_end = slots_pos + byte_idx + ch.len_utf8();
                        break;
                    }
                }
            }
            slots_end
        } else {
            0
        };

        if let Some(webpage_pos) = content[search_start..].find("WebPage {") {
            let abs_webpage_pos = search_start + webpage_pos;
            if let Some(start) = content[abs_webpage_pos..].find('{') {
                let abs_start = abs_webpage_pos + start;
                let mut depth = 0;
                let mut end_pos = None;

                for (byte_idx, ch) in content[abs_start..].char_indices() {
                    if ch == '{' {
                        depth += 1;
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            end_pos = Some(abs_start + byte_idx);
                            break;
                        }
                    }
                }

                if let Some(end) = end_pos {
                    let html = &content[abs_start + 1..end];
                    return html.trim().to_string();
                }
            }
        }

        content.trim().to_string()
    }

    /// Pure function: Extract slot values from page template
    fn extract_slots(page_content: &str) -> HashMap<String, String> {
        let mut slots = HashMap::new();

        if let Some(slots_start) = page_content.find("slots {") {
            let mut depth = 0;
            let mut found_opening = false;
            let mut end_pos = None;

            for (byte_idx, ch) in page_content[slots_start..].char_indices() {
                if ch == '{' {
                    depth += 1;
                    found_opening = true;
                } else if ch == '}' {
                    depth -= 1;
                    if found_opening && depth == 0 {
                        end_pos = Some(slots_start + byte_idx);
                        break;
                    }
                }
            }

            if let Some(end) = end_pos {
                let slots_block = &page_content[slots_start + 7..end];

                for line in slots_block.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    if let Some(colon_pos) = line.find(':') {
                        let key = line[..colon_pos].trim();
                        let value_part = line[colon_pos + 1..].trim().trim_end_matches(',');
                        let value = value_part.trim_matches('"');
                        slots.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        slots
    }

    /// Pure function: Process r-if, r-else-if, r-else, r-for, r-match, r-component directives
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

                if DirectiveParser::has_component_directive(tag) {
                    let component_result = self.process_component(tag);
                    buffer.truncate(tag_start);
                    result.push_str(&buffer);
                    result.push_str(&component_result.html);
                    all_css.extend(component_result.collected_css);
                    buffer.clear();
                    continue;
                }

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
    fn get_tag_name(tag: &str) -> String {
        let tag = tag.trim_start_matches('<').trim_start_matches('/');
        tag.split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('>')
            .to_string()
    }

    /// Pure function: Process a component (r-component)
    fn process_component(&self, tag: &str) -> RenderResult {
        let (name, props) = match DirectiveParser::extract_component(tag) {
            Some(info) => info,
            None => return RenderResult::new(String::new()),
        };

        let loader = match self.context.get_template_loader() {
            Some(loader) => loader,
            None => return RenderResult::new(String::new()),
        };

        let component = match loader.get_component(&name) {
            Some(comp) => comp,
            None => return RenderResult::new(format!("<!-- Component '{}' not found -->", name)),
        };

        let component_html = Self::extract_html(&component.content);

        // Create a new context with component props
        let mut component_context = self.context.clone();
        component_context = component_context.with_context_vars(&self.context);

        for (key, value) in props {
            component_context = component_context.with_var(&key, Value::String(value));
        }

        let component_renderer = Renderer::from_context(component_context);
        let processed = component_renderer.process_directives(&component_html);
        let evaluator = component_renderer.context.create_evaluator();
        let interpolated = Self::process_interpolations_with_evaluator(&processed.html, &evaluator);

        let scope_name = name.clone();
        let scoped_html = Self::add_scope_attribute(&interpolated, &scope_name);

        RenderResult {
            html: scoped_html,
            collected_css: processed.collected_css,
        }
    }

    /// Pure function: Add data-rhtmx scope attribute to the root element
    fn add_scope_attribute(html: &str, scope_name: &str) -> String {
        let html = html.trim();

        if let Some(first_gt) = html.find('>') {
            if let Some(first_lt) = html.find('<') {
                if first_lt == 0 {
                    let tag = &html[..=first_gt];

                    if tag.contains("data-rhtmx=") {
                        return html.to_string();
                    }

                    let insert_pos = if tag.ends_with("/>") {
                        first_gt - 1
                    } else {
                        first_gt
                    };

                    return format!(
                        "{} data-rhtmx=\"{}\"{}",
                        &html[..insert_pos],
                        scope_name,
                        &html[insert_pos..]
                    );
                }
            }
        }

        format!("<div data-rhtmx=\"{}\">{}</div>", scope_name, html)
    }

    /// Pure function: Process a match block (r-match, r-when, r-default)
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
    fn process_interpolations_with_evaluator(html: &str, evaluator: &ExpressionEvaluator) -> String {
        let re = Regex::new(r"\{([^}]+)\}").unwrap();

        re.replace_all(html, |caps: &regex::Captures| {
            let expr = &caps[1];
            evaluator.eval_string(expr)
        })
        .to_string()
    }

    /// Pure function: Render a partial (without layout)
    pub fn render_partial(&self, content: &str) -> Result<RenderResult> {
        let clean_content = Self::strip_layout_directive(content);
        self.render(&clean_content)
    }

    /// Pure function: Check if content should be rendered as a partial
    pub fn is_partial(&self, content: &str) -> bool {
        !Self::has_component(content)
    }

    /// Pure function: Check if content has named partials
    pub fn has_named_partials(&self, content: &str) -> bool {
        content.contains("partial ")
    }

    /// Pure function: List all named partials in content
    pub fn list_partials(&self, content: &str) -> Vec<String> {
        let mut partials = Vec::new();
        let re = Regex::new(r"partial\s+(\w+)\s*\(").unwrap();

        for cap in re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                partials.push(name.as_str().to_string());
            }
        }

        partials
    }

    /// Pure function: Extract a named partial by name
    fn extract_named_partial(content: &str, name: &str) -> Result<String> {
        let search_pattern = format!("partial {}", name);

        if let Some(start_pos) = content.find(&search_pattern) {
            let after_partial = &content[start_pos..];

            if let Some(brace_pos) = after_partial.find('{') {
                let abs_brace_pos = start_pos + brace_pos;
                let mut depth = 0;
                let mut end_pos = None;

                for (byte_idx, ch) in content[abs_brace_pos..].char_indices() {
                    if ch == '{' {
                        depth += 1;
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            end_pos = Some(abs_brace_pos + byte_idx);
                            break;
                        }
                    }
                }

                if let Some(end) = end_pos {
                    let html = &content[abs_brace_pos + 1..end];
                    return Ok(html.trim().to_string());
                }
            }
        }

        anyhow::bail!("Partial '{}' not found", name)
    }

    /// Pure function: Render a named partial with name
    pub fn render_named_partial(&self, content: &str, name: &str) -> Result<RenderResult> {
        let partial_html = Self::extract_named_partial(content, name)?;
        let processed = self.process_directives(&partial_html);
        let evaluator = self.context.create_evaluator();
        let interpolated = Self::process_interpolations_with_evaluator(&processed.html, &evaluator);

        Ok(RenderResult {
            html: interpolated,
            collected_css: processed.collected_css,
        })
    }

    /// Pure function: Parse @layout directive from page content
    pub fn parse_layout_directive(&self, content: &str) -> Option<LayoutDirective> {
        let re = Regex::new(r#"^\s*@layout\((false|"([^"]+)")\)"#).unwrap();

        if let Some(caps) = re.captures(content) {
            if caps.get(1).map(|m| m.as_str()) == Some("false") {
                return Some(LayoutDirective::None);
            } else if let Some(name) = caps.get(2) {
                return Some(LayoutDirective::Custom(name.as_str().to_string()));
            }
        }

        None
    }

    /// Pure function: Strip @layout directive from content
    pub fn strip_layout_directive(content: &str) -> String {
        let re = Regex::new(r#"^\s*@layout\((false|"[^"]+")\)\s*\n?"#).unwrap();
        re.replace(content, "").to_string()
    }

    /// Pure function: Render page with layout
    pub fn render_with_layout(
        &self,
        layout_content: &str,
        page_content: &str,
    ) -> Result<RenderResult> {
        let clean_page_content = Self::strip_layout_directive(page_content);
        let slots = Self::extract_slots(&clean_page_content);

        let layout_html_raw = Self::extract_html(layout_content);
        let layout_result = self.process_directives(&layout_html_raw);

        let page_result = self.render(&clean_page_content)?;

        let mut result = layout_result.html.replace("{slots.content}", &page_result.html);

        let slot_pattern =
            Regex::new(r#"\{slots\.get\("([^"]+)"\)\.unwrap_or\("([^"]*)"\)\}"#).unwrap();
        result = slot_pattern
            .replace_all(&result, |caps: &regex::Captures| {
                let key = &caps[1];
                let default = &caps[2];
                slots
                    .get(key)
                    .map(|s| s.as_str())
                    .unwrap_or(default)
                    .to_string()
            })
            .to_string();

        let evaluator = self.context.create_evaluator();
        result = Self::process_interpolations_with_evaluator(&result, &evaluator);

        let mut combined_css = layout_result.collected_css.clone();
        combined_css.extend(page_result.collected_css);

        let final_html = Self::inject_css(&result, &combined_css);

        Ok(RenderResult {
            html: final_html,
            collected_css: combined_css,
        })
    }

    /// Pure function: Inject collected CSS into the HTML <head>
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