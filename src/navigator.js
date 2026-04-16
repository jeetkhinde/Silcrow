// /navigator.js
// ════════════════════════════════════════════════════════════
// Navigator — client-side routing, history, caching
// ════════════════════════════════════════════════════════════

// Verb attributes: s-get, s-post, s-put, s-delete, s-patch
const VERB_ATTRS = ["s-get", "s-post", "s-put", "s-delete", "s-patch"];
const VERB_SELECTOR = VERB_ATTRS.map(function(a) { return "[" + a + "]"; }).join(",");
const FORM_VERB_SELECTOR = VERB_ATTRS.map(function(a) { return "form[" + a + "]"; }).join(",");
const DEFAULT_TIMEOUT = 30000;

const CACHE_TTL = 5 * 60 * 1000;
const MAX_CACHE = 50;
const abortMap = new WeakMap();
let routeHandler = null;
let errorHandler = null;
const responseCache = new Map();
const preloadInflight = new Map();

// ── Verb Resolution ────────────────────────────────────────
// Returns {url, method} or null if no verb attribute is found.
function resolveVerb(el) {
  for (var i = 0; i < VERB_ATTRS.length; i++) {
    var raw = el.getAttribute(VERB_ATTRS[i]);
    if (raw !== null) {
      // Unified placeholder: Replaces :key with the printed attribute value
      if (raw.includes(":key")) {
        var closest = el.closest("[:key]");
        if (closest) {
          var id = closest.getAttribute(":key");
          raw = raw.replace(/:key/g, id);
        }
      }
      try {
        return {
          url: new URL(raw, location.origin).href,
          method: VERB_ATTRS[i].slice(2).toUpperCase()
        };
      } catch (e) {
        return null;
      }
    }
  }
  return null;
}
// ── Target Resolution ──────────────────────────────────────
/**
 * Resolves the target element for a response swap.
 * Prioritizes explicit s-target, then bubbles up to the nearest loop block.
 */
function getTarget(el) {
  let sel = el.getAttribute("s-target");

  if (sel) {
    // 1. Explicit target with :key interpolation support
    if (sel.includes(":key")) {
      const closest = el.closest("[:key]");
      if (closest) sel = sel.replace(/:key/g, closest.getAttribute(":key"));
    }
    const target = document.querySelector(sel);
    if (target) return target;
  }

  // 2. Contextual Bubble-up: Find the nearest loop item
  const listItem = el.closest("[:key]");
  if (listItem) {
    // If we are inside an s-for block, the primary target is the container 
    // holding the s-for template. This allows the server to return 
    // a single object for a "merge" patch.
    const container = listItem.parentElement;
    if (container && container.querySelector("template[s-for]")) {
      return container;
    }
    return listItem; // Fallback to the individual block
  }

  return el; // Ultimate fallback: target the triggering element
}

// ── Timeout Resolution ─────────────────────────────────────
function getTimeout(el) {
  const val = el?.getAttribute("s-timeout");
  return val ? parseInt(val, 10) : DEFAULT_TIMEOUT;
}

// ── Loading State ──────────────────────────────────────────
function showLoading(el) {
  el.classList.add("silcrow-loading");
  el.setAttribute("aria-busy", "true");
}

function hideLoading(el) {
  el.classList.remove("silcrow-loading");
  el.removeAttribute("aria-busy");
}

// ── Cache Management ───────────────────────────────────────
function cacheSet(url, entry) {
  responseCache.set(url, entry);
  if (responseCache.size > MAX_CACHE) {
    const oldest = responseCache.keys().next().value;
    responseCache.delete(oldest);
  }
}

function cacheGet(url) {
  const cached = responseCache.get(url);
  if (!cached) return null;
  if (Date.now() - cached.ts > CACHE_TTL) {
    responseCache.delete(url);
    return null;
  }
  return cached;
}

function bustCacheOnMutation() {
  responseCache.clear();
}

