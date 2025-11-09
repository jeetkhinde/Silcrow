// File: src/renderer.rs
// Purpose: Render RHTML templates with directive support

use crate::template_loader::TemplateLoader;
use anyhow::Result;
use regex::Regex;
use rhtml_parser::{DirectiveParser, ExpressionEvaluator, Value};
use std::collections::HashSet;
use std::sync::Arc;

/// Layout directive parsed from @layout(...) decorator
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutDirective {
    /// @layout(false) - No layout should be applied
    None,
    /// @layout("name") - Use custom layout by name
    Custom(String),
}

/// HTML renderer with directive support
pub struct Renderer {
    evaluator: ExpressionEvaluator,
    template_loader: Option<Arc<TemplateLoader>>,
    collected_css: HashSet<String>, // Track which component CSS has been collected
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            evaluator: ExpressionEvaluator::new(),
            template_loader: None,
            collected_css: HashSet::new(),
        }
    }

    /// Create a new renderer with access to components
    pub fn with_loader(template_loader: Arc<TemplateLoader>) -> Self {
        Self {
            evaluator: ExpressionEvaluator::new(),
            template_loader: Some(template_loader),
            collected_css: HashSet::new(),
        }
    }

    /// Set a variable for expression evaluation
    pub fn set_var(&mut self, name: impl Into<String>, value: Value) {
        self.evaluator.set(name, value);
    }

    /// Collect CSS from a template's scoped CSS
    pub fn collect_template_css(&mut self, scoped_css: &Option<rhtml_parser::ScopedCss>) {
        if let Some(css) = scoped_css {
            self.collected_css.insert(css.scoped_css.clone());
        }
    }

    /// Render a template to HTML
    pub fn render(&mut self, template_content: &str) -> Result<String> {
        let html = self.extract_html(template_content);
        let processed = self.process_directives(&html);
        let interpolated = self.process_interpolations(&processed);
        Ok(interpolated)
    }

    /// Find the position of slots block (either old "slots {" or new "__rhtml_slots__ {")
    fn find_slots_block(&self, content: &str) -> Option<usize> {
        content.find("__rhtml_slots__ {")
            .or_else(|| content.find("slots {"))
    }

    /// Check if content has a WebPage component
    fn has_component(&self, content: &str) -> bool {
        // Skip slots block if exists
        let search_start = if let Some(slots_pos) = self.find_slots_block(content) {
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

    /// Extract HTML content from RHTML template
    /// This needs to extract ONLY the WebPage function content, not slots block
    /// If no WebPage component exists, returns the entire content (for partials)
    fn extract_html(&self, content: &str) -> String {
        // First, skip past any slots block if it exists
        let search_start = if let Some(slots_pos) = self.find_slots_block(content) {
            // Find the end of slots block
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

        // Now find "WebPage {" keyword after the slots block
        if let Some(webpage_pos) = content[search_start..].find("WebPage {") {
            let abs_webpage_pos = search_start + webpage_pos;
            // Find the opening brace after WebPage
            if let Some(start) = content[abs_webpage_pos..].find('{') {
                let abs_start = abs_webpage_pos + start;

                // Find matching closing brace
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

        // No WebPage component found - treat as partial (return entire content)
        content.trim().to_string()
    }

    /// Extract slot values from page template
    fn extract_slots(&self, page_content: &str) -> std::collections::HashMap<String, String> {
        let mut slots = std::collections::HashMap::new();

        // Look for slots { ... } block
        if let Some(slots_start) = page_content.find("slots {") {
            // Find matching closing brace
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
                let slots_block = &page_content[slots_start + 7..end]; // Skip "slots {"

                // Parse each slot line: title: "value",
                for line in slots_block.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Parse: key: "value" or key: "value",
                    if let Some(colon_pos) = line.find(':') {
                        let key = line[..colon_pos].trim();
                        let value_part = line[colon_pos + 1..].trim().trim_end_matches(',');

                        // Remove quotes
                        let value = value_part.trim_matches('"');

                        slots.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        slots
    }

    /// Process r-if, r-else-if, r-else directives
    fn process_directives(&mut self, html: &str) -> String {
        let mut result = String::new();
        let mut chars = html.chars().peekable();
        let mut buffer = String::new();

        while let Some(ch) = chars.next() {
            buffer.push(ch);

            // Look for opening tags
            if ch == '<' && chars.peek() != Some(&'/') && chars.peek() != Some(&'!') {
                // Read until we find the end of the tag
                let tag_start = buffer.len() - 1;
                while let Some(&next_ch) = chars.peek() {
                    buffer.push(chars.next().unwrap());
                    if next_ch == '>' {
                        break;
                    }
                }

                let tag = &buffer[tag_start..];

                // Check if this tag has component directive
                if DirectiveParser::has_component_directive(tag) {
                    // Process the component inline (self-closing tag)
                    let processed = self.process_component(tag);

                    // Remove the tag from buffer and add processed result
                    buffer.truncate(tag_start);
                    result.push_str(&buffer);
                    result.push_str(&processed);
                    buffer.clear();
                    continue;
                }

                // Check if this tag has match directive
                if DirectiveParser::has_match_directive(tag) {
                    // Extract the element (tag + content + closing tag)
                    let (element, _consumed) = self.extract_element(tag, &mut chars);

                    // Process the match block
                    let processed = self.process_match(&element);

                    // Remove the tag from buffer and add processed result
                    buffer.truncate(tag_start);
                    result.push_str(&buffer);
                    result.push_str(&processed);
                    buffer.clear();
                    continue;
                }

                // Check if this tag has loop directive
                if DirectiveParser::has_for_directive(tag) {
                    // Extract the element (tag + content + closing tag)
                    let (element, _consumed) = self.extract_element(tag, &mut chars);

                    // Process the loop
                    let processed = self.process_loop(&element);

                    // Remove the tag from buffer and add processed result
                    buffer.truncate(tag_start);
                    result.push_str(&buffer);
                    result.push_str(&processed);
                    buffer.clear();
                    continue;
                }

                // Check if this tag has conditional directives
                if DirectiveParser::has_if_directive(tag)
                    || DirectiveParser::has_else_if_directive(tag)
                    || DirectiveParser::has_else_directive(tag)
                {
                    // Extract the element (tag + content + closing tag)
                    let (element, _consumed) = self.extract_element(tag, &mut chars);

                    // Process the conditional
                    let processed = self.process_conditional(&element);

                    // Remove the tag from buffer and add processed result
                    buffer.truncate(tag_start);
                    result.push_str(&buffer);
                    result.push_str(&processed);
                    buffer.clear();
                    continue;
                }
            }
        }

        result.push_str(&buffer);
        result
    }

    /// Extract a complete HTML element (opening tag, content, closing tag)
    fn extract_element(
        &self,
        opening_tag: &str,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> (String, usize) {
        let mut element = opening_tag.to_string();
        let mut consumed = 0;

        // Get tag name
        let tag_name = self.get_tag_name(opening_tag);

        // If self-closing, return immediately
        if opening_tag.trim_end().ends_with("/>") {
            return (element, consumed);
        }

        // Read content until closing tag
        let mut depth = 1;

        while let Some(ch) = chars.next() {
            consumed += 1;
            element.push(ch);

            // Check for tags
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

                // Check if opening or closing tag
                if tag_buffer.starts_with("</") {
                    let closing_name = self.get_tag_name(&tag_buffer);
                    if closing_name == tag_name {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                } else if !tag_buffer.ends_with("/>") && !tag_buffer.starts_with("<!") {
                    let opening_name = self.get_tag_name(&tag_buffer);
                    if opening_name == tag_name {
                        depth += 1;
                    }
                }
            }
        }

        (element, consumed)
    }

    /// Get tag name from an HTML tag
    fn get_tag_name(&self, tag: &str) -> String {
        let tag = tag.trim_start_matches('<').trim_start_matches('/');
        tag.split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('>')
            .to_string()
    }

    /// Process a component (r-component)
    fn process_component(&mut self, tag: &str) -> String {
        // Extract component name and props
        let (name, props) = match DirectiveParser::extract_component(tag) {
            Some(info) => info,
            None => return String::new(),
        };

        // Get template loader
        let loader = match &self.template_loader {
            Some(loader) => loader,
            None => return String::new(), // No loader available
        };

        // Load component template
        let component = match loader.get_component(&name) {
            Some(comp) => comp,
            None => return format!("<!-- Component '{}' not found -->", name),
        };

        // Collect CSS from this component
        if let Some(ref scoped_css) = component.scoped_css {
            self.collected_css.insert(scoped_css.scoped_css.clone());
        }

        // Extract HTML from component
        let component_html = self.extract_html(&component.content);

        // Create a new renderer for the component with props as variables
        let mut component_renderer = if let Some(loader) = &self.template_loader {
            Renderer::with_loader(Arc::clone(loader))
        } else {
            Renderer::new()
        };

        // Copy all existing variables to component renderer
        for (var_name, value) in &self.evaluator.variables {
            component_renderer.evaluator.set(var_name, value.clone());
        }

        // Set props as variables in component renderer
        for (key, value) in props {
            component_renderer.evaluator.set(&key, Value::String(value));
        }

        // Render the component
        let processed = component_renderer.process_directives(&component_html);
        let interpolated = component_renderer.process_interpolations(&processed);

        // Add scope attribute to the component HTML
        let scope_name = component
            .scoped_css
            .as_ref()
            .map(|css| css.scope_name.clone())
            .unwrap_or(name.clone());

        self.add_scope_attribute(&interpolated, &scope_name)
    }

    /// Add data-rhtml scope attribute to the root element
    fn add_scope_attribute(&self, html: &str, scope_name: &str) -> String {
        let html = html.trim();

        // Find the first opening tag
        if let Some(first_gt) = html.find('>') {
            if let Some(first_lt) = html.find('<') {
                if first_lt == 0 {
                    // It's an opening tag
                    let tag = &html[..=first_gt];

                    // Check if it's a self-closing tag or already has the attribute
                    if tag.contains("data-rhtml=") {
                        return html.to_string();
                    }

                    // Insert the data-rhtml attribute before the closing >
                    let insert_pos = if tag.ends_with("/>") {
                        first_gt - 1
                    } else {
                        first_gt
                    };

                    let new_tag = format!(
                        "{} data-rhtml=\"{}\"{}",
                        &html[..insert_pos],
                        scope_name,
                        &html[insert_pos..]
                    );

                    return new_tag;
                }
            }
        }

        // If we can't find a tag, wrap it in a div with the scope attribute
        format!("<div data-rhtml=\"{}\">{}</div>", scope_name, html)
    }

    /// Process a match block (r-match, r-when, r-default)
    fn process_match(&mut self, element: &str) -> String {
        // Extract opening tag
        let tag_end = element.find('>').unwrap_or(element.len());
        let opening_tag = &element[..=tag_end];

        // Extract match variable
        let match_var = match DirectiveParser::extract_match_variable(opening_tag) {
            Some(var) => var,
            None => return String::new(),
        };

        // Get the value to match against
        let match_value = self.evaluator.eval_string(&match_var);

        // Clean the opening tag (remove r-match)
        let cleaned_tag = DirectiveParser::remove_directives(opening_tag);

        // Get content between opening and closing tags
        let content_start = tag_end + 1;
        let content_end = element
            .rfind(&format!("</{}", self.get_tag_name(opening_tag)))
            .unwrap_or(element.len());
        let content = &element[content_start..content_end];

        // Parse child elements looking for r-when and r-default
        let mut matched_element = None;
        let mut default_element = None;

        // Parse through content to find when/default elements
        let mut chars = content.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '<' && chars.peek() != Some(&'/') && chars.peek() != Some(&'!') {
                // Found an opening tag, collect it
                let mut tag_buffer = String::from('<');
                while let Some(&next_ch) = chars.peek() {
                    tag_buffer.push(chars.next().unwrap());
                    if next_ch == '>' {
                        break;
                    }
                }

                // Check if this is a when or default directive
                if DirectiveParser::has_when_directive(&tag_buffer) {
                    // Extract the full element
                    let (when_element, _) = self.extract_element_from_tag(&tag_buffer, &mut chars);

                    // Check if this when pattern matches
                    if let Some(pattern) = DirectiveParser::extract_when_pattern(&tag_buffer) {
                        if self.evaluator.eval_string(&pattern) == match_value
                            && matched_element.is_none()
                        {
                            matched_element = Some(when_element);
                        }
                    }
                } else if DirectiveParser::has_default_directive(&tag_buffer) {
                    // Extract the default element
                    let (default_elem, _) = self.extract_element_from_tag(&tag_buffer, &mut chars);
                    default_element = Some(default_elem);
                }
            }
        }

        // Render the matched element or default
        let selected = matched_element.or(default_element).unwrap_or_default();

        if selected.is_empty() {
            return String::new();
        }

        // Remove directives from the selected element and process it
        let tag_end_pos = selected.find('>').unwrap_or(selected.len());
        let elem_tag = &selected[..=tag_end_pos];
        let cleaned_elem_tag = DirectiveParser::remove_directives(elem_tag);
        let processed_element = selected.replacen(elem_tag, &cleaned_elem_tag, 1);

        // Process the content recursively
        let processed = self.process_directives(&processed_element);
        let interpolated = self.process_interpolations(&processed);

        // Wrap in the parent element
        let mut result = String::new();
        result.push_str(&cleaned_tag);
        result.push_str(&interpolated);
        result.push_str(&format!("</{}>", self.get_tag_name(opening_tag)));

        result
    }

    /// Helper to extract element when we already have the opening tag
    fn extract_element_from_tag(
        &self,
        opening_tag: &str,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> (String, usize) {
        let mut element = opening_tag.to_string();
        let mut consumed = 0;

        // Get tag name
        let tag_name = self.get_tag_name(opening_tag);

        // If self-closing, return immediately
        if opening_tag.trim_end().ends_with("/>") {
            return (element, consumed);
        }

        // Read content until closing tag
        let mut depth = 1;

        while let Some(ch) = chars.next() {
            consumed += 1;
            element.push(ch);

            // Check for tags
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

                // Check if opening or closing tag
                if tag_buffer.starts_with("</") {
                    let closing_name = self.get_tag_name(&tag_buffer);
                    if closing_name == tag_name {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                } else if !tag_buffer.ends_with("/>") && !tag_buffer.starts_with("<!") {
                    let opening_name = self.get_tag_name(&tag_buffer);
                    if opening_name == tag_name {
                        depth += 1;
                    }
                }
            }
        }

        (element, consumed)
    }

    /// Process a loop element (r-for)
    fn process_loop(&mut self, element: &str) -> String {
        // Extract opening tag
        let tag_end = element.find('>').unwrap_or(element.len());
        let opening_tag = &element[..=tag_end];

        // Extract loop information
        let (item_var, index_var, collection) = match DirectiveParser::extract_for_loop(opening_tag)
        {
            Some(info) => info,
            None => return String::new(),
        };

        // Get the collection from evaluator
        let items = match self.evaluator.get_array(&collection) {
            Some(arr) => arr,
            None => return String::new(),
        };

        // Clean the opening tag (remove r-for)
        let cleaned_tag = DirectiveParser::remove_directives(opening_tag);

        // Get content between opening and closing tags
        let content_start = tag_end + 1;
        let content_end = element
            .rfind(&format!("</{}", self.get_tag_name(opening_tag)))
            .unwrap_or(element.len());
        let content = &element[content_start..content_end];

        // Render for each item
        let mut result = String::new();
        for (index, item) in items.iter().enumerate() {
            // Create a new renderer with item variable
            let mut item_renderer = Renderer::new();

            // Copy all existing variables
            for (name, value) in &self.evaluator.variables {
                item_renderer.evaluator.set(name, value.clone());
            }

            // Set loop variables
            item_renderer.evaluator.set(&item_var, item.clone());
            if let Some(idx_var) = &index_var {
                item_renderer
                    .evaluator
                    .set(idx_var, Value::Number(index as f64));
            }

            // Process the content
            let processed_content = item_renderer.process_directives(content);
            let interpolated = item_renderer.process_interpolations(&processed_content);

            // Add the element with processed content
            result.push_str(&cleaned_tag);
            result.push_str(&interpolated);
            result.push_str(&format!("</{}>", self.get_tag_name(opening_tag)));
        }

        result
    }

    /// Process a conditional element (r-if, r-else-if, r-else)
    fn process_conditional(&mut self, element: &str) -> String {
        // Extract opening tag
        let tag_end = element.find('>').unwrap_or(element.len());
        let opening_tag = &element[..=tag_end];

        // Determine which directive it has
        let should_render = if DirectiveParser::has_if_directive(opening_tag) {
            if let Some(condition) = DirectiveParser::extract_if_condition(opening_tag) {
                self.evaluator.eval_bool(&condition)
            } else {
                false
            }
        } else if DirectiveParser::has_else_if_directive(opening_tag) {
            if let Some(condition) = DirectiveParser::extract_else_if_condition(opening_tag) {
                self.evaluator.eval_bool(&condition)
            } else {
                false
            }
        } else if DirectiveParser::has_else_directive(opening_tag) {
            true // r-else always renders (we'll handle chaining later)
        } else {
            false
        };

        if should_render {
            // Remove directive and render content
            let cleaned_tag = DirectiveParser::remove_directives(opening_tag);
            element.replacen(opening_tag, &cleaned_tag, 1)
        } else {
            // Don't render
            String::new()
        }
    }

    /// Process {expression} interpolations
    fn process_interpolations(&self, html: &str) -> String {
        let re = Regex::new(r"\{([^}]+)\}").unwrap();

        re.replace_all(html, |caps: &regex::Captures| {
            let expr = &caps[1];
            self.evaluator.eval_string(expr)
        })
        .to_string()
    }

    /// Render page with layout
    /// Render a partial (without layout)
    /// Use this for HTML fragments, HTMX responses, or pages without Page component
    pub fn render_partial(&mut self, content: &str) -> Result<String> {
        // Strip @layout directive if present, then render without layout wrapping
        let clean_content = self.strip_layout_directive(content);
        self.render(&clean_content)
    }

    /// Check if content should be rendered as a partial
    /// Returns true if content has no WebPage component
    pub fn is_partial(&self, content: &str) -> bool {
        !self.has_component(content)
    }

    /// Check if content has named partials (partial Name(...) syntax)
    pub fn has_named_partials(&self, content: &str) -> bool {
        content.contains("partial ")
    }

    /// List all named partials in content
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

    /// Extract a named partial by name
    /// Finds: partial Name(...) { ... }
    fn extract_named_partial(&self, content: &str, name: &str) -> Result<String> {
        // Find pattern: partial {name}(
        let search_pattern = format!("partial {}", name);

        if let Some(start_pos) = content.find(&search_pattern) {
            // Find the opening brace after partial Name(...)
            let after_partial = &content[start_pos..];

            if let Some(brace_pos) = after_partial.find('{') {
                let abs_brace_pos = start_pos + brace_pos;

                // Find matching closing brace
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

    /// Render a named partial with name
    /// Example: render_named_partial(content, "Stats")
    pub fn render_named_partial(&mut self, content: &str, name: &str) -> Result<String> {
        // Extract the named partial HTML
        let partial_html = self.extract_named_partial(content, name)?;

        // TODO: In future, execute associated data function here
        // For now, just render the HTML with current variables

        let processed = self.process_directives(&partial_html);
        let interpolated = self.process_interpolations(&processed);

        Ok(interpolated)
    }

    /// Parse @layout directive from page content
    /// Returns: Some(LayoutDirective) if found, None if not present
    ///
    /// Supported formats:
    /// - @layout(false) -> No layout
    /// - @layout("custom") -> Use specific layout
    /// - No directive -> Default behavior (use _layout.rhtml)
    pub fn parse_layout_directive(&self, content: &str) -> Option<LayoutDirective> {
        // Pattern: @layout(false) or @layout("name") at the START of the file
        // Use ^ to match only at the beginning, with optional whitespace
        let re = Regex::new(r#"^\s*@layout\((false|"([^"]+)")\)"#).unwrap();

        if let Some(caps) = re.captures(content) {
            if caps.get(1).map(|m| m.as_str()) == Some("false") {
                // @layout(false)
                return Some(LayoutDirective::None);
            } else if let Some(name) = caps.get(2) {
                // @layout("custom")
                return Some(LayoutDirective::Custom(name.as_str().to_string()));
            }
        }

        None
    }

    /// Strip @layout directive from content (for rendering)
    pub fn strip_layout_directive(&self, content: &str) -> String {
        // Strip @layout directive only at the start of the file
        let re = Regex::new(r#"^\s*@layout\((false|"[^"]+")\)\s*\n?"#).unwrap();
        re.replace(content, "").to_string()
    }

    pub fn render_with_layout(
        &mut self,
        layout_content: &str,
        page_content: &str,
    ) -> Result<String> {
        // Strip @layout directive if present (shouldn't normally be here, but just in case)
        let clean_page_content = self.strip_layout_directive(page_content);

        // Extract slots from page (before rendering)
        let slots = self.extract_slots(&clean_page_content);

        // Extract and process layout HTML WITHOUT interpolations yet
        let layout_html_raw = self.extract_html(layout_content);
        let layout_processed = self.process_directives(&layout_html_raw);

        // Render page HTML fully (with interpolations)
        let page_html = self.render(&clean_page_content)?;

        // Replace {slots.content} with page HTML
        let mut result = layout_processed.replace("{slots.content}", &page_html);

        // Replace slot placeholders
        // Pattern 1: {slots.get("key").unwrap_or("default")}
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

        // NOW process interpolations on the final result
        result = self.process_interpolations(&result);

        // Inject collected CSS into the <head>
        result = self.inject_css(&result);

        Ok(result)
    }

    /// Inject collected CSS into the HTML <head>
    fn inject_css(&self, html: &str) -> String {
        if self.collected_css.is_empty() {
            return html.to_string();
        }

        // Combine all collected CSS
        let combined_css = self
            .collected_css
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        // Create a <style> tag with the scoped CSS
        let style_tag = format!("<style data-rhtml-scoped>\n{}\n</style>", combined_css);

        // Try to inject into <head> before </head> tag
        if let Some(head_close) = html.find("</head>") {
            let mut result = html.to_string();
            result.insert_str(head_close, &style_tag);
            result.insert(head_close, '\n');
            return result;
        }

        // If no </head> found, try to inject after <head>
        if let Some(head_open) = html.find("<head>") {
            let insert_pos = head_open + 6; // Length of "<head>"
            let mut result = html.to_string();
            result.insert(insert_pos, '\n');
            result.insert_str(insert_pos + 1, &style_tag);
            return result;
        }

        // If no <head> found, return as-is
        html.to_string()
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
