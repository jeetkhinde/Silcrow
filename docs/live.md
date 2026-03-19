# **Live: SSE, WebSocket & Real-Time Updates**

## **Declarative Connections**

Add `s-sse`, `s-ws`, or `s-wss` to any element to automatically open a live connection on page load.

* **SSE (Server-Sent Events):** Use `s-sse` for unidirectional server-to-client streams.

```html
<div id="feed" s-sse="/events/feed">
  <span :text="count"></span> items
</div>

```

* **WebSockets**: Use `s-ws` (or `s-wss` for secure connections) for bidirectional communication.

```html
<div id="chat" s-ws="/ws/chat">
  <span :text="messages"></span>
</div>
```

## **Protocol Enforcement**

Silcrow requires explicit attributes to define the connection type — each attribute must match the intended protocol:

| **Attribute** | **Protocol** | **Direction** |
| --- | --- | --- |
| `s-sse` | Server-Sent Events (HTTP) | Server → Client |
| `s-ws` | WebSocket (`ws://`) | Bidirectional |
| `s-wss` | WebSocket Secure (`wss://`) | Bidirectional |

Using the wrong attribute for a URL scheme (e.g., `s-sse` with a `ws://` URL) will be rejected with a console warning.

## **Connection Sharing**

When multiple elements connect to the same WebSocket URL, Silcrow opens a single shared connection hub.

* **Targeted Messages**: Messages with an explicit `target` selector are applied to the matching element.

* **Fanned Messages**: Messages without a target are dispatched to all elements subscribed to that hub.

## **Programmatic Control**

Connections can also be managed via the JavaScript API:

* `Silcrow.live(root, url)`: Manually opens an SSE connection to a specific root.

* `Silcrow.disconnect(root)`: Pauses a connection and stops automatic reconnection.

* `Silcrow.reconnect(root)`: Resumes a disconnected connection and resets the backoff timer.

* `Silcrow.send(data, root)`: Sends data over an established WebSocket connection.

## **SSE & WebSocket Message Formats**

The server communicates with the client using structured JSON. Both SSE and WebSocket connections share a common set of event types:

| **Type** | **SSE Event Name** | **WS `type` Field** | **Effect** |
| --- | --- | --- | --- |
| `patch` | `message` (default) or `patch` | `"patch"` | Patches JSON data into the target element via `Silcrow.patch()`. Supports `{target, data}` envelope for targeted patches. |
| `html` | `html` | `"html"` | Swaps sanitized HTML into the target element via `safeSetHTML()`. Supports `{target, html}` (SSE) or `{target, markup}` (WS). |
| `invalidate` | `invalidate` | `"invalidate"` | Rebuilds binding maps for the target element via `Silcrow.invalidate()`. |
| `navigate` | `navigate` | `"navigate"` | Triggers client-side navigation to the given URL path. |
| `custom` | `custom` | `"custom"` | Dispatches a `silcrow:sse:{event}` or `silcrow:ws:{event}` CustomEvent on `document`. |

## **SSE Message Format**

The server sends standard SSE messages. The data field must be valid JSON:

```sse
data: {"count": 42, "status": "online"}

```

Silcrow also supports named SSE events for specific actions:

| **Event Name** | **Effect** |
| --- | --- |
| `message` (default) | Parsed as JSON, passed to `Silcrow.patch()` on the root |
| `patch` | Parsed and patched. Supports direct payload on root, or `{target, data}` to patch a specific selector |
| `html` | Swaps HTML via `safeSetHTML()`. Supports `{target, html}`; empty `html` clears target content |
| `invalidate` | Calls `Silcrow.invalidate()` on the root (no data needed) |
| `navigate` | `data` field is a URL path — triggers client-side navigation |

```sse
event: navigate
data: /dashboard

event: invalidate
data:

event: patch
data: {"users": [{"key": "1", "name": "Alice"}]}

event: patch
data: {"target":"#dashboard","data":{"count":42}}

event: html
data: {"target":"#slot","html":"<p>Updated</p>"}

```

## **Reconnection**

When an SSE connection drops, Silcrow reconnects automatically with exponential backoff: 1s → 2s → 4s → 8s → ... up to a maximum of 30 seconds. Backoff resets on successful reconnection or on a manual Silcrow.reconnect() call.

## **Silcrow.disconnect(root)**

Pauses the SSE connection for a root. The connection is closed and automatic reconnection is stopped.

```javascript
Silcrow.disconnect("#feed");

```

## **Silcrow.reconnect(root)**

Resumes a disconnected SSE connection. Resets the backoff timer and reconnects immediately.

```javascript
Silcrow.reconnect("#feed");

```

## **Sending Messages (WebSocket only)**

WebSocket connections are bidirectional. Use `Silcrow.send()` to send data to the server:

```javascript
Silcrow.send({ type: "custom", event: "message", data: { text: "Hello" } }, "#chat");

```

send() is a no-op on SSE connections (SSE is server-to-client only). The connection must be open — if not, a warning is logged.

## **WebSocket Message Format**

WebSocket messages are JSON objects with a `type` field that matches the Rust `WsEvent` enum:

| **Type** | **Fields** | **Effect** |
| --- | --- | --- |
| `patch` | `target`, `data` | Patches JSON data into target element via `Silcrow.patch()` |
| `html` | `target`, `markup` | Swaps HTML into target element via `safeSetHTML()` |
| `invalidate` | `target` | Rebuilds binding maps for target element |
| `navigate` | `path` | Triggers client-side navigation |
| `custom` | `event`, `data` | Dispatches `silcrow:ws:{event}` CustomEvent on `document` |

```json
{"type": "patch", "target": "#stats", "data": {"count": 42}}
{"type": "html", "target": "#slot", "markup": "<p>Updated</p>"}
{"type": "navigate", "path": "/dashboard"}
{"type": "custom", "event": "refresh", "data": {"section": "sidebar"}}

```