// ── Side-Effect Header Processing ──────────────────────────
function processSideEffectHeaders(sideEffects, primaryTarget) {
  if (!sideEffects) return;

  // Order: patch → invalidate → navigate → sse
  if (sideEffects.patch) {
    try {
      const payload = JSON.parse(sideEffects.patch);
      if (
        payload &&
        typeof payload === "object" &&
        payload.target &&
        Object.prototype.hasOwnProperty.call(payload, "data")
      ) {
        const el = document.querySelector(payload.target);
        if (el) patch(payload.data, el);
      }
    } catch (e) {
      warn("Failed to process silcrow-patch header: " + e.message);
    }
  }

  if (sideEffects.invalidate) {
    const el = document.querySelector(sideEffects.invalidate);
    if (el) invalidate(el);
  }

  if (sideEffects.navigate) {
    navigate(sideEffects.navigate, {trigger: "header"});
  }

  if (sideEffects.sse) {
    const ssePath = normalizeSSEEndpoint(sideEffects.sse);
    if (!ssePath) return;
    document.dispatchEvent(
      new CustomEvent("silcrow:sse", {
        bubbles: true,
        detail: {path: ssePath, target: primaryTarget || null},
      })
    );
  }

  if (sideEffects.ws) {
    const target = primaryTarget || document.body;
    openWsLive(target, sideEffects.ws);
  }
}

// ── Fetch Request Construction ─────────────────────────────
function buildFetchOptions(method, body, wantsHTML, signal) {
  const opts = {
    method,
    headers: {
      "silcrow-target": "true",
      "Accept": wantsHTML ? "text/html" : "application/json",
    },
    signal,
  };

  if (body) {
    if (body instanceof FormData) {
      opts.body = body;
    } else if (body instanceof URLSearchParams) {
      opts.headers["Content-Type"] = "application/x-www-form-urlencoded";
      opts.body = body;
    } else {
      opts.headers["Content-Type"] = "application/json";
      opts.body = JSON.stringify(body);
    }
  }

  return opts;
}

// ── Response Header Processing ─────────────────────────────
function processResponseHeaders(response, fullUrl) {
  const result = {
    redirected: response.redirected,
    finalUrl: response.url || fullUrl,
    pushUrl: null,
    retargetSelector: null,
    sideEffects: {
      patch: response.headers.get("silcrow-patch"),
      invalidate: response.headers.get("silcrow-invalidate"),
      navigate: response.headers.get("silcrow-navigate"),
      sse: response.headers.get("silcrow-sse"),
      ws: response.headers.get("silcrow-ws"),
    },
  };

  // Fire trigger events
  const triggerHeader = response.headers.get("silcrow-trigger");
  if (triggerHeader) {
    try {
      const triggers = JSON.parse(triggerHeader);
      Object.entries(triggers).forEach(([evt, detail]) => {
        document.dispatchEvent(new CustomEvent(evt, {bubbles: true, detail}));
      });
    } catch (e) {
      document.dispatchEvent(new CustomEvent(triggerHeader, {bubbles: true}));
    }
  }

  // Retarget
  result.retargetSelector = response.headers.get("silcrow-retarget");

  // Push URL override
  result.pushUrl = response.headers.get("silcrow-push");
  if (result.pushUrl) {
    result.finalUrl = new URL(result.pushUrl, location.origin).href;
    result.redirected = true;
  }

  return result;
}

// ── Swap Content Preparation ───────────────────────────────
function prepareSwapContent(text, contentType, targetSelector) {
  const isJSON = contentType.includes("application/json");
  let swapContent;

  if (isJSON) {
    swapContent = JSON.parse(text);
    processToasts(true, swapContent);
  } else {
    const isFullPage = !targetSelector;
    swapContent = extractHTML(text, targetSelector, isFullPage);
    processToasts(false);
  }

  return {swapContent, isJSON};
}

// ── Post-Swap Finalization ─────────────────────────────────
function finalizeNavigation(ctx) {
  const {pushUrl, redirected, finalUrl, fullUrl, shouldPushHistory,
    trigger, targetSelector, targetEl, sideEffects} = ctx;

  processSideEffectHeaders(sideEffects, targetEl);

  const finalHistoryUrl = pushUrl || (redirected ? finalUrl : fullUrl);
  if (shouldPushHistory && trigger !== "popstate") {
    history.pushState(
      {silcrow: true, url: finalHistoryUrl, targetSelector},
      "",
      finalHistoryUrl
    );
  }

  if (trigger === "popstate") {
    const saved = (history.state || {}).scrollY;
    window.scrollTo(0, saved || 0);
  } else if (shouldPushHistory) {
    window.scrollTo(0, 0);
  }

  document.dispatchEvent(
    new CustomEvent("silcrow:load", {
      bubbles: true,
      detail: {url: finalUrl, target: targetEl, redirected},
    })
  );
}

