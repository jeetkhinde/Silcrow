# JavaScript API

The `window.Silcrow` object serves as the public interface for the runtime. While Silcrow's primary philosophy relies on HTML attributes, this API provides programmatical escape hatches for complex logic.

---

## 1. Runtime Binding

### `Silcrow.patch(data, root, options?)`
**Why to use:** To manually inject a JSON logic update into the DOM.
**When to use:** You might receive data from an external library (like a map callback or a third-party SDK) and want to bind its results into the DOM.
**How to use:**
```javascript
Silcrow.patch({ user: { score: 100 } }, "#score-badge");
```

### `Silcrow.invalidate(root)`
**Why to use:** To clear internal binding caches for a DOM tree.
**When to use:** If you manually modify the DOM (e.g., adding classes or appending new HTML nodes with Vanilla JS) and need Silcrow to re-scan the new tree for `:` binding attributes.
**How to use:**
```javascript
Silcrow.invalidate(document.body);
```

### `Silcrow.stream(root)`
**Why to use:** Returns a microtask-batched version of `patch()`.
**When to use:** Ideal for high-frequency updates (e.g. mouse tracking, constant websocket messages) so that Silcrow only updates the DOM once per animation frame, taking the absolute latest data.
**How to use:**
```javascript
const updateCard = Silcrow.stream("#card");
updateCard({ x: 1 });
updateCard({ x: 2 }); // Only this payload triggers a DOM mutation
```

---

## 2. Navigation

### `Silcrow.go(path, options?)`
**Why to use:** To programmatically execute a client-side route or mutation.
**When to use:** You want to trigger a save or navigation programmatically, perhaps following a `setTimeout` or an internal game-engine event, rather than a user click.
**How to use:**
```javascript
Silcrow.go("/api/checkout", { 
  method: "POST", 
  body: JSON.stringify({ items: [] }),
  target: "#cart"
});
```

---

## 3. Real-time (Live)

### `Silcrow.live(root, url)`
**Why to use:** To programmatically open an SSE connection.
**When to use:** You don't want to use the declarative `s-sse` attribute in the DOM, but want to strictly control when a user connects to a stream from JavaScript.
**How to use:**
```javascript
Silcrow.live(document.body, "/api/stream");
```

### `Silcrow.send(data, root)`
**Why to use:** To send data over an active WebSocket to the server.
**When to use:** Used to push events in bidirectional apps (e.g. typing indicators, chat messages) into a WebSocket connection previously opened by `s-ws`.
**How to use:**
```javascript
Silcrow.send({ type: "typing", user: "Alice" }, document.body);
```

### `Silcrow.disconnect(root)` & `Silcrow.reconnect(root)`
**Why to use:** To pause and resume live connections explicitly.
**When to use:** You want to disconnect a stream manually when the user backgrounds the tab, or reconnect when they return, to save on resources.
**How to use:**
```javascript
Silcrow.disconnect("#chat-pane");
// ... later
Silcrow.reconnect("#chat-pane");
```

---

## 4. Feedback & Systems

### `Silcrow.optimistic(data, root)` & `Silcrow.revert(root)`
**Why to use:** To immediately show a UI state while waiting for the server.
**When to use:** When you have a slow API endpoint (e.g., toggling a "Like" button), use `optimistic` to patch the counter instantly. If the network fails, use `revert` to restore the pre-patched DOM snapshot.
**How to use:**
```javascript
// Snapshot is taken and UI updates instantly
Silcrow.optimistic({ liked: true }, "#post"); 

fetch('/api/like', ...).catch(() => {
  Silcrow.revert("#post"); // Undo if it fails
});
```

### `Silcrow.onToast(handler)`
**Why to use:** Plugs your UI notification library into Silcrow.
**When to use:** Run once on page load to map server-sent toasts into your visualization tool of choice (e.g., Toastify, sweetalert).
```javascript
Silcrow.onToast((message, level) => {
  MyNotificationLibrary.show({ text: message, variant: level });
});
```

### `Silcrow.use(fn)`
**Why to use:** To register global middleware that transforms data before it reaches the DOM.
**When to use:** Formatting dates, currency, or translating localization strings across the entire application immediately before they hit a binding. Note: must be called before page load.
**How to use:**
```javascript
Silcrow.use((data) => {
  if (data.timestamp) data.timestamp = new Date(data.timestamp).toLocaleString();
  return data;
});
```

### `Silcrow.onRoute(handler)` & `Silcrow.onError(handler)`
**Why to use:** To intercept navigating and error operations globally.
**When to use:** Hook in custom authorization logic, or send crash analytics to an error tracking service.
**How to use:**
```javascript
Silcrow.onError((err, ctx) => { Sentry.captureException(err); });
```

### `Silcrow.destroy()`
**Why to use:** To completely dismantle Silcrow.
**When to use:** Used strictly if you are unmounting Silcrow in a complex Micro-Frontend architecture. It removes all event listeners, clears caches, and terminates active SSE/WS connections.
**How to use:**
```javascript
Silcrow.destroy();
```
