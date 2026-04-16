# Details: Events

Silcrow uses standard DOM CustomEvents to coordinate logic. All events bubble and most are dispatched on the `document` (with the exception of `silcrow:patched` which fires directly on the root element being updated).

---

## 1. Events Fired by Silcrow

### `silcrow:navigate`
**Why to use:** To intercept a fetch request before it is sent.
**When to use:** If you need to append a custom authorization header dynamically, or if you want to conditionally cancel the request (e.g. unsaved changes warning).
**How to use:**
```javascript
document.addEventListener("silcrow:navigate", (e) => {
  if (hasUnsavedChanges) e.preventDefault();
  console.log("Navigating to:", e.detail.url);
});
```

### `silcrow:before-swap`
**Why to use:** To pause the DOM mutation after the network request finishes, but before the HTML is swapped or JSON is patched.
**When to use:** For orchestrating CSS exit animations.
**How to use:**
Call `e.detail.proceed()` manually when ready.
```javascript
document.addEventListener("silcrow:before-swap", (e) => {
  const target = document.querySelector(e.detail.target);
  target.classList.add("fade-out");
  setTimeout(() => e.detail.proceed(), 300);
});
```

### `silcrow:load`
**Why to use:** To re-initialize third-party scripts.
**When to use:** After a successful DOM swap (e.g., if you swap in HTML that contains a chart, you need to call `new Chart()` on the new elements).
**How to use:**
```javascript
document.addEventListener("silcrow:load", (e) => {
  initMyCharts(e.detail.target);
});
```

### `silcrow:error`
**Why to use:** To handle global network failures.
**When to use:** For popping up an offline indicator, showing custom timeout messages, or logging to monitoring services.
**How to use:**
```javascript
document.addEventListener("silcrow:error", (e) => {
  alert("Fetch failed for " + e.detail.url);
});
```

### `silcrow:patched`
**Why to use:** To react specifically to JSON data binding updates.
**When to use:** Sent locally on the updated element. Use it inside components to react when their internal state changes.
**How to use:**
```javascript
element.addEventListener("silcrow:patched", (e) => {
  console.log("Paths updated:", e.detail.paths);
});
```

### `silcrow:optimistic` & `silcrow:revert`
**Why to use:** To hook into the optimistic UI lifecycle.
**When to use:** If you need to disable a submit button while an optimistic mutation is pending, and re-enable it if a `silcrow:revert` occurs.
**How to use:**
```javascript
document.addEventListener("silcrow:optimistic", (e) => { /* Pending UI */ });
document.addEventListener("silcrow:revert", (e) => { /* Rollback UI */ });
```

### `silcrow:live:connect` & `silcrow:live:disconnect`
**Why to use:** To show connection status indicators in the UI.
**When to use:** Whenever an SSE or WebSocket stream connects or drops (giving you the exact ms until the next backoff retry).
**How to use:**
```javascript
document.addEventListener("silcrow:live:disconnect", (e) => {
  showToast(`Reconnecting in ${e.detail.reconnectIn / 1000}s...`);
});
```

### `silcrow:sse:[event]` & `silcrow:ws:[event]`
**Why to use:** To dispatch custom envelopes directly from server streams.
**When to use:** If the server sends a named event over SSE (e.g. `event: ping`), Silcrow translates it to `silcrow:sse:ping` on the client.
**How to use:**
```javascript
document.addEventListener("silcrow:sse:custom-alert", (e) => {
  alert(e.detail.message);
});
```

---

## 2. Events Consumed by Silcrow (Global Listeners)

Silcrow listens at the `document` level to automatically intercept standard browser behaviors.

### `click`
Intercepts clicks on elements possessing a verb attribute (`s-get`, `s-post`, `s-put`, `s-patch`, `s-delete`) to perform an AJAX or fetch navigation instead of a standard browser page load.

### `submit`
Intercepts form submissions globally for forms with a verb attribute (`form[s-get]`, `form[s-post]`, etc.), preventing standard request behaviors, serializing the inputs into a `FormData` object, and sending them via `fetch`.

### `mouseenter`
Listens globally to trigger background background prefetching for any elements holding the `s-preload` attribute.

### `popstate`
Listens on the `window` to capture browser Back/Forward navigation. It re-fetches the correct URL and restores the saved scroll position instead of relying purely on the browser's cache.

### `silcrow:sse`
Fired by the networking pipeline (via HTTP Headers) and consumed internally by the Live system to establish an explicit Server-Sent Events stream to the requested URL.
