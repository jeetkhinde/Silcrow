# RHTML Configuration Guide

RHTML is designed to work out-of-the-box with sensible defaults. Configuration is **optional** but available when you need to customize your project structure.

## Quick Start

**RHTML works without any configuration file!** Just create your `pages/` and `components/` directories and start building.

If you need customization, create an `rhtml.toml` file in your project root.

---

## Configuration File

### Location
```
my-project/
├── rhtml.toml          ← Optional configuration file
├── pages/
├── components/
└── Cargo.toml
```

### Minimal Configuration

The simplest `rhtml.toml`:

```toml
# Everything uses defaults - this file is optional!
```

Or customize just what you need:

```toml
[routing]
pages_dir = "app"  # Use Next.js-style structure
```

---

## Routing Configuration

### Directory Structure

**Configure where your pages and components live:**

```toml
[routing]
# Directory containing page files (default: "pages")
pages_dir = "pages"

# Directory containing component files (default: "components")
components_dir = "components"
```

**Examples:**

**Next.js-style:**
```toml
[routing]
pages_dir = "app"
components_dir = "components"
```

**Hugo-style:**
```toml
[routing]
pages_dir = "content"
components_dir = "layouts"
```

**Custom:**
```toml
[routing]
pages_dir = "routes"
components_dir = "ui"
```

### Case Sensitivity

**Control URL matching behavior:**

```toml
[routing]
# Default: true (case-insensitive)
# /about, /About, /ABOUT all match the same route
case_insensitive = true
```

**Case-sensitive routing:**
```toml
[routing]
case_insensitive = false
# Now /about and /About are different routes
```