// ── Core Navigate ──────────────────────────────────────────
async function navigate(url, options = {}) {
  const {
    method = "GET",
    body = null,
    target = null,
    trigger = "click",
    skipHistory = false,
    sourceEl = null,
  } = options;

  const fullUrl = new URL(url, location.origin).href;
  let targetEl = target || document.body;
  const targetSelector = sourceEl?.getAttribute("s-target") || null;
  const shouldPushHistory = !skipHistory && !targetSelector && method === "GET";

  const event = new CustomEvent("silcrow:navigate", {
    bubbles: true,
    cancelable: true,
    detail: {url: fullUrl, method, trigger, target: targetEl},
  });
  if (!document.dispatchEvent(event)) return;

  // Abort previous in-flight GET to the same target
  const prevAbort = abortMap.get(targetEl);
  if (prevAbort && prevAbort.method === "GET") {
    prevAbort.controller.abort();
  }
  const controller = new AbortController();
  abortMap.set(targetEl, {controller, method});

  const timeout = getTimeout(sourceEl);
  let timedOut = false;
  const timeoutId = setTimeout(() => {timedOut = true; controller.abort();}, timeout);

  showLoading(targetEl);

  try {
    let cached = method === "GET" ? cacheGet(fullUrl) : null;

    let text, contentType, redirected = false, finalUrl = fullUrl, pushUrl = null;
    let sideEffects = null;

    const wantsHTML = sourceEl?.hasAttribute("s-html");
    if (cached) {
      // Side-effect headers are intentionally not cached — they are
      // one-shot triggers that should only fire on the original response.
      text = cached.text;
      contentType = cached.contentType;
    } else {
      const fetchOpts = buildFetchOptions(method, body, wantsHTML, controller.signal);
      const response = await fetch(fullUrl, fetchOpts);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const headerResult = processResponseHeaders(response, fullUrl);
      redirected = headerResult.redirected;
      finalUrl = headerResult.finalUrl;
      pushUrl = headerResult.pushUrl;
      sideEffects = headerResult.sideEffects;

      // Apply retarget
      if (headerResult.retargetSelector) {
        const newTarget = document.querySelector(headerResult.retargetSelector);
        if (newTarget) targetEl = newTarget;
      }

      text = await response.text();
      contentType = response.headers.get("Content-Type") || "";

      const cacheControl = response.headers.get("silcrow-cache");
      if (method === "GET" && !redirected && cacheControl !== "no-cache") {
        cacheSet(fullUrl, {text, contentType, ts: Date.now()});
      }

      if (method !== "GET") {
        bustCacheOnMutation();
      }
    }

    // Route handler middleware
    if (routeHandler) {
      const handled = await routeHandler({
        url: fullUrl, finalUrl, redirected, method,
        trigger, response: text, contentType, target: targetEl,
      });
      if (handled === false) {
        hideLoading(targetEl);
        return;
      }
    }

    // Save scroll position before pushing
    if (shouldPushHistory && trigger !== "popstate") {
      const current = history.state || {};
      history.replaceState(
        {...current, scrollY: window.scrollY},
        "",
        location.href
      );
    }

    // Prepare and execute swap
    const {swapContent, isJSON} = prepareSwapContent(text, contentType, targetSelector);

    let swapExecuted = false;
    const proceed = () => {
      if (swapExecuted) return;
      swapExecuted = true;
      if (isJSON) {
        patch(swapContent, targetEl);
      } else {
        safeSetHTML(targetEl, swapContent);
      }
    };

    const beforeSwap = new CustomEvent("silcrow:before-swap", {
      bubbles: true,
      cancelable: true,
      detail: {url: finalUrl, target: targetEl, content: swapContent, isJSON, proceed},
    });

    if (!document.dispatchEvent(beforeSwap)) return;
    if (!swapExecuted) proceed();

    // Finalize: side-effects, history, scroll, load event
    finalizeNavigation({
      pushUrl, redirected, finalUrl, fullUrl,
      shouldPushHistory, trigger, targetSelector, targetEl,
      sideEffects,
    });

  } catch (err) {
    if (err.name === "AbortError") {
      if (timedOut) {
        const timeoutErr = new Error(
          `[silcrow] Request timed out after ${timeout}ms`
        );
        timeoutErr.name = "TimeoutError";
        document.dispatchEvent(
          new CustomEvent("silcrow:error", {
            bubbles: true,
            detail: {error: timeoutErr, url: fullUrl},
          })
        );
        if (errorHandler) {
          errorHandler(timeoutErr, {url: fullUrl, method, trigger, target: targetEl});
        }
      }
      return;
    }

    if (errorHandler) {
      errorHandler(err, {url: fullUrl, method, trigger, target: targetEl});
    } else {
      console.error("[silcrow]", err);
    }

    document.dispatchEvent(
      new CustomEvent("silcrow:error", {
        bubbles: true,
        detail: {error: err, url: fullUrl},
      })
    );
  } finally {
    clearTimeout(timeoutId);
    hideLoading(targetEl);
    abortMap.delete(targetEl);
  }
}

