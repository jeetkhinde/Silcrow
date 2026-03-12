// silcrow/ws.js
// ════════════════════════════════════════════════════════════
// WebSocket — bidirectional live connections
// ════════════════════════════════════════════════════════════

function normalizeWsEndpoint(rawUrl) {
  if (typeof rawUrl !== "string") return null;
  const value = rawUrl.trim();
  if (!value) return null;

  let parsed;
  try {
    parsed = new URL(value, location.origin);
  } catch (e) {
    warn("Invalid WS URL: " + value);
    return null;
  }

  // Convert http(s) to ws(s) for WebSocket
  if (parsed.protocol === "https:") {
    parsed.protocol = "wss:";
  } else if (parsed.protocol === "http:") {
    parsed.protocol = "ws:";
  }

  if (parsed.protocol !== "ws:" && parsed.protocol !== "wss:") {
    warn("Rejected non-ws(s) WebSocket URL: " + parsed.href);
    return null;
  }

  const expectedOrigin = location.origin.replace(/^http(s?)/, "ws$1");
  if (parsed.origin !== expectedOrigin) {
    warn("Rejected cross-origin WebSocket URL: " + parsed.href);
    return null;
  }

  return parsed.href;
}

const wsHubs = new Map(); // normalized URL → hub object

function createWsHub(url) {
  return {
    url,
    socket: null,
    subscribers: new Set(),
    backoff: 1000,
    paused: false,
    reconnectTimer: null,
  };
}

function getOrCreateWsHub(url) {
  let hub = wsHubs.get(url);
  if (!hub) {
    hub = createWsHub(url);
    wsHubs.set(url, hub);
  }
  return hub;
}

function removeWsHub(hub) {
  if (hub.subscribers.size > 0) return; // safety: don't remove if subscribers exist
  if (hub.reconnectTimer) {
    clearTimeout(hub.reconnectTimer);
    hub.reconnectTimer = null;
  }
  if (hub.socket) {
    hub.socket.close();
    hub.socket = null;
  }
  wsHubs.delete(hub.url);
}

function connectWsHub(hub) {
  if (hub.paused) return;
  if (hub.socket && hub.socket.readyState <= WebSocket.OPEN) return; // already connected/connecting

  const socket = new WebSocket(hub.url);
  hub.socket = socket;

  socket.onopen = function () {
    hub.backoff = 1000;
    document.dispatchEvent(
      new CustomEvent("silcrow:live:connect", {
        bubbles: true,
        detail: {
          url: hub.url,
          protocol: "ws",
          subscribers: Array.from(hub.subscribers),
        },
      })
    );
  };

  socket.onmessage = function (e) {
    dispatchWsMessage(hub, e.data);
  };

  socket.onclose = function () {
    hub.socket = null;
    if (hub.paused) return;
    if (hub.subscribers.size === 0) {
      removeWsHub(hub);
      return;
    }

    const reconnectIn = hub.backoff;

    document.dispatchEvent(
      new CustomEvent("silcrow:live:disconnect", {
        bubbles: true,
        detail: {
          url: hub.url,
          protocol: "ws",
          reconnectIn,
          subscribers: Array.from(hub.subscribers),
        },
      })
    );

    hub.reconnectTimer = setTimeout(function () {
      hub.reconnectTimer = null;
      connectWsHub(hub);
    }, reconnectIn);

    hub.backoff = Math.min(hub.backoff * 2, MAX_BACKOFF);
  };

  socket.onerror = function () {
    // onerror is always followed by onclose per spec
  };
}

function dispatchWsMessage(hub, rawData) {
  try {
    const msg = JSON.parse(rawData);
    const type = msg && msg.type;

    let targets;
    if (msg.target) {
      const el = document.querySelector(msg.target);
      targets = el ? [el] : [];
    } else {
      targets = hub.subscribers;
    }

    if (type === "patch") {
      if (msg.data !== undefined) {
        for (const el of targets) {
          patch(msg.data, el);
        }
      }
    } else if (type === "html") {
      for (const el of targets) {
        safeSetHTML(el, msg.markup == null ? "" : String(msg.markup));
      }
    } else if (type === "invalidate") {
      for (const el of targets) {
        invalidate(el);
      }
    } else if (type === "navigate") {
      // Navigate runs once, not per subscriber
      if (msg.path) {
        navigate(msg.path.trim(), {trigger: "ws"});
      }
    } else if (type === "custom") {
      // Custom event dispatched once on document
      document.dispatchEvent(
        new CustomEvent("silcrow:ws:" + (msg.event || "message"), {
          bubbles: true,
          detail: {url: hub.url, data: msg.data},
        })
      );
    } else {
      warn("Unknown WS event type: " + type);
    }
  } catch (err) {
    warn("Failed to parse WS message: " + err.message);
  }
}

function unsubscribeWs(element) {
  const state = liveConnections.get(element);
  if (!state || state.protocol !== "ws") return;

  const hub = state.hub;
  if (hub) {
    hub.subscribers.delete(element);
    if (hub.subscribers.size === 0) {
      removeWsHub(hub);
    }
  }

  unregisterLiveState(state);
}

function openWsLive(root, url) {
  const element = typeof root === "string" ? document.querySelector(root) : root;
  if (!element) {
    warn("WS live root not found: " + root);
    return;
  }

  const fullUrl = normalizeWsEndpoint(url);
  if (!fullUrl) return;

  // Unsubscribe from previous hub if switching URLs
  const existing = liveConnections.get(element);
  if (existing && existing.protocol === "ws") {
    unsubscribeWs(element);
  } else if (existing) {
    // Was SSE — use existing SSE cleanup
    pauseLiveState(existing);
    unregisterLiveState(existing);
  }

  // Subscribe to hub
  const hub = getOrCreateWsHub(fullUrl);
  hub.subscribers.add(element);

  // Register in liveConnections for compatibility with disconnect/reconnect APIs
  const state = {
    es: null,
    socket: null,
    url: fullUrl,
    element,
    backoff: 0,       // backoff is hub-level now
    paused: false,
    reconnectTimer: null,
    protocol: "ws",
    hub,               // reference to shared hub
  };
  registerLiveState(state);

  // Connect hub if not already connected
  connectWsHub(hub);
}

function sendWs(root, data) {
  const states = resolveLiveStates(root);
  if (!states.length) {
    warn("No live connection found for send target");
    return;
  }

  // Deduplicate: send once per hub, not once per subscriber
  const sentHubs = new Set();

  for (const state of states) {
    if (state.protocol !== "ws") {
      warn("Cannot send on SSE connection — use WS for bidirectional");
      continue;
    }

    const hub = state.hub;
    if (!hub || sentHubs.has(hub)) continue;
    sentHubs.add(hub);

    if (!hub.socket || hub.socket.readyState !== WebSocket.OPEN) {
      warn("WebSocket not open for send");
      continue;
    }

    try {
      const payload = typeof data === "string" ? data : JSON.stringify(data);
      hub.socket.send(payload);
    } catch (err) {
      warn("WS send failed: " + err.message);
    }
  }
}