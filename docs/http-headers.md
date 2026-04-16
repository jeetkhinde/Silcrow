# HTTP Headers

> **Source Module(s):** [`src/navigator.js`](../src/navigator.js)

Silcrow leverages custom HTTP headers to coordinate UI orchestrations, caching, and state transitions directly from the server, eliminating the need for client-side JavaScript controllers.

---

## 1. Headers Sent to the Server

### `silcrow-target`

**Why to use:** Allows the server to identify whether an incoming request was triggered by Silcrow.js or by a standard browser navigation.

**When to use:** Use this on the backend to dynamically serve either a lightweight HTML fragment (for Silcrow) or a full HTML page layout (for a direct browser hit).

**How to use:**
Silcrow automatically appends this header to every fetch request it makes:

```http
slicrow-target: true
```

### `Accept`

**Why to use:** Instructs the server on the expected response format.

**When to use:** Use this on the backend to determine if you should return JSON data (for data binding) or HTML strings (for direct DOM swaps).

**How to use:**
Silcrow automatically sets this based on the presence of the `s-html` attribute on the triggering element:

```http
Accept: application/json
// OR if the trigger had s-html:
Accept: text/html
```

---

## 2. Headers Received from the Server (During Fetch)

These headers are processed immediately upon receiving the response.

### `silcrow-trigger`

#### **Why to use:**

To programmatically fire a custom DOM event on the client side without writing JavaScript.

#### **When to use:**

When a server action completes (like closing a modal, triggering an analytic event, or resetting a form) and you want other parts of the client UI to react to that event.

#### **How to use:**

Return the header as a JSON string or plain text event name. It will be dispatched on the document.

```http
silcrow-trigger: "modal:close"
// OR with detail payload:
silcrow-trigger: {"task:created": {"id": 123}}
```

### `silcrow-retarget`

**Why to use:** To dynamically override the `s-target` attribute defined on the client.

**When to use:** When an error occurs on the server (e.g. form validation failure) and you want to swap the response into an error container rather than the original target.

**How to use:**
Return a valid CSS selector in the header:

```http
silcrow-retarget: #error-message-container
```

### `silcrow-push`

**Why to use:** To update the URL in the browser's history programmatically from the backend without requiring a full page redirection.

**When to use:** When you complete an update operation and want the URL to reflect the new state of the page (e.g. after saving a draft, push the permalink).

**How to use:**
Provide the new relative or absolute URL:

```http
silcrow-push: /tasks/123
```

### `silcrow-cache`

**Why to use:** To explicitly prevent the client from caching a `GET` response.

**When to use:** When requesting highly volatile data (like a constantly updating leaderboard or sensitive user balance) that should never be stored in Silcrow's 5-minute memory cache.

**How to use:**

```http
silcrow-cache: no-cache
```

---

## 3. Headers Received from the Server (Side Effects)

These headers are executed sequentially *after* the initial response has been swapped or patched into the DOM. The execution order is strictly: Patch → Invalidate → Navigate → SSE/WS.

### `silcrow-patch`

**Why to use:** To update a *secondary* part of the page using JSON data binding, in addition to the primary request's target.

**When to use:** E.g., You submit a form to update a user profile (primary swap targets the form), and you also want to increment a global "profile update count" badge elsewhere on the page.

**How to use:**

Send a JSON string containing the target selector and the data to bind:

```http
silcrow-patch: {"target": "#update-counter", "data": {"count": 42}}
```

### `silcrow-invalidate`

**Why to use:** To clear Silcrow's cached binding maps for a specific element tree, forcing it to rescan for `:text`, `:class`, and `s-for` loops on the next patch.

**When to use:** When your server response injects entirely new HTML structures containing reactive data attributes that Silcrow hasn't seen yet.

**How to use:**

Provide the CSS selector of the container to invalidate:

```http
silcrow-invalidate: #dashboard
```

### `silcrow-navigate`

**Why to use:** To instruct the client to immediately fetch a new URL.

**When to use:** When an action succeeds (like creating an account) and you want to redirect the user to a new page (like their dashboard) seamlessly via client-side routing.

**How to use:**
Provide the URL path to navigate to:

```http
silcrow-navigate: /dashboard
```

### `silcrow-sse`

**Why to use:** To dynamically open a Server-Sent Events (SSE) connection.

**When to use:** When a user enters a specific state (e.g., joins a chat room) and the server wants to start pushing real-time updates to them.

**How to use:**
Provide the SSE endpoint URL. Silcrow will fire the `silcrow:sse` event internally to handle the connection via the Live system.

```http
silcrow-sse: /api/live/chat
```

### `silcrow-ws`

**Why to use:** To dynamically open a WebSocket connection.

**When to use:** Similar to SSE, but when you require a bi-directional real-time connection.

**How to use:**
Provide the WebSocket endpoint URL. If the trigger targeted an element, the connection binds to that element; otherwise, it binds to `document.body`.

```http
silcrow-ws: wss://example.com/live
```
