# HTTP Handlers Documentation

Complete documentation for RHTMX HTTP verb macros and routing system.

## Documentation Files

### 📖 [HTTP_HANDLERS_GUIDE.md](./HTTP_HANDLERS_GUIDE.md) - Complete Guide

Comprehensive documentation covering all aspects of HTTP handlers:
- Overview and key features
- All HTTP verb macros (#[get], #[post], #[put], #[patch], #[delete])
- File-based routing system
- Request handling (type-safe, path params, query params)
- Response builders (Ok, Error, Redirect)
- Common patterns and real-world examples
- Best practices
- Macro definition locations
- Advanced topics

**Best for:** In-depth learning, understanding all features, reference material

---

### 🚀 [HTTP_HANDLERS_QUICK_REF.md](./HTTP_HANDLERS_QUICK_REF.md) - Quick Reference

Quick reference guide with code examples:
- Import statements
- Basic handlers
- Available macros table
- Path and query parameters
- Type-safe requests
- Response builders (Ok, Error, Redirect)
- Return types
- File-based routing
- Common patterns
- Tips and tricks

**Best for:** Quick lookups, copy-paste examples, common patterns

---

### 📊 [HTTP_HANDLERS_SUMMARY.md](./HTTP_HANDLERS_SUMMARY.md) - Implementation Summary

High-level summary of the implementation:
- What was built and why
- Key features overview
- Architecture and design
- Implementation structure (macro definitions, core implementation)
- Benefits and comparison with traditional frameworks
- How it works (compile time expansion)
- What was created (files and structure)
- Highlights and design philosophy

**Best for:** Understanding the system architecture, design decisions, big picture

---

## Quick Links

**Getting Started:**
1. Read the [Quick Reference](./HTTP_HANDLERS_QUICK_REF.md) for immediate examples
2. Check [Common Patterns](./HTTP_HANDLERS_QUICK_REF.md#common-patterns) section
3. Look at examples in `examples/users_crud.rs`

**Deep Dive:**
1. Start with [Overview](./HTTP_HANDLERS_GUIDE.md#overview)
2. Read [File-Based Routing](./HTTP_HANDLERS_GUIDE.md#file-based-routing)
3. Study [Response Builders](./HTTP_HANDLERS_GUIDE.md#response-builders)
4. Review [Best Practices](./HTTP_HANDLERS_GUIDE.md#best-practices)

**Understanding Architecture:**
1. Read [Implementation Summary](./HTTP_HANDLERS_SUMMARY.md)
2. Check [How It Works](./HTTP_HANDLERS_SUMMARY.md#how-it-works)
3. Review [Architecture Benefits](./HTTP_HANDLERS_SUMMARY.md#architecture-benefits)

---

## Common Questions

### Which file should I read?

- **"I just want to write handlers"** → [Quick Reference](./HTTP_HANDLERS_QUICK_REF.md)
- **"I want to understand everything"** → [Complete Guide](./HTTP_HANDLERS_GUIDE.md)
- **"I want to understand why it's designed this way"** → [Summary](./HTTP_HANDLERS_SUMMARY.md)

### Where are the macros defined?

See [Macro Definition Location](./HTTP_HANDLERS_GUIDE.md#macro-definition-location) in the Complete Guide.

- **Definitions:** `rhtmx-macro/src/lib.rs` (lines 159-248)
- **Implementation:** `rhtmx-macro/src/http.rs`

### What HTTP methods are supported?

All standard HTTP methods:
- `get!` - GET
- `post!` - POST
- `put!` - PUT
- `patch!` - PATCH
- `delete!` - DELETE

### How do I handle errors?

Use `Result<OkResponse, ErrorResponse>` return type. See [Error Handling with Result](./HTTP_HANDLERS_GUIDE.md#error-handling-with-result) in the guide.

### How do I update multiple page elements?

Use Out-of-Band (OOB) updates with `.render_oob()`. See [Out-of-Band (OOB) Updates](./HTTP_HANDLERS_GUIDE.md#out-of-band-oob-updates) in the guide.

---

## Related Documentation

- **Layouts**: [docs/LAYOUTS.md](../LAYOUTS.md) - Page layout system
- **Features**: [docs/FEATURES.md](../FEATURES.md) - All RHTMX features
- **HTML Macro**: [docs/html!/](../html!/) - HTML template macro documentation
- **Quick Start**: [QUICKSTART.md](../QUICKSTART.md) - 5-minute getting started guide

---

## Examples

Complete working examples are in `examples/users_crud.rs` showing:
- Basic CRUD operations
- Path parameters
- Type-safe requests
- Response builders
- OOB updates
- Error handling

Run the demo:
```bash
cargo run --example users_crud
```

---

## Implementation Details

### Macro Locations

All macros are implemented as `proc_macro_attribute` in the `rhtmx-macro` crate:

```
rhtmx-macro/
├── src/
│   ├── lib.rs      # Macro definitions (#[get], #[post], etc.)
│   └── http.rs     # Implementation (http_handler function)
```

### Response Builders

Response types are defined in the main `rhtmx` crate:

```
src/
├── lib.rs          # Public API exports
└── html.rs         # Response types (OkResponse, ErrorResponse, etc.)
```

### How Routes Work

1. **File location** determines the base route
   - `pages/users/index.rs` → `/users`
   - `pages/users/[id].rs` → `/users/:id`

2. **HTTP verb macro** determines the method
   - `get!` → GET `/users`
   - `post!` → POST `/users`

3. **Route metadata** is generated at compile time for framework registration

---

## Key Concepts

### Zero Configuration

No router setup needed. The framework automatically discovers routes from:
1. File structure in `pages/` directory
2. HTTP verb macros in handler functions

### Type Safety

Rust compiler validates:
- Function signatures match HTTP handler contract
- Request body deserialization types
- Response builder usage
- All expressions in handler

### Compile-Time Optimization

- Route patterns parsed at compile time
- Metadata modules generated automatically
- HTTP method determination at compile time
- Zero runtime dispatch overhead

---

Happy coding with RHTMX HTTP handlers! 🚀
