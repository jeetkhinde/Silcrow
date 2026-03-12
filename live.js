// silcrow/live.js
// ════════════════════════════════════════════════════════════
// Live — SSE connections & real-time updates
// ════════════════════════════════════════════════════════════

const liveConnections = new Map();      // element → state (SSE) or hub-state (WS compat)
const liveConnectionsByUrl = new Map(); // url → Set<state>  (kept for resolveLiveStates compat)
const sseHubs = new Map();              // normalized url → SseHub
const MAX_BACKOFF = 30000;
const LIVE_HTTP_PROTOCOLS = new Set(["http:", "https:"]);

function isLikelyLiveUrl(value) {
  return (
    typeof value === "string" &&
    (value.startsWith("/") ||
      value.startsWith("http://") ||
      value.startsWith("https://"))
  );
}

function normalizeSSEEndpoint(rawUrl) {
  if (typeof rawUrl !== "string") return null;
  const value = rawUrl.trim();
  if (!value) return null;

  let parsed;
  try {
    parsed = new URL(value, location.origin);
  } catch (e) {
    warn("Invalid SSE URL: " + value);
    return null;
  }

  if (!LIVE_HTTP_PROTOCOLS.has(parsed.protocol)) {
    warn("Rejected non-http(s) SSE URL: " + parsed.href);
    return null;
  }
  if (parsed.origin !== location.origin) {
    warn("Rejected cross-origin SSE URL: " + parsed.href);
    return null;
  }

  return parsed.href;
}

function resolveLiveTarget(selector, fallback) {
  if (typeof selector !== "string" || !selector) return fallback;
  return document.querySelector(selector) || null;
}

function applyLivePatchPayload(payload, fallbackTarget) {
  if (
    payload &&
    typeof payload === "object" &&
    !Array.isArray(payload) &&
    Object.prototype.hasOwnProperty.call(payload, "target")
  ) {
    if (!Object.prototype.hasOwnProperty.call(payload, "data")) {
      warn("SSE patch envelope missing data field");
      return;
    }

    const target = resolveLiveTarget(payload.target, fallbackTarget);
    if (target) {
      patch(payload.data, target);
    }
    return;
  }

  patch(payload, fallbackTarget);
}



function pauseLiveState(state) {
  state.paused = true;
  if (state.protocol !== "ws" && state.hub) {
    state.paused = true;
    state.hub.paused = true;
    if (state.hub.reconnectTimer) {
      clearTimeout(state.hub.reconnectTimer);
      state.hub.reconnectTimer = null;
    }
    if (state.hub.es) {
      state.hub.es.close();
      state.hub.es = null;
    }
  }
}

function resolveLiveStates(root) {
  if (typeof root === "string") {
    // Route key: disconnect/reconnect all connections for the URL
    if (
      root.startsWith("/") ||
      root.startsWith("http://") ||
      root.startsWith("https://")
    ) {
      const fullUrl = new URL(root, location.origin).href;
      // Try HTTP-scheme first (SSE connections)
      let states = liveConnectionsByUrl.get(fullUrl);
      if (!states || states.size === 0) {
        // Fall back to WS-scheme (WebSocket connections)
        const wsUrl = fullUrl.replace(/^http(s?)/, "ws$1");
        states = liveConnectionsByUrl.get(wsUrl);
      }
      return states ? Array.from(states) : [];
    }

    const element = document.querySelector(root);
    if (!element) return [];
    const state = liveConnections.get(element);
    return state ? [state] : [];
  }

  if (!root) return [];
  const state = liveConnections.get(root);
  return state ? [state] : [];
}

function onSSEEvent(e) {
  const path = e?.detail?.path;
  if (!path || typeof path !== "string") return;

  const root = e?.detail?.target || document.body;
  openLive(root, path);
}

function createSseHub(url) {
  return {
    url,
    es: null,
    subscribers: new Set(),
    backoff: 1000,
    paused: false,
    reconnectTimer: null,
  };
}