**Why case-insensitive by default?**
- More user-friendly (users don't have to remember exact casing)
- Prevents duplicate content issues (SEO)
- Matches browser behavior expectations
- Most modern web frameworks default to this

---

## Complete Configuration Reference

### Full Example

```toml
[project]
name = "my-rhtml-app"
version = "0.1.0"
author = "Your Name"

[server]
port = 3000
host = "127.0.0.1"
workers = 4

[routing]
pages_dir = "pages"
components_dir = "components"
case_insensitive = true
# base_path = "/api"           # Optional: mount under /api
trailing_slash = false

[build]
output_dir = "dist"
static_dir = "static"
minify_html = false
minify_css = false

[dev]
hot_reload = true
port = 3000
open_browser = false
watch_paths = ["pages", "components", "static"]
```

---

## Configuration Sections

### [project]
Project metadata (informational only)

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `name` | String | "rhtml-app" | Project name |
| `version` | String | "0.1.0" | Project version |
| `author` | String | None | Author name (optional) |

### [server]
Server runtime configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `port` | Number | 3000 | Server port |
| `host` | String | "127.0.0.1" | Server host |
| `workers` | Number | 4 | Worker thread count |

### [routing]
**File structure and route behavior**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `pages_dir` | String | "pages" | Directory for page files |
| `components_dir` | String | "components" | Directory for component files |
| `case_insensitive` | Boolean | **true** | Case-insensitive URL matching |
| `base_path` | String | None | Base path prefix for all routes |
| `trailing_slash` | Boolean | false | Enforce trailing slashes |

### [build]
Production build settings (future)

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `output_dir` | String | "dist" | Build output directory |
| `static_dir` | String | "static" | Static assets directory |
| `minify_html` | Boolean | false | Minify HTML output |
| `minify_css` | Boolean | false | Minify CSS output |

### [dev]
Development server settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `hot_reload` | Boolean | true | Enable hot reload |
| `port` | Number | 3000 | Dev server port |
| `open_browser` | Boolean | false | Auto-open browser |
| `watch_paths` | Array | ["pages", "components", "static"] | Paths to watch |

---

## Common Patterns

### Next.js Migration

Coming from Next.js? Use this structure:

```toml
[routing]
pages_dir = "app"
components_dir = "components"
case_insensitive = true
```

Your project structure:
```
my-project/
├── app/
│   ├── page.rhtml        (instead of pages/index.rhtml)
│   └── users/
│       └── page.rhtml
└── components/
```

### Hugo Migration

Coming from Hugo? Use this structure:

```toml
[routing]
pages_dir = "content"
components_dir = "layouts"
```

### Monorepo Setup

Multiple apps in one repo:

```toml
# App 1: frontend
[routing]
pages_dir = "apps/frontend/pages"
components_dir = "shared/components"

# App 2: admin
# Use a separate rhtml.toml in apps/admin/
```

---

## Environment Overrides

Some settings can be overridden via environment variables:

```bash
# Disable hot reload
HOT_RELOAD=false cargo run

# Change port
PORT=8080 cargo run
```

---

## Configuration Loading

RHTML loads configuration in this order:

1. **Check for `rhtml.toml`** in project root
2. **If not found or empty** → Use defaults
3. **Merge with defaults** → Missing values use defaults
4. **Apply environment overrides**

This means:
- ✅ No config file needed - just works
- ✅ Partial config supported - only override what you need
- ✅ Defaults are sensible for most projects

---

## Testing Configuration

**Check what configuration is active:**

```bash
cargo run
```

Output shows:
```
⚙️  Configuration:
   - Port: 3000
   - Pages directory: pages
   - Components directory: components
   - Case-insensitive routing: true
```

**Test with custom config:**

Create `rhtml.toml`:
```toml
[routing]
pages_dir = "app"
```

Run again:
```
⚙️  Configuration:
   - Pages directory: app        ← Changed!
   - Components directory: components
   - Case-insensitive routing: true
```

---

## FAQ

### Do I need a config file?

**No!** RHTML works perfectly with defaults. Only create `rhtml.toml` if you need to customize.

### Can I use both `pages/` and `app/` directories?

**No.** Choose one via `pages_dir` setting. This keeps your structure clear and predictable.

### What happens if `pages_dir` doesn't exist?

RHTML will fail with a clear error message on startup. Create the directory or fix the config.

### Can I change directories after creating routes?

**Yes**, but you'll need to:
1. Move your files to the new directory
2. Update `rhtml.toml`
3. Restart the server

### Are there performance implications for case-insensitive routing?

**No.** The performance difference is negligible (microseconds). Case-insensitive matching uses optimized string comparison.

### Can I have case-sensitive and case-insensitive routes in the same app?

**No.** The setting is app-wide. This ensures consistent behavior across your entire application.

---

## Migration Guide

### From Existing RHTML Project

If you're using the old structure (before v0.2.0):

**Nothing changes!** Your project works exactly as before. The defaults match the old behavior except:

⚠️ **Case-insensitive routing is now the default** (was case-sensitive before)

If you need the old case-sensitive behavior:

```toml
[routing]
case_insensitive = false
```

### From Next.js

1. Rename `pages/` to `app/` (or keep `pages/`)
2. Create `rhtml.toml`:

```toml
[routing]
pages_dir = "pages"  # or "app"
components_dir = "components"
```

3. Convert `.tsx` files to `.rhtml` syntax

### From Hugo

1. Rename `content/` to `pages/` (or configure it)
2. Create `rhtml.toml`:

```toml
[routing]
pages_dir = "content"
components_dir = "layouts"
```

3. Convert Hugo templates to RHTML syntax

---

## Best Practices

### Start Simple
```toml
# Don't configure unless you need to
# Default structure works for 90% of projects
```

### Be Explicit
```toml
# If you customize, comment why
[routing]
pages_dir = "app"     # Using Next.js style for team familiarity
```

### Keep It Minimal
```toml
# Only override what you actually need to change
[routing]
pages_dir = "app"
# Don't specify case_insensitive = true (it's already the default)
```

### Document Your Structure
```
# In your README.md
## Project Structure
- `app/` - Page routes (configured via rhtml.toml)
- `components/` - Reusable components
- `static/` - Static assets
```

---

## Troubleshooting

### "Failed to load templates" error

**Cause:** `pages_dir` doesn't exist

**Fix:**
```bash
mkdir pages  # or whatever your pages_dir is set to
```

### Routes not matching (case issues)

**Cause:** `case_insensitive` setting

**Fix:**
```toml
[routing]
case_insensitive = true  # or false, depending on what you need
```

### Components not found

**Cause:** `components_dir` is wrong

**Fix:** Check the directory name in your config matches the actual directory:
```bash
ls -la components/  # Does this directory exist?
```

### Changes not taking effect

**Cause:** Server running with old config

**Fix:** Restart the server:
```bash
# Ctrl+C to stop, then
cargo run
```

---

## What's Not Configurable (And Why)

Some things are intentionally not configurable to keep RHTML simple:

### File Extensions
- ✗ Template extension is always `.rhtml`
- **Why:** Keeps tooling simple, editor support consistent

### Special Files
- ✗ Layout files are always `_layout.rhtml`
- ✗ Error pages are always `_error.rhtml`
- **Why:** Conventions make code predictable across projects

### Route Syntax
- ✗ Dynamic segments are always `[param]`
- ✗ Catch-all is always `[...slug]`
- **Why:** Standard syntax improves learning curve

---

## Related Documentation

- [File-Based Routing](DYNAMIC_ROUTING.md) - How routing works
- [rhtml.toml Reference](rhtml.toml) - Annotated config example
- [Migration Guide](README.md#migration) - Migrating from other frameworks

---

**Remember:** Configuration is optional. Start with defaults, configure only when needed!
