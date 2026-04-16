# HTML Attributes and Keywords

Silcrow is fundamentally driven by custom HTML attributes, allowing you to declare routing, data-binding, and real-time connectivity directly in your templates.

---

## 1. Data Binding & Templates

### `s-use`
**Why to use:** To bind an entire JavaScript object to an element, spreading its properties automatically.
**When to use:** When you have a complex UI component (like a card or a toast) and receive a full data object from the server, instead of binding every specific `:class`, `:text`, etc. individually.
**How to use:**
```html
<div class="user-card" s-use="user"></div>
```
*(If `user` evaluates to `{name: "Alice", email: "alice@example.com"}`, it binds `:name` and `:email` under the hood).*

### `: (Colon Shorthands)`
**Why to use:** To create granular reactive data paths on any DOM element.
**When to use:** When you want specific DOM properties (like text content, visibility, or class names) to automatically update when Silcrow patches new data. Supported shorthands: `:text`, `:class`, `:style`, `:show`, `:value`, `:disabled`, `:hidden`.
**How to use:**
```html
<h1 :text="user.name"></h1>
<button :disabled="user.isBanned">Submit</button>
```

### `template[s-for]`
**Why to use:** To render collections and lists dynamically from data arrays or objects.
**When to use:** For tables, list items, or any repeating fragment of UI. It supports multi-sibling tags (e.g. `dt`/`dd`) without wrapper divs.
**How to use:**
Place the `s-for` attribute on a `<template>` inside a container element:
```html
<ul>
  <template s-for="user in users" :key="user.id">
    <li :text="user.name"></li>
  </template>
</ul>
```

### `:key`
**Why to use:** To provide stable identity to items in an `s-for` loop, preventing unnecessary DOM destruction and rebuilding.
**When to use:** Always use this on your `<template>` element when rendering lists. It also acts as a placeholder string `/:key/` inside verb attributes to let buttons automatically resolve their context ID.
**How to use:**
```html
<template s-for="task in tasks" :key="task.uuid">...
```

---

## 2. Navigation & Fetching

### `s-get`, `s-post`, `s-put`, `s-patch`, `s-delete`
**Why to use:** To fetch a URL on interaction and mutate the DOM with the response. The attribute name declares the HTTP method.
**When to use:** Instead of standard `<a href>` links or traditional `<form action>` attributes for a Single-Page App feel.
**How to use:**
```html
<a s-get="/dashboard">Dashboard</a>
<!-- With implicit :key interpolation: -->
<button s-post="/api/tasks/:key/complete">✓</button>
<button s-delete="/items/:key" s-target="#notifications">Remove</button>
```

### `s-target`
**Why to use:** To specify exactly which part of the DOM should be updated with the server's response.
**When to use:** When the response shouldn't replace the entire page, but just a specific widget, container, or layout area.
**How to use:**
Provide a valid CSS selector:
```html
<button s-get="/stats" s-target="#stats-panel">Load Stats</button>
```

### `s-html`
**Why to use:** To force the request to ask for HTML rather than JSON.
**When to use:** When your endpoint can return both, but this specific interaction requires the HTML string representation.
**How to use:**
```html
<button s-get="/reports/1" s-html s-target="#view">View Draft</button>
```

### `s-skip-history`
**Why to use:** To prevent full-page GET navigations from pushing an entry to the browser's history stack.
**When to use:** For transient views like tabs or filters that you don't want the user to navigate back into using the browser's Back button.
**How to use:**
```html
<a s-get="/dashboard?tab=analytics" s-skip-history>Analytics</a>
```

### `s-preload`
**Why to use:** To fetch the URL silently in the background when the user hovers over the element.
**When to use:** For high-traffic links to make navigation feel instantaneous.
**How to use:**
```html
<a s-get="/heavy-page" s-preload>Load Page</a>
```

### `s-timeout`
**Why to use:** To override Silcrow's default 30,000ms fetch timeout.
**When to use:** For long-running server actions (like AI generation or bulk exports) where 30 seconds is not enough.
**How to use:**
Specify milliseconds:
```html
<button s-post="/generate" s-timeout="60000">Generate Report</button>
```

---

## 3. Live & Debugging

### `s-sse`
**Why to use:** To declaratively open a Server-Sent Events stream when the element loads.
**When to use:** For real-time one-way updates like a live stock ticker or notification bell.
**How to use:**
```html
<div s-sse="/api/notifications/stream"></div>
```

### `s-ws` / `s-wss`
**Why to use:** To declaratively open a WebSocket connection.
**When to use:** For real-time bi-directional communication (chat apps, collaborative cursors).
**How to use:**
```html
<div s-wss="wss://api.example.com/chat"></div>
```

### `s-debug`
**Why to use:** To enable verbose console warnings and halt execution on template validation errors.
**When to use:** Exclusively during local development.
**How to use:**
Place it on the body tag:
```html
<body s-debug>
```

---

## 4. Utility Markers

### `silcrow-loading`
**Why to use:** It is a CSS class added dynamically by Silcrow while a fetch is executing.
**When to use:** To write CSS rules that show loading spinners, dim the screen, or change the cursor.
**How to use:**
```css
.silcrow-loading {
  opacity: 0.5;
  pointer-events: none;
}
```

### `silcrow_toasts`
**Why to use:** It is a cookie read by Silcrow after an offline or HTML response to populate toast events.
**When to use:** Used on your backend. When responding with a full page redirect (where you can't send JSON to `_toasts`), set this cookie with a URL-encoded JSON array.
**How to use:**
Backend (pseudo-code):
```rust
headers.append("Set-Cookie", "silcrow_toasts=%5B%7B%22message%22%3A%22Saved%22%7D%5D; Path=/");
```
