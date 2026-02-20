// silcrow/crates/silcrow/silcrow.js — Silcrow DOM patcher runtime and navigator implementation

(function () {
  "use strict";

  const DEBUG = document.body.hasAttribute("s-debug");

  function warn(msg) {
    if (DEBUG) console.warn("[silcrow]", msg);
  }

  function throwErr(msg) {
    if (DEBUG) throw new Error("[silcrow] " + msg);
  }

  // ════════════════════════════════════════════════════════════
  // Runtime — reactive data binding & DOM patching
  // ════════════════════════════════════════════════════════════

  const instanceCache = new WeakMap();
  const validatedTemplates = new WeakSet();
  const localBindingsCache = new WeakMap();

  const PATH_RE = /^\.?[A-Za-z0-9_-]+(\.[A-Za-z0-9_-]+)*$/;
  function isValidPath(p) {return PATH_RE.test(p);}

  function parseBind(el) {
    const raw = el.getAttribute("s-bind");
    if (!raw) return null;
    const idx = raw.indexOf(":");
    const path = idx === -1 ? raw : raw.substring(0, idx);
    const prop = idx === -1 ? null : raw.substring(idx + 1);
    return {path, prop};
  }

  function isOnHandler(prop) {
    return prop && prop.toLowerCase().startsWith("on");
  }

  const knownProps = {
    value: "string",
    checked: "boolean",
    disabled: "boolean",
    selected: "boolean",
    src: "string",
    href: "string",
    selectedIndex: "number",
  };

  function setValue(el, prop, value) {
    if (isOnHandler(prop)) {
      throwErr("Binding to event handler attribute rejected: " + prop);
      return;
    }

    if (prop === null) {
      el.textContent = value == null ? "" : String(value);
      return;
    }

    if (value == null) {
      if (prop in knownProps) {
        const t = knownProps[prop];
        if (t === "boolean") el[prop] = false;
        else if (t === "number") el[prop] = 0;
        else el[prop] = "";
      } else {
        el.removeAttribute(prop);
      }
      return;
    }

    if (prop in knownProps) {
      el[prop] = value;
    } else {
      el.setAttribute(prop, String(value));
    }
  }

  function scanBindableNodes(root) {
    const result = [];
    if (root.hasAttribute && root.hasAttribute("s-bind")) result.push(root);
    const descendants = root.querySelectorAll("[s-bind]");
    for (const el of descendants) {
      if (el.closest("template")) continue;
      result.push(el);
    }
    return result;
  }

  function registerBinding(el, scalarMap) {
    const parsed = parseBind(el);
    if (!parsed) return;
    const {path, prop} = parsed;
    if (!path || path.startsWith(".")) return;
    if (isOnHandler(prop)) {
      throwErr("Binding to event handler attribute rejected: " + prop);
      return;
    }
    if (!isValidPath(path)) {
      warn("Invalid path: " + path);
      return;
    }

    if (!scalarMap.has(path)) scalarMap.set(path, []);
    scalarMap.get(path).push({el, prop});
  }

  function registerSubtreeBindings(node, scalarMap) {
    const nodes = scanBindableNodes(node);
    for (const el of nodes) {
      const parsed = parseBind(el);
      if (!parsed) continue;

      // Skip local bindings - they're cached in localBindingsCache
      if (parsed.path.startsWith('.')) continue;

      registerBinding(el, scalarMap);
    }
  }

  function validateTemplate(tpl) {
    const content = tpl.content;
    if (content.querySelectorAll("script").length) {
      throwErr("Script not allowed in template");
    }
    for (const el of content.querySelectorAll("*")) {
      for (const attr of el.attributes) {
        if (attr.name.toLowerCase().startsWith("on")) {
          throwErr("Event handler attribute not allowed in template");
        }
      }
      if (el.hasAttribute("s-list")) {
        throwErr("Nested s-list not allowed");
      }
    }
  }

  function cloneTemplate(tpl, scalarMap) {
    if (!validatedTemplates.has(tpl)) {
      validateTemplate(tpl);
      validatedTemplates.add(tpl);
    }
    const frag = tpl.content.cloneNode(true);
    const elements = [];
    for (const n of frag.children) {
      if (n.nodeType === 1) elements.push(n);
    }
    if (elements.length !== 1) {
      throwErr("Template must contain exactly one element child");
      return document.createElement("div");
    }
    const node = elements[0];

    // Cache local bindings on the node itself
    const localBindings = new Map();

    if (node.hasAttribute("s-bind")) {
      const parsed = parseBind(node);
      if (parsed?.path.startsWith('.')) {
        const field = parsed.path.substring(1); // Remove "."
        if (!localBindings.has(field)) {
          localBindings.set(field, []);
        }
        localBindings.get(field).push({el: node, prop: parsed.prop});
      }
    }

    // Then check descendants
    for (const el of node.querySelectorAll("[s-bind]")) {
      const parsed = parseBind(el);
      if (parsed?.path.startsWith('.')) {
        const field = parsed.path.substring(1);
        if (!localBindings.has(field)) {
          localBindings.set(field, []);
        }
        localBindings.get(field).push({el, prop: parsed.prop});
      }
    }

    // Store in WeakMap
    localBindingsCache.set(node, localBindings);

    registerSubtreeBindings(node, scalarMap);
    return node;
  }


  function makeTemplateResolver(container, scalarMap) {
    const templateId = container.getAttribute("s-template");

    return function resolve(item) {
      let tpl = null;

      if (item && item.key != null) {
        const keyStr = String(item.key);
        const hashIdx = keyStr.indexOf("#");
        if (hashIdx !== -1) {
          const tplName = keyStr.substring(0, hashIdx);
          tpl = document.getElementById(tplName);
        }
      }

      if (!tpl && templateId) tpl = document.getElementById(templateId);

      if (!tpl) {
        tpl = container.querySelector(":scope > template");
      }

      if (!tpl) {
        throwErr("No resolvable template for collection");
        return document.createElement("div");
      }

      return cloneTemplate(tpl, scalarMap);
    };
  }

  function isValidCollectionArray(items) {
    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      if (item == null || typeof item !== "object" || Array.isArray(item)) return false;
      if (!("key" in item)) return false;
    }
    return true;
  }

  function reconcile(container, items, resolveTemplate) {
    if (!isValidCollectionArray(items)) {
      warn("Collection array contains invalid items, discarding");
      return;
    }

    const existing = new Map();
    for (const child of container.children) {
      if (child.dataset && child.hasAttribute("s-key")) {
        existing.set(child.dataset.key, child);
      }
    }

    const validItems = [];
    for (const item of items) {
      if (item.key == null) {
        warn("Collection item missing key, skipping");
        continue;
      }
      validItems.push(item);
    }

    const seen = new Set();
    for (const item of validItems) {
      const k = String(item.key);
      if (seen.has(k)) {
        warn("Duplicate key: " + k);
        return;
      }
      seen.add(k);
    }

    const nextKeys = new Set();
    let prevNode = null;

    for (const item of validItems) {
      const key = String(item.key);
      nextKeys.add(key);

      let node = existing.get(key);

      if (!node) {
        node = resolveTemplate(item);
        node.dataset.key = key;
        node.setAttribute("s-key", "");
      }

      patchItem(node, item);

      if (prevNode) {
        if (prevNode.nextElementSibling !== node) {
          prevNode.after(node);
        }
      } else {
        if (container.firstElementChild !== node) {
          container.prepend(node);
        }
      }

      prevNode = node;
    }

    for (const [key, node] of existing) {
      if (!nextKeys.has(key)) {
        node.remove();
      }
    }
  }

  function patchItem(node, item) {
    const bindings = localBindingsCache.get(node);
    if (!bindings) return;

    for (const field in item) {
      if (field === "key") continue;
      const targets = bindings.get(field);
      if (!targets) continue;
      for (const {el, prop} of targets) {
        setValue(el, prop, item[field]);
      }
    }
  }

  function resolvePath(obj, path) {
    const parts = path.split('.');
    let current = obj;
    for (const part of parts) {
      if (current == null) return undefined;
      // Block prototype pollution
      if (part === '__proto__' || part === 'constructor' || part === 'prototype') {
        return undefined;
      }
      current = current[part];
    }
    return current;
  }

  function buildMaps(root) {
    const scalarMap = new Map();
    const collectionMap = new Map();

    const bindings = root.querySelectorAll("[s-bind]");
    for (const el of bindings) {
      if (el.closest("template")) continue;
      registerBinding(el, scalarMap);
    }

    if (root.hasAttribute && root.hasAttribute("s-bind") && !root.closest("template")) {
      registerBinding(root, scalarMap);
    }

    const lists = root.querySelectorAll("[s-list]");
    for (const container of lists) {
      const listName = container.getAttribute("s-list");
      if (!isValidPath(listName)) {
        throwErr("Invalid collection name: " + listName);
        continue;
      }

      collectionMap.set(listName, {
        container,
        resolveTemplate: makeTemplateResolver(container, scalarMap),
      });
    }

    return {scalarMap, collectionMap};
  }

  function applyPatch(data, scalarMap, collectionMap) {
    for (const [path, bindings] of scalarMap.entries()) {
      const value = resolvePath(data, path);
      if (value !== undefined) {
        for (const {el, prop} of bindings) {
          setValue(el, prop, value);
        }
      }
    }

    for (const [path, {container, resolveTemplate}] of collectionMap.entries()) {
      const value = resolvePath(data, path);
      if (Array.isArray(value)) {
        reconcile(container, value, resolveTemplate);
      } else if (value !== undefined && DEBUG) {
        warn("Collection value is not an array: " + path);
      }
    }
  }

  function resolveRoot(root) {
    if (typeof root === "string") {
      const el = document.querySelector(root);
      if (!el) {
        throwErr("Root element not found: " + root);
        return document.createElement("div");
      }
      return el;
    }
    if (root instanceof Element) return root;
    throwErr("Invalid root: must be selector string or Element");
    return document.createElement("div");
  }

  function patch(data, root, options = {}) {
    const element = resolveRoot(root);

    let instance = instanceCache.get(element);

    if (!instance || options.invalidate) {
      instance = buildMaps(element);
      instanceCache.set(element, instance);
    }

    applyPatch(data, instance.scalarMap, instance.collectionMap);
    // Emit custom event after patching
    if (!options.silent) {
      element.dispatchEvent(new CustomEvent('silcrow:patched', {
        bubbles: true,
        detail: {paths: Array.from(instance.scalarMap.keys())}
      }));
    }
  }

  function invalidate(root) {
    const element = resolveRoot(root);
    instanceCache.delete(element);

    const templates = element.querySelectorAll('template');
    for (const tpl of templates) {
      validatedTemplates.delete(tpl);
    }
  }

  function stream(root) {
    const element = resolveRoot(root);
    let pending = null;
    let scheduled = false;

    return function update(data) {
      pending = data;
      if (scheduled) return;

      scheduled = true;
      queueMicrotask(() => {
        scheduled = false;
        patch(pending, element);
      });
    };
  }

  // ════════════════════════════════════════════════════════════
  // Navigator — client-side routing, history, caching
  // ════════════════════════════════════════════════════════════

  const HTTP_METHODS = ["DELETE", "PUT", "POST", "PATCH", "GET"];
  const DEFAULT_TIMEOUT = 30000; // 30s

  const CACHE_TTL = 5 * 60 * 1000; // 5 minutes
  const MAX_CACHE = 50;
  const abortMap = new WeakMap();
  let routeHandler = null;
  let errorHandler = null;
  const responseCache = new Map();
  const preloadInflight = new Map();

  // ── HTTP Method Detection ──────────────────────────────────
  function getMethod(el) {
    if (el.tagName === "FORM") {
      return (el.getAttribute("method") || "POST").toUpperCase();
    }
    for (const method of HTTP_METHODS) {
      if (el.hasAttribute(method) || el.hasAttribute(method.toLowerCase())) {
        return method;
      }
    }
    return "GET";
  }

  // ── URL Resolution ─────────────────────────────────────────
  function resolveUrl(el) {
    const raw = el.getAttribute("s-action");
    if (!raw) return null;
    try {
      return new URL(raw, location.origin).href;
    } catch (e) {
      return null;
    }
  }

  // ── Target Resolution ──────────────────────────────────────
  function getTarget(el) {
    const sel = el.getAttribute("s-target");
    if (sel) {
      const target = document.querySelector(sel);
      if (target) return target;
    }
    return el;
  }

  // ── Timeout Resolution ─────────────────────────────────────
  function getTimeout(el) {
    const val = el?.getAttribute("s-timeout");
    return val ? parseInt(val, 10) : DEFAULT_TIMEOUT;
  }

  // ── Response Processing ────────────────────────────────────
  function extractHTML(html, targetSelector, isFullPage) {
    const trimmed = html.trimStart();
    if (trimmed.startsWith("<!") || trimmed.startsWith("<html")) {
      const parser = new DOMParser();
      const doc = parser.parseFromString(html, "text/html");

      if (isFullPage) {
        const title = doc.querySelector("title");
        if (title) document.title = title.textContent;
      }

      if (targetSelector) {
        const match = doc.querySelector(targetSelector);
        if (match) return match.innerHTML;
      }

      return doc.body.innerHTML;
    }
    return html;
  }

  // ── Safe HTML Assignment ───────────────────────────────────
  function safeSetHTML(el, html) {
    if (el.setHTML) {
      el.setHTML(html);
    } else {
      el.innerHTML = html;
    }
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
    const targetEl = target || document.body;
    const targetSelector = sourceEl?.getAttribute("s-target") || null;

    // Partial updates skip history by default
    const shouldPushHistory = !skipHistory && !targetSelector && method === "GET";

    // Fire cancelable pre-navigation event
    const event = new CustomEvent("silcrow:navigate", {
      bubbles: true,
      cancelable: true,
      detail: {url: fullUrl, method, trigger, target: targetEl},
    });
    if (!document.dispatchEvent(event)) return;

    // Abort previous in-flight GET for this target (never abort mutations)
    const prevAbort = abortMap.get(targetEl);
    if (prevAbort && prevAbort.method === "GET") {
      prevAbort.controller.abort();
    }
    const controller = new AbortController();
    abortMap.set(targetEl, {controller, method});

    // Timeout: per-element or global default
    const timeout = getTimeout(sourceEl);
    const timeoutId = setTimeout(() => controller.abort(), timeout);

    showLoading(targetEl);

    try {
      let cached = method === "GET" ? cacheGet(fullUrl) : null;

      let text, contentType, redirected = false, finalUrl = fullUrl;
      const wantsHTML = sourceEl?.hasAttribute("s-html");
      if (cached) {
        text = cached.text;
        contentType = cached.contentType;
      } else {
        const fetchOptions = {
          method,
          headers: {
            "silcrow-target": "true",
            "Accept": wantsHTML ? "text/html" : "application/json",
          },
          signal: controller.signal,
        };

        if (body) {
          if (body instanceof FormData) {
            fetchOptions.body = body;
          } else {
            fetchOptions.headers["Content-Type"] = "application/json";
            fetchOptions.body = JSON.stringify(body);
          }
        }

        const response = await fetch(fullUrl, fetchOptions);

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        // Redirect detection
        redirected = response.redirected;
        finalUrl = response.url || fullUrl;

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

      // Route handler hook (receives redirect info)
      if (routeHandler) {
        const handled = await routeHandler({
          url: fullUrl,
          finalUrl,
          redirected,
          method,
          trigger,
          response: text,
          contentType,
          target: targetEl,
        });
        if (handled === false) {
          hideLoading(targetEl);
          return;
        }
      }

      // Save scroll position in current history state BEFORE mutation
      if (shouldPushHistory && trigger !== "popstate") {
        const current = history.state || {};
        history.replaceState(
          {...current, scrollY: window.scrollY},
          "",
          location.href
        );
      }

      // Prepare swap content
      let swapContent;
      const isJSON = contentType.includes("application/json");

      if (isJSON) {
        swapContent = JSON.parse(text);
      } else {
        const isFullPage = !targetSelector;
        swapContent = extractHTML(text, targetSelector, isFullPage);
      }

      // Fire silcrow:before-swap — transition hook
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
        detail: {
          url: finalUrl,
          target: targetEl,
          content: swapContent,
          isJSON,
          proceed,
        },
      });

      if (!document.dispatchEvent(beforeSwap)) return;

      // If no listener called proceed(), do it now
      if (!swapExecuted) proceed();

      // History push AFTER successful render (use finalUrl if redirected)
      const historyUrl = redirected ? finalUrl : fullUrl;
      if (shouldPushHistory && trigger !== "popstate") {
        history.pushState(
          {silcrow: true, url: historyUrl, targetSelector},
          "",
          historyUrl
        );
      }

      // Scroll: restore on popstate, top on full-page nav
      if (trigger === "popstate") {
        const saved = (history.state || {}).scrollY;
        window.scrollTo(0, saved || 0);
      } else if (shouldPushHistory) {
        window.scrollTo(0, 0);
      }

      // Dispatch success event
      document.dispatchEvent(
        new CustomEvent("silcrow:load", {
          bubbles: true,
          detail: {url: finalUrl, target: targetEl, redirected},
        })
      );
    } catch (err) {
      if (err.name === "AbortError") {
        // Distinguish timeout from user-initiated abort
        if (controller.signal.aborted) {
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

  // ── Click Handler (opt-in: only [s-action]) ────────────────
  async function onClick(e) {
    if (e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    if (e.button !== 0) return;

    const el = e.target.closest("[s-action]");
    if (!el || el.tagName === "FORM") return;

    e.preventDefault();

    const fullUrl = resolveUrl(el);
    if (!fullUrl) return;

    const inflight = preloadInflight.get(fullUrl);
    if (inflight) await inflight;

    navigate(fullUrl, {
      method: getMethod(el),
      target: getTarget(el),
      skipHistory: el.hasAttribute("s-skip-history"),
      sourceEl: el,
      trigger: "click",
    });
  }

  // ── Form Handler (opt-in: only form[s-action]) ─────────────
  function onSubmit(e) {
    const form = e.target.closest("form[s-action]");
    if (!form) return;

    e.preventDefault();

    const method = getMethod(form);
    const formData = new FormData(form);

    if (method === "GET") {
      const actionUrl = new URL(form.getAttribute("s-action"), location.origin);
      for (const [k, v] of formData) {
        actionUrl.searchParams.append(k, v);
      }
      navigate(actionUrl.href, {
        method: "GET",
        target: getTarget(form),
        sourceEl: form,
        trigger: "submit",
      });
    } else {
      navigate(form.getAttribute("s-action"), {
        method,
        body: formData,
        target: getTarget(form),
        sourceEl: form,
        trigger: "submit",
      });
    }
  }

  // ── Popstate Handler ───────────────────────────────────────
  function onPopState(e) {
    if (!e.state) return; // Safari guard

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
    const el = e.target.closest("[s-preload]");
    if (!el) return;

    const fullUrl = resolveUrl(el);
    if (!fullUrl || responseCache.has(fullUrl) || preloadInflight.has(fullUrl)) return;
    const controller = new AbortController();
    const wantsHTML = el.hasAttribute("s-html");
    const promise = fetch(fullUrl, {
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
          cacheSet(fullUrl, {text, contentType, ts: Date.now()});
        }
      })
      .catch(() => {})
      .finally(() => preloadInflight.delete(fullUrl));

    preloadInflight.set(fullUrl, promise);
  }

  // ── Init & Teardown ────────────────────────────────────────
  function init() {
    document.addEventListener("click", onClick);
    document.addEventListener("submit", onSubmit);
    window.addEventListener("popstate", onPopState);
    document.addEventListener("mouseenter", onMouseEnter, true);

    if (!history.state?.silcrow) {
      history.replaceState(
        {silcrow: true, url: location.href},
        "",
        location.href
      );
    }
  }

  function destroy() {
    document.removeEventListener("click", onClick);
    document.removeEventListener("submit", onSubmit);
    window.removeEventListener("popstate", onPopState);
    document.removeEventListener("mouseenter", onMouseEnter, true);
    responseCache.clear();
    preloadInflight.clear();
  }

  // ════════════════════════════════════════════════════════════
  // Unified Public API
  // ════════════════════════════════════════════════════════════

  window.Silcrow = {
    // Runtime
    patch,
    invalidate,
    stream,

    // Navigation
    go(path, options = {}) {
      return navigate(path, {
        method: options.method || (options.body ? "POST" : "GET"),
        body: options.body || null,
        target: options.target
          ? document.querySelector(options.target)
          : null,
        trigger: "api",
      });
    },

    onRoute(handler) {
      routeHandler = handler;
      return this;
    },

    onError(handler) {
      errorHandler = handler;
      return this;
    },

    cache: {
      clear(path) {
        if (path) {
          const url = new URL(path, location.origin).href;
          responseCache.delete(url);
        } else {
          responseCache.clear();
        }
      },
      has(path) {
        const url = new URL(path, location.origin).href;
        return !!cacheGet(url);
      },
    },

    destroy,
  };

  // Backward compatibility
  window.SilcrowNavigate = window.Silcrow;

  // Auto-init navigation when DOM is ready
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
