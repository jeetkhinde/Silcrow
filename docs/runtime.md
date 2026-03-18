# **Runtime: Data Binding & DOM Patching**

## **Scalar Binding**

Use colon-prefixed attributes to bind specific properties to data paths. For boolean attributes (like `disabled` or `hidden`), Silcrow removes the attribute entirely if the value is `false`. For text content, use the special :text shorthand. To toggle visibility via CSS, use :show.

```html
<h1 :text="user.name"></h1>
<div :show="user.is_online">Online Now</div>
<div :class="user.status_flags"></div>
<input :value="user.email" />
<button :disabled="user.banned">Action</button>

```

Patch data into the DOM:

```javascript
Silcrow.patch({
  user: { name: "Alice", email: "a@b.com", avatar: "/img/alice.png", banned: false, status_flags: { online: true, admin: false } }
}, "#app");

```

## Spread Binding with s-use

Use the `s-use` directive to spread an object's properties onto an element. This is useful for binding multiple properties at once.

```html
<div s-use="user"></div>
```

This is equivalent to:

```html
<div :name="user.name" :email="user.email" :avatar="user.avatar" :banned="user.banned" :status_flags="user.status_flags"></div>
```

```html
<div class="card" s-use="taskUI"></div> 
```

```javascript
Silcrow.patch({ taskUI: { text: "Fix bug", class: { "is-active": true }, show: true } }, "#app");
```

The second argument is a root — either a CSS selector string or a DOM Element. Silcrow only patches bindings within that root.

**Known properties** are set as DOM properties. All other bindings are set as attributes.

```javascript
// Known properties
const knownProps = {
  value: "string",
  checked: "boolean",
  disabled: "boolean",
  selected: "boolean",
  hidden: "boolean",    
  required: "boolean",  
  readOnly: "boolean",  
  src: "string",
  href: "string",
  selectedIndex: "number",
};
```

**Security:** Binding to event handler attributes (onclick, onload, etc.) is rejected. Text content is set via textContent, never innerHTML.

## **Fragment Loops**

Render collections of objects into a container using the `<template s-for>` directive. Silcrow supports multi-sibling fragments (e.g., dt/dd pairs) without requiring a wrapper div.

```html
<dl>
  <template s-for="task in tasks" :key="task.id">
    <dt :text="task.title"></dt>
    <dd>
      <button s-action="/tasks/:key/done" POST>Complete</button>
    </dd>
  </template>
</dl>

```

### **Identity & Stability**

Silcrow uses **Identity-Locked Reconciliation**:

1. **Explicit Key**: If `:key="task.id"` is provided, Silcrow uses that data field.

2. **Implicit Identity**: If `:key` is omitted or set to `index`, Silcrow uses a `WeakMap` to assign a stable UUID to the object reference. This ensures that if the list reorders, the DOM nodes move instead of being destroyed, preserving `<input>` focus and CSS transitions.

### **Printed Context**

Silcrow "prints" the stable ID as a `:key` attribute on **every sibling** in the rendered block. This allows nested actions to resolve their context automatically.

### **Collection Patching Modes**

Silcrow dispatches logic based on the shape of the data patched to an s-for path:

| Value shape | Mode | Behavior |
| --- | --- | --- |
| Array `[...]` | **Full sync** | Reconciles the entire list (add, remove, reorder). |
| Object `{...}` | **Merge** | Appends or updates a single item based on its `:key`. |
| Object `{_remove: true}` | **Remove** | Deletes the specific item matching the `:key`. |

**Full sync** (initial load, delete, reorder):

```javascript
Silcrow.patch({
  todos: [
    { id: "1", text: "Buy milk", done: false },
    { id: "2", text: "Write docs", done: true },
  ]
}, "#app");

```

**Merge** (create or update a single item — no need to send the full list):

```javascript
Silcrow.patch({
  todos: { id: "3", text: "Ship it", done: false }
}, "#app");
```

The new item is appended; existing items with keys "1" and "2" are untouched. If an item with `id` "3" already exists, it is updated in-place.

**Remove** (delete a single item — no need to send the full list):

```javascript
Silcrow.patch({
  todos: { id: "2", _remove: true }
}, "#app");
```

The item with `id` "2" is removed from the DOM. All other items are untouched. The `_remove` field is a reserved tombstone sentinel — any other fields in the object are ignored.

**Direct targeting:** `s-target` can point directly to the `[s-for]` container:

```html
<form s-action="/todos" POST s-target="#todo-list">...</form>
<ul id="todo-list" s-for="todo in todos" :key="todo.id">...</ul>

```

**Local bindings** use a leading dot (.text, .done) — they bind to fields on the individual item, not the global data object.

**Reconciliation:** Silcrow uses keyed reconciliation. Existing DOM nodes are reused by key, new items are created from the template, and duplicate keys are automatically skipped to prevent UI corruption.

**Template resolution order:**

```text
1. Item key prefix — if key is `special#3`, Silcrow looks for `<template id="special">`
2. `s-template` attribute on the container
3. Inline <template> child of the container

```

**Template rules:** Templates must contain exactly one element child. Scripts and event handler attributes inside templates are rejected during validation.

**Server-Rendered Lists (Hydration):**
Silcrow seamlessly handles collections that are pre-rendered by the server. If an item exists in the DOM with an `s-key` but was not created dynamically via Silcrow's `<template>` cloning, Silcrow will lazily scan and cache its shorthand reactive attributes (e.g., :text, :value) on the first patch. This allows you to serve fully populated HTML on initial load and effortlessly transition to client-side patches.

## **Data Processing Pipeline**

Silcrow processes data through a multi-stage lifecycle before patching the DOM:

1. **Middleware**: Global transformers registered via Silcrow.use().

2. **Toasts**: Automatic extraction and display of server-sent notifications.

3. **Smart Unwrapping**: If the payload is `{ data: X }` with a single key and `X` is a plain object, Silcrow unwraps it to simplify binding paths. Primitives (`{ data: "Loading…" }`) and arrays (`{ data: [...] }`) are never unwrapped — they pass through as-is for direct binding.

4. **Safety Check**: Verification that the final payload is a valid non-null object.

### **Silcrow.use(fn)**

Register a global middleware function to transform data across all patches. **Middleware must be registered before Silcrow initializes** (i.e., before `DOMContentLoaded`). Calls to `Silcrow.use()` after initialization are rejected with a console warning.

Each middleware receives a deep-cloned copy of the data, so mutations inside a middleware cannot affect the original payload or other middleware in the chain.

```javascript
  Silcrow.use((data) => {
    data.lastUpdated = new Date().toLocaleTimeString();
    return data; // Return modified object or new object
  });
```

## **Silcrow.patch(data, root, options?)**

The core patching function. Options:

* invalidate: true — rebuilds the binding map from scratch (use after DOM mutations)
* silent: true — suppresses the silcrow:patched custom event

After each patch, a silcrow:patched event fires on the root with detail.paths listing all bound paths.

## **Silcrow.invalidate(root)**

Clears the cached binding map and template validations for a root. Call this when you've added or removed shorthand binding or s-list elements dynamically.

## **Silcrow.stream(root)**

Returns a microtask-batched update function. Multiple calls within the same microtask are coalesced — only the last data wins.

```javascript
const update = Silcrow.stream("#dashboard");
update({ count: 1 });
update({ count: 2 });
update({ count: 3 }); // only this patch executes

```

## **Path Resolution**

Dot-separated paths resolve into nested objects: `"user.profile.name"` reads `data.user.profile.name`. Prototype pollution paths (`__proto__`, `constructor`, `prototype`) are blocked and return `undefined`.