function getOrCreateSseHub(url) {
  let hub = sseHubs.get(url);
  if (!hub) {
    hub = createSseHub(url);
    sseHubs.set(url, hub);
  }
  return hub;
}

function removeSseHub(hub) {
  if (hub.subscribers.size > 0) return;
  if (hub.reconnectTimer) {
    clearTimeout(hub.reconnectTimer);
    hub.reconnectTimer = null;
  }
  if (hub.es) {
    hub.es.close();
    hub.es = null;
  }
  sseHubs.delete(hub.url);
}

function openLive(root, url) {
  const element = typeof root === "string" ? document.querySelector(root) : root;
  if (!element) {
    warn("Live root not found: " + root);
    return;
  }

  const fullUrl = normalizeSSEEndpoint(url);
  if (!fullUrl) return;

  // Unsubscribe from existing SSE hub if switching
  const existing = liveConnections.get(element);
  if (existing && existing.protocol !== "ws") {
    unsubscribeSse(element);
  }

  const hub = getOrCreateSseHub(fullUrl);
  hub.subscribers.add(element);

  const state = {
    url: fullUrl,
    element,
    paused: false,
    protocol: "sse",
    hub,
  };
  liveConnections.set(element, state);

  let byUrl = liveConnectionsByUrl.get(fullUrl);
  if (!byUrl) {
    byUrl = new Set();
    liveConnectionsByUrl.set(fullUrl, byUrl);
  }
  byUrl.add(state);

  connectSseHub(hub);
}

function unsubscribeSse(element) {
  const state = liveConnections.get(element);
  if (!state || state.protocol === "ws") return;

  const hub = state.hub;
  if (hub) {
    hub.subscribers.delete(element);
    if (hub.subscribers.size === 0) removeSseHub(hub);
  }

  if (liveConnections.get(element) === state) liveConnections.delete(element);

  const byUrl = liveConnectionsByUrl.get(state.url);
  if (byUrl) {
    byUrl.delete(state);
    if (byUrl.size === 0) liveConnectionsByUrl.delete(state.url);
  }
}


function connectSseHub(hub) {
  if (hub.paused || hub.subscribers.size === 0) return;
  if (hub.es && hub.es.readyState < EventSource.CLOSED) return;

  const es = new EventSource(hub.url);
  hub.es = es;

  es.onopen = function () {
    hub.backoff = 1000;
    hub.subscribers.forEach(function (el) {
      document.dispatchEvent(new CustomEvent("silcrow:live:connect", {
        bubbles: true,
        detail: {root: el, url: hub.url, protocol: "sse"},
      }));
    });
  };

  es.onmessage = function (e) {
    try {
      const payload = JSON.parse(e.data);
      const fallback = hub.subscribers.size > 0
        ? hub.subscribers.values().next().value
        : document.body;
      applyLivePatchPayload(payload, fallback);
    } catch (err) {
      warn("Failed to parse SSE message: " + err.message);
    }
  };

  es.addEventListener("patch", function (e) {
    try {
      const payload = JSON.parse(e.data);
      let target = null;
      let data = payload;

      if (payload && typeof payload === "object" && !Array.isArray(payload) &&
        Object.prototype.hasOwnProperty.call(payload, "target")) {
        data = payload.data;
        if (payload.target) target = document.querySelector(payload.target);
      }

      if (!target && hub.subscribers.size > 0) {
        target = hub.subscribers.values().next().value;
      }

      if (target && data !== undefined) patch(data, target);
    } catch (err) {
      warn("Failed to parse SSE patch event: " + err.message);
    }
  });

  es.addEventListener("html", function (e) {
    try {
      const payload = JSON.parse(e.data);
      const target = payload.target
        ? document.querySelector(payload.target)
        : (hub.subscribers.size > 0 ? hub.subscribers.values().next().value : null);
      if (target && Object.prototype.hasOwnProperty.call(payload, "html")) {
        safeSetHTML(target, payload.html == null ? "" : String(payload.html));
      }
    } catch (err) {
      warn("Failed to parse SSE html event: " + err.message);
    }
  });

  es.addEventListener("invalidate", function (e) {
    const selector = e.data ? e.data.trim() : null;
    if (selector) {
      const target = document.querySelector(selector);
      if (target) invalidate(target);
    } else {
      hub.subscribers.forEach(function (el) {invalidate(el);});
    }
  });

  es.addEventListener("navigate", function (e) {
    if (e.data) navigate(e.data.trim(), {trigger: "sse"});
  });

  es.addEventListener("custom", function (e) {
    try {
      const payload = JSON.parse(e.data);
      document.dispatchEvent(new CustomEvent("silcrow:sse:" + (payload.event || "custom"), {
        bubbles: true,
        detail: {url: hub.url, data: payload.data},
      }));
    } catch (err) {
      warn("Failed to parse SSE custom event: " + err.message);
    }
  });

  es.onerror = function () {
    es.close();
    hub.es = null;

    if (hub.paused || hub.subscribers.size === 0) {
      if (hub.subscribers.size === 0) removeSseHub(hub);
      return;
    }

    const reconnectIn = hub.backoff;
    hub.subscribers.forEach(function (el) {
      document.dispatchEvent(new CustomEvent("silcrow:live:disconnect", {
        bubbles: true,
        detail: {root: el, url: hub.url, protocol: "sse", reconnectIn},
      }));
    });

    hub.reconnectTimer = setTimeout(function () {
      hub.reconnectTimer = null;
      connectSseHub(hub);
    }, reconnectIn);

    hub.backoff = Math.min(hub.backoff * 2, MAX_BACKOFF);
  };
}

