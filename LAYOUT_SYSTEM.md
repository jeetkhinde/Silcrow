# RHTML Layout and Slotting System

## Overview

RHTML supports a modern layout and slotting system that provides:

- **Layout wrapping** using `_layout.rhtml` files in the pages directory hierarchy
- **Automatic layout resolution** - nested directories can have their own `_layout.rhtml` files
- **Flexible layout control** with `@layout` directive for custom or no layout
- **Partial rendering** for AJAX requests and dynamic content updates
- **Type-safe slot contracts** (planned for future releases)

**Current Status:** Layout system is implemented and functional. Type-safe slot contracts via `LayoutSlots` structs are infrastructure-ready for future enhancements.

## Architecture

The layout system uses:

1. **File-based layout discovery** - `_layout.rhtml` files define layouts
2. **Template loader** - Discovers and caches layout files
3. **Route matching** - Determines which layout applies to each page
4. **Renderer** - Processes directives and renders layouts with pages
5. **Layout registry** (future) - Will enable compile-time slot validation

### Components

#### 1. Template Loader (`src/template_loader.rs`)

Discovers and manages layout and page templates:

- Scans `pages/` directory recursively
- Loads `_layout.rhtml` files at each directory level
- Caches templates for performance
- Provides hot reload support during development

#### 2. Renderer (`src/renderer.rs`)

Processes templates and renders layouts with pages:

- Parses RHTML directives (`@if`, `@for`, `@match`, `@partial`, `@layout`)
- Evaluates expressions and variables
- Renders layouts with page content
- Extracts and manages scoped CSS
- Supports partial rendering for AJAX requests

#### 3. Layout Registry & Resolver (Future Infrastructure)

Located in `rhtml-macro/src/`:

- `layout_registry.rs` - Will store layout metadata
- `layout_resolver.rs` - Will find `_layout.rhtml` files (ready for enhancement)
- `layout.rs` - `#[layout]` macro (infrastructure for future type-safe slots)
- `slot.rs` - `slot!` macro (infrastructure for future slot system)

Currently these are prepared for future enhancements but not actively used in the rendering pipeline.

## Syntax

### Defining Layouts

Layouts are defined as RHTML template files in the `pages/` directory. The `_layout.rhtml` filename is special - it marks a file as a layout.

**File**: `pages/_layout.rhtml`

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <title>@{page_title.unwrap_or("My App")}</title>
</head>
<body>
  <nav><!-- Navigation --></nav>
  <main>
    @{content}
  </main>
  <footer>© 2024</footer>
</body>
</html>
```

### Using Layouts in Pages

Pages automatically use the nearest layout file found by walking up the directory tree. The page body is automatically inserted into the layout.

**File**: `pages/home.rhtml`

```html
<div class="container">
  <h1>Welcome!</h1>
  <p>This content goes into the layout.</p>
</div>
```

### Controlling Layout Behavior

Use the `@layout` directive to control how a page is rendered:

**No layout (render page only):**
```html
@layout(false)

<div class="standalone">
  <h1>Standalone Page</h1>
</div>
```

**Custom layout:**
```html
@layout("admin/_layout")

<div class="admin-dashboard">
  <h1>Admin Panel</h1>
</div>
```

**Partial rendering (for AJAX):**

Query parameter: `?partial=true` - renders page without layout
Query parameter: `?partial=SectionName` - renders named partial from page

## How It Works

### Processing Flow (Current Implementation)

1. **Route Matching**: Request comes in and is matched to a file-based route
2. **Template Loading**: Template loader retrieves the matching page template
3. **Layout Resolution**: System walks up directory tree to find `_layout.rhtml`
4. **Request Context**: Query params, form data, route params added to renderer state
5. **Directive Parsing**: `@layout` directive checked for special instructions
6. **Rendering**:
   - If `@layout(false)`: render page only
   - If `@layout("custom")`: render with custom layout
   - If normal page + layout found: render page within layout
   - If `?partial=true`: render page without layout
   - If `?partial=Name`: render named partial block only
7. **CSS Collection**: Scoped CSS from layout and page collected and rendered
8. **Response**: HTML sent to browser with LiveReload injection (if enabled)

### Key Features

✅ **File-based Discovery**: Automatic layout inheritance via directory structure
✅ **Flexible Control**: `@layout` directive for per-page customization
✅ **AJAX Support**: Partial rendering via query parameters for HTMX/fetch
✅ **Scoped CSS**: Component styles automatically extracted
✅ **Hot Reload**: Templates update instantly during development
✅ **Type-Safe Macros**: `#[webpage]` and `#[component]` for compile-time checks

