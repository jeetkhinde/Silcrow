// File: rhtml-macro/src/html.rs
// Purpose: Implementation of the html! macro for compile-time HTML generation

use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{Result as SynResult, Error as SynError};

/// HTML element node
#[derive(Debug, Clone)]
pub struct Element {
    pub tag: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    pub self_closing: bool,
}

/// HTML attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

/// Attribute value (static string or dynamic expression)
#[derive(Debug, Clone)]
pub enum AttributeValue {
    Static(String),
    Dynamic(TokenStream),
}

/// HTML node (element, text, or expression)
#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(String),
    Expression(TokenStream),
}

/// Parser for html! macro input
pub struct HtmlParser {
    input: String,
    pos: usize,
}

impl HtmlParser {
    pub fn new(input: String) -> Self {
        Self { input, pos: 0 }
    }

    /// Parse the entire HTML input
    pub fn parse(&mut self) -> SynResult<Vec<Node>> {
        let mut nodes = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace();

            if self.is_eof() {
                break;
            }

            if self.peek_char() == Some('<') {
                nodes.push(self.parse_element()?);
            } else if self.peek_char() == Some('{') {
                nodes.push(self.parse_expression()?);
            } else {
                nodes.push(self.parse_text()?);
            }
        }

        Ok(nodes)
    }

    /// Parse an HTML element
    fn parse_element(&mut self) -> SynResult<Node> {
        self.consume_char('<')?;

        // Check for closing tag
        if self.peek_char() == Some('/') {
            return Err(SynError::new(
                Span::call_site(),
                "Unexpected closing tag",
            ));
        }

        // Parse tag name
        let tag = self.parse_identifier()?;

        // Parse attributes
        let mut attributes = Vec::new();
        loop {
            self.skip_whitespace();

            if self.peek_char() == Some('>') ||
               (self.peek_char() == Some('/') && self.peek_ahead(1) == Some('>')) {
                break;
            }

            attributes.push(self.parse_attribute()?);
        }

        // Check for self-closing tag
        let self_closing = if self.peek_char() == Some('/') {
            self.consume_char('/')?;
            true
        } else {
            false
        };

        self.consume_char('>')?;

        // Parse children if not self-closing
        let children = if self_closing {
            Vec::new()
        } else {
            self.parse_children(&tag)?
        };

        Ok(Node::Element(Element {
            tag,
            attributes,
            children,
            self_closing,
        }))
    }

    /// Parse element children
    fn parse_children(&mut self, parent_tag: &str) -> SynResult<Vec<Node>> {
        let mut children = Vec::new();

        loop {
            self.skip_whitespace();

            // Check for closing tag
            if self.peek_char() == Some('<') && self.peek_ahead(1) == Some('/') {
                self.consume_char('<')?;
                self.consume_char('/')?;
                let closing_tag = self.parse_identifier()?;

                if closing_tag != parent_tag {
                    return Err(SynError::new(
                        Span::call_site(),
                        format!("Mismatched closing tag: expected </{}>, got </{}>", parent_tag, closing_tag),
                    ));
                }

                self.skip_whitespace();
                self.consume_char('>')?;
                break;
            }

            if self.peek_char() == Some('<') {
                children.push(self.parse_element()?);
            } else if self.peek_char() == Some('{') {
                children.push(self.parse_expression()?);
            } else if !self.is_eof() {
                children.push(self.parse_text()?);
            } else {
                return Err(SynError::new(
                    Span::call_site(),
                    format!("Unclosed tag: <{}>", parent_tag),
                ));
            }
        }

        Ok(children)
    }

    /// Parse an attribute
    fn parse_attribute(&mut self) -> SynResult<Attribute> {
        let name = self.parse_identifier()?;

        self.skip_whitespace();

        if self.peek_char() != Some('=') {
            // Boolean attribute
            return Ok(Attribute {
                name,
                value: AttributeValue::Static(String::new()),
            });
        }

        self.consume_char('=')?;
        self.skip_whitespace();

        // Parse value
        let value = if self.peek_char() == Some('"') {
            self.consume_char('"')?;
            let val = self.parse_until('"')?;
            self.consume_char('"')?;
            AttributeValue::Static(val)
        } else if self.peek_char() == Some('{') {
            self.consume_char('{')?;
            let expr = self.parse_until_balanced('}')?;
            self.consume_char('}')?;
            AttributeValue::Dynamic(expr.parse().map_err(|_| {
                SynError::new(Span::call_site(), "Invalid expression")
            })?)
        } else {
            return Err(SynError::new(
                Span::call_site(),
                "Expected '\"' or '{' after '='",
            ));
        };

        Ok(Attribute { name, value })
    }

    /// Parse text content
    fn parse_text(&mut self) -> SynResult<Node> {
        let mut text = String::new();

        while !self.is_eof() {
            let ch = self.peek_char();

            if ch == Some('<') || ch == Some('{') {
                break;
            }

            text.push(self.next_char().unwrap());
        }

        Ok(Node::Text(text.trim().to_string()))
    }

    /// Parse an expression {expr}
    fn parse_expression(&mut self) -> SynResult<Node> {
        self.consume_char('{')?;
        let expr = self.parse_until_balanced('}')?;
        self.consume_char('}')?;

        let tokens: TokenStream = expr.parse().map_err(|_| {
            SynError::new(Span::call_site(), "Invalid expression")
        })?;

        Ok(Node::Expression(tokens))
    }

    /// Parse an identifier (tag name or attribute name)
    fn parse_identifier(&mut self) -> SynResult<String> {
        let mut ident = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == ':' {
                ident.push(self.next_char().unwrap());
            } else {
                break;
            }
        }

        if ident.is_empty() {
            return Err(SynError::new(Span::call_site(), "Expected identifier"));
        }

        Ok(ident)
    }

    /// Parse until a specific character
    fn parse_until(&mut self, delimiter: char) -> SynResult<String> {
        let mut result = String::new();

        while !self.is_eof() {
            if self.peek_char() == Some(delimiter) {
                break;
            }

            result.push(self.next_char().unwrap());
        }

        Ok(result)
    }

    /// Parse until balanced delimiter (handles nested braces/brackets)
    fn parse_until_balanced(&mut self, closing: char) -> SynResult<String> {
        let opening = match closing {
            '}' => '{',
            ')' => '(',
            ']' => '[',
            _ => return self.parse_until(closing),
        };

        let mut result = String::new();
        let mut depth = 0;

        while !self.is_eof() {
            let ch = self.peek_char().unwrap();

            if ch == opening {
                depth += 1;
                result.push(self.next_char().unwrap());
            } else if ch == closing {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                result.push(self.next_char().unwrap());
            } else if ch == '"' || ch == '\'' {
                // Handle strings (don't count braces inside strings)
                let quote = ch;
                result.push(self.next_char().unwrap());

                while !self.is_eof() && self.peek_char() != Some(quote) {
                    if self.peek_char() == Some('\\') {
                        result.push(self.next_char().unwrap()); // backslash
                        if !self.is_eof() {
                            result.push(self.next_char().unwrap()); // escaped char
                        }
                    } else {
                        result.push(self.next_char().unwrap());
                    }
                }

                if !self.is_eof() {
                    result.push(self.next_char().unwrap()); // closing quote
                }
            } else {
                result.push(self.next_char().unwrap());
            }
        }

        Ok(result)
    }

    /// Consume a specific character
    fn consume_char(&mut self, expected: char) -> SynResult<()> {
        match self.next_char() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(SynError::new(
                Span::call_site(),
                format!("Expected '{}', got '{}'", expected, ch),
            )),
            None => Err(SynError::new(Span::call_site(), "Unexpected end of input")),
        }
    }

    /// Peek at the current character
    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Peek ahead n characters
    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.input[self.pos..].chars().nth(n)
    }

    /// Advance to the next character
    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    /// Check if at end of input
    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