// ── Click Handler (opt-in: verb attributes) ────────────────
async function onClick(e) {
  if (e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
  if (e.button !== 0) return;

  if (!e.target || typeof e.target.closest !== "function") return;
  const el = e.target.closest(VERB_SELECTOR);
  if (!el || el.tagName === "FORM") return;

  e.preventDefault();

  const verb = resolveVerb(el);
  if (!verb) return;

  const inflight = preloadInflight.get(verb.url);
  if (inflight) await inflight;

  navigate(verb.url, {
    method: verb.method,
    target: getTarget(el),
    skipHistory: el.hasAttribute("s-skip-history"),
    sourceEl: el,
    trigger: "click",
  });
}

// ── Form Handler (opt-in: verb attributes on form) ─────────
function onSubmit(e) {
  if (!e.target || typeof e.target.closest !== "function") return;
  const form = e.target.closest(FORM_VERB_SELECTOR);
  if (!form) return;

  e.preventDefault();

  const verb = resolveVerb(form);
  if (!verb) return;

  const formData = new FormData(form);

  if (verb.method === "GET") {
    const actionUrl = new URL(verb.url, location.origin);
    for (const [k, v] of formData) {
      actionUrl.searchParams.append(k, v);
    }
    navigate(actionUrl.href, {
      method: verb.method,
      target: getTarget(form),
      sourceEl: form,
      trigger: "submit",
    });
  } else {
    const hasFiles = [...formData.values()].some(v => v instanceof File);

    navigate(verb.url, {
      method: verb.method,
      body: hasFiles ? formData : new URLSearchParams(formData),
      target: getTarget(form),
      sourceEl: form,
      trigger: "submit",
    });
  }
}

// ── Popstate Handler ───────────────────────────────────────
function onPopState(e) {
  if (!e.state) return;

  const url = location.href;
  const state = e.state;

  const targetSelector = state.targetSelector;
  const target = targetSelector
    ? document.querySelector(targetSelector)
    : document.body;

  navigate(url, {
    method: "GET",
    target: target || document.body,
    trigger: "popstate",
    skipHistory: true,
  });
}

// ── Preload Handler ────────────────────────────────────────
function onMouseEnter(e) {
  if (!e.target || typeof e.target.closest !== "function") return;
  const el = e.target.closest("[s-preload]");
  if (!el) return;

  const verb = resolveVerb(el);
  if (!verb || responseCache.has(verb.url) || preloadInflight.has(verb.url)) return;
  const controller = new AbortController();
  const wantsHTML = el.hasAttribute("s-html");
  const promise = fetch(verb.url, {
    headers: {"silcrow-target": "true", "Accept": wantsHTML ? "text/html" : "application/json"},
    signal: controller.signal,
  })
    .then((r) => {
      if (!r.ok) throw new Error(`HTTP ${r.status}`);
      const contentType = r.headers.get("Content-Type") || "";
      const cacheControl = r.headers.get("silcrow-cache");
      return r.text().then((text) => ({text, contentType, cacheControl}));
    })
    .then(({text, contentType, cacheControl}) => {
      if (cacheControl !== "no-cache") {
        cacheSet(verb.url, {text, contentType, ts: Date.now()});
      }
    })
    .catch(() => {})
    .finally(() => preloadInflight.delete(verb.url));

  preloadInflight.set(verb.url, promise);
}