## File Structure

```
pages/
├── _layout.rhtml          # Root layout
├── index.rhtml            # Uses slot! for slot values
├── about.rhtml
│
├── users/
│   ├── _layout.rhtml      # Users-specific layout
│   ├── index.rhtml        # Uses users layout
│   ├── new.rhtml
│   └── [id].rhtml
│
└── test/
    ├── _layout.rhtml      # Test layout example
    └── index.rhtml        # Test page example
```

## Implementation Files

### Core Files

1. **`rhtml-macro/src/`**
   - `lib.rs` - Exports `#[layout]`, `slot!`, `#[component]` macros
   - `layout.rs` - Implements `#[layout]` macro
   - `slot.rs` - Implements `slot!` macro
   - `layout_registry.rs` - Layout metadata storage (future)
   - `layout_resolver.rs` - Layout file discovery (future)

2. **`src/renderer.rs`**
   - `find_slots_block()` - Locates slot declarations
   - Slot value extraction and processing

3. **Example Files**
   - `pages/_layout.rhtml` - Root layout with new syntax
   - `pages/users/_layout.rhtml` - Nested layout example
   - `pages/test/_layout.rhtml` - Example layout
   - `pages/test/index.rhtml` - Example page with `slot!`

## Testing

### Running Tests

```bash
# Test the macro compilation
cd rhtml-macro && cargo test

# Test the full application
cargo run
curl http://localhost:3000
curl http://localhost:3000/users
```

### Test Coverage

✅ `slot!` macro compilation
✅ `#[layout]` macro compilation
✅ Layout rendering with slots
✅ Nested layouts
✅ Optional slots with defaults

## Best Practices

### 1. Use Consistent Layout Naming

Keep `_layout.rhtml` as the standard layout filename. This ensures the system finds your layouts automatically.

```
pages/
├── _layout.rhtml          # Root layout
├── home.rhtml
├── users/
│   ├── _layout.rhtml      # Users section layout (overrides root)
│   ├── index.rhtml
│   └── [id].rhtml
```

### 2. Provide Fallback Values

In layouts, use `@if` to provide conditional content and sensible defaults:

```html
<title>@{ page_title.unwrap_or("Default Title") }</title>
<meta name="description" content="@{ meta_description.unwrap_or("My App") }" />
```

### 3. Use `@layout(false)` for Standalone Pages

Pages like login, API responses, or admin dashboards should render without layouts:

```html
@layout(false)

<div class="login-form">
  <!-- Page content -->
</div>
```

### 4. Leverage Partials for AJAX

When building interactive pages, use named partials for AJAX updates:

```html
@partial("User List")
<ul>
  @for user in users
    <li>@{user.name}</li>
  @end
</ul>
@end

@partial("User Detail")
<div class="detail">
  <!-- User detail content -->
</div>
@end
```

Then request updates with: `?partial=User%20List`

## Future Enhancements

1. **Type-Safe Slot System (Planned)**
   - Define slots using `LayoutSlots` struct with `#[layout]` macro
   - Compile-time validation of required vs optional slots
   - Better IDE support with type hints
   - Build on current `layout_registry.rs` and `layout_resolver.rs` infrastructure

2. **Advanced Slot Types**
   - Component slots for composable layouts
   - Function slots for dynamic content generation
   - Slot inheritance and composition

3. **Better Error Messages**
   - Specific compile errors for missing or invalid slots
   - Suggestions for typos in slot names
   - Type mismatch explanations

4. **Layout Caching Optimization**
   - Compile-time layout resolution
   - Static dispatch for known layouts
   - Reduced runtime overhead

## Notes

### Current Implementation
- The `content` variable is automatically filled with the page body
- Nested layouts are resolved by walking up the directory tree
- Use `@layout(false)` to render a page without its default layout
- Use `@layout("path/to/_layout")` to use a custom layout
- Query parameters `?partial=true` or `?partial=Name` control partial rendering

### Future (LayoutSlots System)
- The `LayoutSlots` struct will define explicit slot contracts
- Slot values will be passed through the `slot!` macro
- Use `Option<T>` for optional slots
- Type-safe slot validation at compile time

## Questions & Support

For questions or issues with the layout system:
- Check existing layouts in `pages/` for examples
- Review macro implementations in `rhtml-macro/src/`
- See renderer implementation in `src/renderer.rs`
