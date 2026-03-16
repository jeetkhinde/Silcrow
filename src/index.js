// /index.js
// ════════════════════════════════════════════════════════════
// API — Public Surface & "One Way" Lifecycle
// ════════════════════════════════════════════════════════════

let liveObserver = null;
let middlewareLocked = false;

function init() {
  document.addEventListener("click", onClick);
  document.addEventListener("submit", onSubmit);
  window.addEventListener("popstate", onPopState);
  document.addEventListener("mouseenter", onMouseEnter, true);
  document.addEventListener("silcrow:sse", onSSEEvent);

  if (!history.state?.silcrow) {
    history.replaceState({silcrow: true, url: location.href}, "", location.href);
  }

  // 1. Unified Live Initialization
  initLiveElements();

  // 2. Fragment-Aware Mutation Observer
  // Updated to track elements by our stable identity (:key)
  liveObserver = new MutationObserver(function (mutations) {
    function cleanupLiveNode(node) {
      const state = liveConnections.get(node);
      if (!state) return;

      if (state.protocol === "ws") {
        unsubscribeWs(node);
      } else {
        pauseLiveState(state);
        unregisterLiveState(state);
      }
    }

    for (const mutation of mutations) {
      for (const removed of mutation.removedNodes) {
        if (removed.nodeType !== 1) continue;

        cleanupLiveNode(removed);

        // Cleanup any nested live connections within the removed fragment
        if (removed.querySelectorAll) {
          for (const child of removed.querySelectorAll("[s-live]")) {
            cleanupLiveNode(child);
          }
        }
      }
    }
  });

  liveObserver.observe(document.body, {childList: true, subtree: true});

  // Fix 6: Lock middleware pipeline after initialization
  middlewareLocked = true;
}

function destroy() {
  document.removeEventListener("click", onClick);
  document.removeEventListener("submit", onSubmit);
  window.removeEventListener("popstate", onPopState);
  document.removeEventListener("mouseenter", onMouseEnter, true);
  document.removeEventListener("silcrow:sse", onSSEEvent);

  if (liveObserver) {
    liveObserver.disconnect();
    liveObserver = null;
  }

  responseCache.clear();
  preloadInflight.clear();
  destroyAllLive();
}

window.Silcrow = {
  // --- Runtime (Unified ":" Bindings) ---
  patch,         // Handles middleware, toasts, and s-for blocks
  invalidate,    // Clears cached maps for a root
  stream,        // Batched updates for high-frequency data

  // --- Navigation (Unified ":" Placeholders) ---
  go(path, options = {}) {
    return navigate(path, {
      method: options.method || (options.body ? "POST" : "GET"),
      body: options.body || null,
      target: options.target ? document.querySelector(options.target) : null,
      skipHistory: options.skipHistory || false,
      trigger: "api",
    });
  },

  // --- Live (SSE & WebSocket) ---
  live: openLive,     // Declarative connection manager
  send: sendWs,       // Unified WebSocket sender
  disconnect: disconnectLive,
  reconnect: reconnectLive,

  // --- Feedback Systems ---
  optimistic: optimisticPatch,
  revert: revertOptimistic,
  onToast: (handler) => {setToastHandler(handler); return window.Silcrow;},

  // --- Extensibility ---
  use(fn) {
    if (middlewareLocked) {
      warn("Silcrow.use() called after init — middleware registration is closed.");
      return this;
    }
    if (typeof fn === 'function') patchMiddleware.push(fn);
    return this;
  },

  onRoute: (h) => {routeHandler = h; return window.Silcrow;},
  onError: (h) => {errorHandler = h; return window.Silcrow;},

  destroy,
};

// Auto-boot Silcrow
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}