/// Code generator for HTML nodes
pub struct CodeGenerator;

impl CodeGenerator {
    /// Generate Rust code from parsed nodes
    pub fn generate(nodes: Vec<Node>) -> TokenStream {
        let mut statements = quote! {};

        for node in nodes {
            statements.extend(Self::generate_node(&node));
        }

        // Use the correct path depending on whether we're inside the rhtmx crate
        // When used inside rhtmx crate, use crate::Html
        // When used outside, use rhtmx::Html
        let html_path = if std::env::var("CARGO_CRATE_NAME").ok().as_deref() == Some("rhtmx") {
            quote! { crate::Html }
        } else {
            quote! { rhtmx::Html }
        };

        // Wrap in a block expression so it can be used in let bindings
        quote! {
            {
                let mut __html = String::new();
                #statements
                #html_path(__html)
            }
        }
    }

    /// Generate code for a single node
    fn generate_node(node: &Node) -> TokenStream {
        match node {
            Node::Element(el) => Self::generate_element(el),
            Node::Text(text) => {
                quote! {
                    __html.push_str(#text);
                }
            }
            Node::Expression(expr) => {
                quote! {
                    __html.push_str(&format!("{}", #expr));
                }
            }
        }
    }

    /// Generate code for an element
    fn generate_element(element: &Element) -> TokenStream {
        // Check for r-directives
        if let Some(directive_code) = Self::check_directives(element) {
            return directive_code;
        }

        let tag = &element.tag;
        let mut output = quote! {
            __html.push_str("<");
            __html.push_str(#tag);
        };

        // Add attributes
        for attr in &element.attributes {
            output.extend(Self::generate_attribute(attr));
        }

        if element.self_closing {
            output.extend(quote! {
                __html.push_str(" />");
            });
        } else {
            output.extend(quote! {
                __html.push_str(">");
            });

            // Add children
            for child in &element.children {
                output.extend(Self::generate_node(child));
            }

            // Closing tag
            output.extend(quote! {
                __html.push_str("</");
                __html.push_str(#tag);
                __html.push_str(">");
            });
        }

        output
    }

    /// Generate code for an attribute
    fn generate_attribute(attr: &Attribute) -> TokenStream {
        let name = &attr.name;

        match &attr.value {
            AttributeValue::Static(val) => {
                if val.is_empty() {
                    // Boolean attribute
                    quote! {
                        __html.push_str(" ");
                        __html.push_str(#name);
                    }
                } else {
                    quote! {
                        __html.push_str(" ");
                        __html.push_str(#name);
                        __html.push_str("=\"");
                        __html.push_str(#val);
                        __html.push_str("\"");
                    }
                }
            }
            AttributeValue::Dynamic(expr) => {
                quote! {
                    __html.push_str(" ");
                    __html.push_str(#name);
                    __html.push_str("=\"");
                    __html.push_str(&format!("{}", #expr));
                    __html.push_str("\"");
                }
            }
        }
    }

    /// Check for and handle r-directives
    fn check_directives(element: &Element) -> Option<TokenStream> {
        // Check for r-match (pattern matching)
        if let Some(r_match) = element.attributes.iter().find(|a| a.name == "r-match") {
            return Some(Self::generate_r_match(element, r_match));
        }

        // Check for r-for
        if let Some(r_for) = element.attributes.iter().find(|a| a.name == "r-for") {
            return Some(Self::generate_r_for(element, r_for));
        }

        // Check for r-if
        if let Some(r_if) = element.attributes.iter().find(|a| a.name == "r-if") {
            return Some(Self::generate_r_if(element, r_if));
        }

        // Check for r-else-if (standalone - usually follows r-if)
        if let Some(r_else_if) = element.attributes.iter().find(|a| a.name == "r-else-if") {
            return Some(Self::generate_r_else_if(element, r_else_if));
        }

        // Check for r-else (standalone)
        if element.attributes.iter().any(|a| a.name == "r-else") {
            return Some(Self::generate_r_else(element));
        }

        None
    }

    /// Generate code for r-for directive
    fn generate_r_for(element: &Element, r_for_attr: &Attribute) -> TokenStream {
        let for_expr = match &r_for_attr.value {
            AttributeValue::Static(s) => s,
            _ => return quote! {},
        };

        // Parse "item in items" or "(index, item) in items"
        let parts: Vec<&str> = for_expr.split(" in ").collect();
        if parts.len() != 2 {
            return quote! {};
        }

        let var_part = parts[0].trim();
        let collection_str = parts[1].trim();
        let collection: TokenStream = collection_str.parse().unwrap_or_default();

        // Create element without r-for attribute
        let mut clean_element = element.clone();
        clean_element.attributes.retain(|a| a.name != "r-for");

        let element_code = Self::generate_element(&clean_element);

        // Check if it's (index, item) or just item
        if var_part.starts_with('(') && var_part.ends_with(')') {
            // Parse (index, item)
            let inner = &var_part[1..var_part.len() - 1];
            let vars: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

            if vars.len() == 2 {
                let index_var: TokenStream = vars[0].parse().unwrap_or_default();
                let item_var: TokenStream = vars[1].parse().unwrap_or_default();

                return quote! {
                    for (#index_var, #item_var) in (#collection).into_iter().enumerate() {
                        #element_code
                    }
                };
            }
        }

        // Simple "item in items"
        let item_var: TokenStream = var_part.parse().unwrap_or_default();

        quote! {
            for #item_var in #collection {
                #element_code
            }
        }
    }

    /// Generate code for r-if directive
    fn generate_r_if(element: &Element, r_if_attr: &Attribute) -> TokenStream {
        let condition = match &r_if_attr.value {
            AttributeValue::Static(s) => s.parse::<TokenStream>().unwrap_or_default(),
            AttributeValue::Dynamic(expr) => expr.clone(),
        };

        // Create element without r-if attribute
        let mut clean_element = element.clone();
        clean_element.attributes.retain(|a| a.name != "r-if");

        let element_code = Self::generate_element(&clean_element);

        quote! {
            if #condition {
                #element_code
            }
        }
    }

    /// Generate code for r-else-if directive
    fn generate_r_else_if(element: &Element, r_else_if_attr: &Attribute) -> TokenStream {
        let condition = match &r_else_if_attr.value {
            AttributeValue::Static(s) => s.parse::<TokenStream>().unwrap_or_default(),
            AttributeValue::Dynamic(expr) => expr.clone(),
        };

        // Create element without r-else-if attribute
        let mut clean_element = element.clone();
        clean_element.attributes.retain(|a| a.name != "r-else-if");

        let element_code = Self::generate_element(&clean_element);

        quote! {
            else if #condition {
                #element_code
            }
        }
    }

    /// Generate code for r-else directive
    fn generate_r_else(element: &Element) -> TokenStream {
        // Create element without r-else attribute
        let mut clean_element = element.clone();
        clean_element.attributes.retain(|a| a.name != "r-else");

        let element_code = Self::generate_element(&clean_element);

        quote! {
            else {
                #element_code
            }
        }
    }

    /// Generate code for r-match directive (pattern matching)
    ///
    /// Example usage:
    /// ```ignore
    /// <div r-match="status">
    ///     <div r-when="\"active\"">Active</div>
    ///     <div r-when="\"pending\"">Pending</div>
    ///     <div r-default>Unknown</div>
    /// </div>
    /// ```
    fn generate_r_match(element: &Element, r_match_attr: &Attribute) -> TokenStream {
        let match_expr = match &r_match_attr.value {
            AttributeValue::Static(s) => s.parse::<TokenStream>().unwrap_or_default(),
            AttributeValue::Dynamic(expr) => expr.clone(),
        };

        // Process children to find r-when and r-default
        let mut match_arms = Vec::new();
        let mut has_default = false;

        for child in &element.children {
            if let Node::Element(child_el) = child {
                // Check for r-when
                if let Some(r_when) = child_el.attributes.iter().find(|a| a.name == "r-when") {
                    let pattern = match &r_when.value {
                        AttributeValue::Static(s) => s.parse::<TokenStream>().unwrap_or_default(),
                        AttributeValue::Dynamic(expr) => expr.clone(),
                    };

                    let mut clean_child = child_el.clone();
                    clean_child.attributes.retain(|a| a.name != "r-when");
                    let child_code = Self::generate_element(&clean_child);

                    match_arms.push(quote! {
                        #pattern => {
                            #child_code
                        }
                    });
                }
                // Check for r-default
                else if child_el.attributes.iter().any(|a| a.name == "r-default") {
                    let mut clean_child = child_el.clone();
                    clean_child.attributes.retain(|a| a.name != "r-default");
                    let child_code = Self::generate_element(&clean_child);

                    match_arms.push(quote! {
                        _ => {
                            #child_code
                        }
                    });
                    has_default = true;
                }
            }
        }

        // If no default, add empty default arm
        if !has_default {
            match_arms.push(quote! {
                _ => {}
            });
        }

        quote! {
            match #match_expr {
                #(#match_arms)*
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_element() {
        let mut parser = HtmlParser::new("<div>Hello</div>".to_string());
        let nodes = parser.parse().unwrap();

        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Element(el) => {
                assert_eq!(el.tag, "div");
                assert_eq!(el.children.len(), 1);
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_parse_attributes() {
        let mut parser = HtmlParser::new(r#"<div class="test" id="main"></div>"#.to_string());
        let nodes = parser.parse().unwrap();

        match &nodes[0] {
            Node::Element(el) => {
                assert_eq!(el.attributes.len(), 2);
                assert_eq!(el.attributes[0].name, "class");
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_parse_expression() {
        let mut parser = HtmlParser::new("<div>{user.name}</div>".to_string());
        let nodes = parser.parse().unwrap();

        match &nodes[0] {
            Node::Element(el) => {
                assert_eq!(el.children.len(), 1);
                matches!(&el.children[0], Node::Expression(_));
            }
            _ => panic!("Expected element"),
        }
    }
}