function disconnectLive(root) {
  const states = resolveLiveStates(root);
  if (!states.length) return;

  for (const state of states) {
    pauseLiveState(state);
  }
}

function reconnectLive(root) {
  const states = resolveLiveStates(root);
  if (!states.length) return;

  const reconnectedHubs = new Set();

  for (const state of states) {
    state.paused = false;

    if (state.protocol === "ws") {
      // Re-subscribe to hub
      const hub = getOrCreateWsHub(state.url);
      hub.subscribers.add(state.element);
      state.hub = hub;

      if (!reconnectedHubs.has(hub)) {
        reconnectedHubs.add(hub);
        hub.paused = false;
        hub.backoff = 1000;
        if (hub.reconnectTimer) {
          clearTimeout(hub.reconnectTimer);
          hub.reconnectTimer = null;
        }
        connectWsHub(hub);
      }
    } else {
      const hub = state.hub;
      if (!hub) continue;
      state.paused = false;
      hub.paused = false;
      hub.backoff = 1000;
      if (hub.reconnectTimer) {
        clearTimeout(hub.reconnectTimer);
        hub.reconnectTimer = null;
      }
      connectSseHub(hub);
    }
  }
}

function destroyAllLive() {
  for (const state of liveConnections.values()) {
    if (state.protocol !== "ws") state.paused = true;
  }
  liveConnections.clear();
  liveConnectionsByUrl.clear();

  for (const hub of sseHubs.values()) {
    if (hub.reconnectTimer) clearTimeout(hub.reconnectTimer);
    if (hub.es) hub.es.close();
  }
  sseHubs.clear();

  for (const hub of wsHubs.values()) {
    if (hub.reconnectTimer) clearTimeout(hub.reconnectTimer);
    if (hub.socket) hub.socket.close();
  }
  wsHubs.clear();
}

// ── Auto-scan for s-live elements on init ──────────────────
function initLiveElements() {
  const elements = document.querySelectorAll("[s-live]");
  for (const el of elements) {
    const raw = el.getAttribute("s-live");
    if (!raw) continue;

    if (raw.startsWith("ws:")) {
      openWsLive(el, raw.slice(3));
    } else {
      openLive(el, raw);
    }
  }
}
