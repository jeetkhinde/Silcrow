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

  // 0. SSR hydration seed — populates route atoms + prefetch cache
  // before any framework adapter subscribes, so React's getServerSnapshot
  // returns real data and use() sees a stable resolved promise.
  seedAtomsFromSSR();

  // 1. Unified Live Initialization
  initLiveElements();

  // 1b. Vanilla scope bindings (s-bind="scope")
  initScopeBindings();

  // 2. Fragment-Aware Mutation Observer
  // Tracks live connections AND atom subscriptions for removed nodes,
  // so that detaching an element releases all its references.
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
        unbindElementAtoms(removed);

        if (removed.querySelectorAll) {
          for (const child of removed.querySelectorAll("[s-sse], [s-ws], [s-wss]")) {
            cleanupLiveNode(child);
          }
          for (const child of removed.querySelectorAll("[s-bind]")) {
            unbindElementAtoms(child);
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

  routeAtoms.clear();
  streamAtoms.clear();
  scopeAtoms.clear();
  prefetchPromises.clear();
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

  // --- Headless Store (framework-agnostic; powers React/Solid/Vue/Svelte) ---
  prefetch: prefetchRoute,   // memoized; returns identity-stable Promise<data>
  submit: submitAction,      // async fetch returning {ok, status, data, html, headers}
  subscribe(scope, fn) {
    const atom = resolveAtomByScope(scope, true);
    return atom ? atom.subscribe(fn) : function () {};
  },
  snapshot(scope) {
    const atom = resolveAtomByScope(scope, false);
    return atom ? atom.get() : undefined;
  },
  publish(scope, data) {
    const atom = resolveAtomByScope(scope, true);
    if (atom) atom.patch(data);
  },

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