// Silcrow.js — Hypermedia Runtime
// Built: 2026-05-01T00:12:20.468Z
(function(){
"use strict";
// /debug.js
// ════════════════════════════════════════════════════════════
// Debug — shared diagnostics
// ════════════════════════════════════════════════════════════

const DEBUG = document.body.hasAttribute("s-debug");

function warn(msg) {
  if (DEBUG) console.warn("[silcrow]", msg);
}

function throwErr(msg) {
  if (DEBUG) throw new Error("[silcrow] " + msg);
}

// /url-safety.js
// ════════════════════════════════════════════════════════════
// URL Safety — shared protocol & URL validation primitives
// ════════════════════════════════════════════════════════════

const URL_SAFE_PROTOCOLS = new Set(["http:", "https:", "mailto:", "tel:"]);

const URL_ATTRS = new Set([
  "action",
  "background",
  "cite",
  "formaction",
  "href",
  "poster",
  "src",
  "xlink:href",
]);

const SAFE_DATA_IMAGE_RE =
  /^data:image\/(?:avif|bmp|gif|jpe?g|png|webp);base64,[a-z0-9+/]+=*$/i;

function hasSafeProtocol(raw, allowDataImage) {
  const value = String(raw || "").trim();
  if (!value) return true;

  const compact = value.replace(/[\u0000-\u0020\u007F]+/g, "");
  if (/^(?:javascript|vbscript|file):/i.test(compact)) return false;

  if (/^data:/i.test(compact)) {
    return allowDataImage && SAFE_DATA_IMAGE_RE.test(compact);
  }

  try {
    const parsed = new URL(value, location.origin);
    return URL_SAFE_PROTOCOLS.has(parsed.protocol);
  } catch (e) {
    return false;
  }
}

function hasSafeSrcSet(raw) {
  const parts = String(raw || "").split(",");
  for (const part of parts) {
    const candidate = part.trim();
    if (!candidate) continue;
    const idx = candidate.search(/\s/);
    const url = idx === -1 ? candidate : candidate.slice(0, idx);
    if (!hasSafeProtocol(url, false)) {
      return false;
    }
  }
  return true;
}

// /safety.js
// ════════════════════════════════════════════════════════════
// Safety — HTML extraction & sanitization
// ════════════════════════════════════════════════════════════

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

const FORBIDDEN_HTML_TAGS = new Set([
  "base",
  "embed",
  "frame",
  "iframe",
  "link",
  "meta",
  "object",
  "script",
  "style",
]);

function hardenBlankTargets(node) {
  if (node.tagName !== "A") return;
  if (String(node.getAttribute("target") || "").toLowerCase() !== "_blank") return;

  const relTokens = new Set(
    String(node.getAttribute("rel") || "")
      .toLowerCase()
      .split(/\s+/)
      .filter(Boolean)
  );
  relTokens.add("noopener");
  relTokens.add("noreferrer");
  node.setAttribute("rel", Array.from(relTokens).join(" "));
}

function sanitizeTree(root) {
  for (const tag of FORBIDDEN_HTML_TAGS) {
    for (const node of root.querySelectorAll(tag)) {
      node.remove();
    }
  }

  for (const node of root.querySelectorAll("*")) {
    if (node.namespaceURI !== "http://www.w3.org/1999/xhtml") {
      node.remove();
      continue;
    }

    for (const attr of [...node.attributes]) {
      const name = attr.name.toLowerCase();
      const value = attr.value;

      if (name.startsWith("on") || name === "style" || name === "srcdoc") {
        node.removeAttribute(attr.name);
        continue;
      }

      if (name === "srcset" && !hasSafeSrcSet(value)) {
        node.removeAttribute(attr.name);
        continue;
      }

      if (URL_ATTRS.has(name)) {
        const allowDataImage = name === "src" && node.tagName === "IMG";
        if (!hasSafeProtocol(value, allowDataImage)) {
          node.removeAttribute(attr.name);
        }
      }
    }

    hardenBlankTargets(node);
  }

  for (const tpl of root.querySelectorAll("template")) {
    sanitizeTree(tpl.content);
  }
}

function safeSetHTML(el, raw) {
  const markup = raw == null ? "" : String(raw);

  if (el.setHTML) {
    el.setHTML(markup);
    return;
  }

  const doc = new DOMParser().parseFromString(markup, "text/html");
  sanitizeTree(doc.body);

  el.innerHTML = doc.body.innerHTML;
}

// /toasts.js
// ════════════════════════════════════════════════════════════
// Toasts — notification processing
// ════════════════════════════════════════════════════════════

let toastHandler = null;

function processToasts(isJSON, content = null) {
  if (!toastHandler) return;

  if (isJSON && content && content._toasts) {
    content._toasts.forEach(t => toastHandler(t.message, t.level));
    delete content._toasts;

    if (content.data !== undefined && Object.keys(content).length === 1) {
      Object.assign(content, content.data);
      delete content.data;
    }
  } else if (!isJSON) {
    const match = document.cookie.match(new RegExp('(^|;\\s*)silcrow_toasts=([^;]+)'));
    if (match) {
      try {
        const rawJSON = decodeURIComponent(match[2]);
        const toasts = JSON.parse(rawJSON);
        toasts.forEach(t => toastHandler(t.message, t.level));
      } catch (e) {
        console.error("Failed to parse toasts", e);
      }
      document.cookie = "silcrow_toasts=; Max-Age=0; path=/";
    }
  }
}

function setToastHandler(handler) {
  toastHandler = handler;
  processToasts(false);
}

// /atoms.js
// ════════════════════════════════════════════════════════════
// Atoms — Headless Reactive Store
// ════════════════════════════════════════════════════════════
// The canonical sink for network-sourced data. The DOM patcher
// is one consumer; framework adapters (React/Solid/Vue/Svelte)
// subscribe via Silcrow.subscribe(scope, fn) and read snapshots
// via Silcrow.snapshot(scope). Structural sharing keeps Object.is
// stable for unchanged subtrees so React 19's useSyncExternalStore
// and use() are safe.

const BLOCKED_ATOM_KEYS = new Set(["__proto__", "constructor", "prototype"]);

function isPlainMergeable(v) {
  if (v === null || typeof v !== "object") return false;
  if (Array.isArray(v)) return true;
  const proto = Object.getPrototypeOf(v);
  return proto === Object.prototype || proto === null;
}

function mergePath(prev, next) {
  if (Object.is(next, prev)) return prev;
  if (!isPlainMergeable(prev) || !isPlainMergeable(next)) return next;
  if (Array.isArray(prev) !== Array.isArray(next)) return next;

  const out = Array.isArray(prev) ? prev.slice() : Object.assign({}, prev);
  let changed = false;
  for (const k in next) {
    if (!Object.prototype.hasOwnProperty.call(next, k)) continue;
    if (BLOCKED_ATOM_KEYS.has(k)) continue;
    const merged = mergePath(prev[k], next[k]);
    if (!Object.is(merged, prev[k])) {
      out[k] = merged;
      changed = true;
    }
  }
  return changed ? out : prev;
}

function createAtom(initial) {
  let value = initial;
  const subs = new Set();

  function notify() {
    for (const fn of subs) {
      try { fn(value); } catch (e) { console.error("[silcrow] atom subscriber threw", e); }
    }
  }

  return {
    get() { return value; },
    set(next) {
      if (Object.is(next, value)) return;
      value = next;
      notify();
    },
    patch(data) {
      const next = mergePath(value, data);
      if (Object.is(next, value)) return;
      value = next;
      notify();
    },
    subscribe(fn) {
      subs.add(fn);
      return function unsubscribe() { subs.delete(fn); };
    },
    _subCount() { return subs.size; },
  };
}

const routeAtoms = new Map();   // pathname -> atom
const streamAtoms = new Map();  // SSE/WS url -> atom
const scopeAtoms = new Map();   // user-named scope -> atom

function getOrCreateAtom(map, key, initial) {
  let atom = map.get(key);
  if (!atom) {
    atom = createAtom(initial);
    map.set(key, atom);
  }
  return atom;
}

function resolveAtomByScope(scope, createIfMissing) {
  if (typeof scope !== "string" || !scope) return null;
  if (scope.startsWith("route:")) {
    const key = scope.slice(6);
    if (!key) return null;
    return createIfMissing
      ? getOrCreateAtom(routeAtoms, key, undefined)
      : routeAtoms.get(key) || null;
  }
  if (scope.startsWith("stream:")) {
    const key = scope.slice(7);
    if (!key) return null;
    return createIfMissing
      ? getOrCreateAtom(streamAtoms, key, undefined)
      : streamAtoms.get(key) || null;
  }
  return createIfMissing
    ? getOrCreateAtom(scopeAtoms, scope, undefined)
    : scopeAtoms.get(scope) || null;
}

// ── Prefetch (use()-safe promise memoization) ──────────────
const prefetchPromises = new Map(); // pathname -> Promise<data>

function prefetchRoute(path) {
  if (typeof path !== "string" || !path) {
    return Promise.reject(new Error("[silcrow] prefetch requires a string path"));
  }
  const key = (function() {
    try { return new URL(path, location.origin).pathname; }
    catch (e) { return path; }
  })();

  const existing = prefetchPromises.get(key);
  if (existing) return existing;

  const url = new URL(path, location.origin).href;
  const promise = fetch(url, {
    headers: {
      "silcrow-target": "true",
      "Accept": "application/json",
    },
  })
    .then(function (r) {
      if (!r.ok) throw new Error("HTTP " + r.status);
      return r.json();
    })
    .then(function (data) {
      // Smart unwrap: { data: X } -> X (matches patch() semantics)
      if (
        data && typeof data === "object" &&
        data.data !== undefined &&
        Object.keys(data).length === 1 &&
        typeof data.data === "object" &&
        data.data !== null &&
        !Array.isArray(data.data)
      ) {
        data = data.data;
      }
      getOrCreateAtom(routeAtoms, key, undefined).set(data);
      return data;
    })
    .catch(function (err) {
      // Evict on error so next attempt is fresh
      prefetchPromises.delete(key);
      throw err;
    });

  prefetchPromises.set(key, promise);
  return promise;
}

function evictPrefetch(path) {
  if (path == null) {
    prefetchPromises.clear();
    return;
  }
  let key = path;
  try { key = new URL(path, location.origin).pathname; } catch (e) {}
  prefetchPromises.delete(key);
}

// ── Async submit (returns parsed result; for useActionState) ─
async function submitAction(url, body, options) {
  options = options || {};
  const fullUrl = new URL(url, location.origin).href;
  const method = options.method || (body ? "POST" : "GET");

  const opts = {
    method,
    headers: {
      "silcrow-target": "true",
      "Accept": "application/json",
    },
  };
  if (options.headers) Object.assign(opts.headers, options.headers);

  if (body) {
    if (body instanceof FormData) {
      opts.body = body;
    } else if (body instanceof URLSearchParams) {
      opts.headers["Content-Type"] = "application/x-www-form-urlencoded";
      opts.body = body;
    } else if (typeof body === "string") {
      opts.body = body;
    } else {
      opts.headers["Content-Type"] = "application/json";
      opts.body = JSON.stringify(body);
    }
  }

  const response = await fetch(fullUrl, opts);
  const contentType = response.headers.get("Content-Type") || "";
  const text = await response.text();

  if (method !== "GET") {
    // Mutation: bust GET cache and any prefetch promise for affected paths
    bustCacheOnMutation();
    const inv = response.headers.get("silcrow-invalidate");
    if (inv) evictPrefetch(inv);
  }

  let parsed = null;
  if (contentType.includes("application/json") && text) {
    try {
      parsed = JSON.parse(text);
      processToasts(true, parsed);
    } catch (e) {
      warn("submit: invalid JSON response");
    }
  }

  if (options.scope && parsed !== null) {
    resolveAtomByScope(options.scope, true).set(parsed);
  }

  return {
    ok: response.ok,
    status: response.status,
    data: parsed,
    html: parsed === null ? text : null,
    headers: response.headers,
  };
}

// ── Vanilla element ↔ atom binding (s-bind) ────────────────
const elementAtomSubs = new WeakMap(); // element -> Set<unsubscribe>

function bindElementToScope(el, scope) {
  const atom = resolveAtomByScope(scope, true);
  if (!atom) return;

  const apply = function (value) {
    if (value === undefined || value === null) return;
    try { patch(value, el); }
    catch (e) { warn("s-bind apply failed: " + e.message); }
  };

  // Initial paint if data is already present
  apply(atom.get());

  const unsub = atom.subscribe(apply);
  let set = elementAtomSubs.get(el);
  if (!set) { set = new Set(); elementAtomSubs.set(el, set); }
  set.add(unsub);
}

function unbindElementAtoms(el) {
  const set = elementAtomSubs.get(el);
  if (!set) return;
  for (const unsub of set) {
    try { unsub(); } catch (e) {}
  }
  elementAtomSubs.delete(el);
}

function initScopeBindings() {
  document.querySelectorAll("[s-bind]").forEach(function (el) {
    const scope = el.getAttribute("s-bind");
    if (scope) bindElementToScope(el, scope);
  });
}

// ── SSR hydration seed ─────────────────────────────────────
// A host can inject `window.__silcrow_seed = { "/path": data, ... }`
// (or `window.__pilcrow_props` for back-compat) before silcrow boots.
// Seeding the route atom + prefetch cache lets React's useSyncExternalStore
// + use() return real data on first paint without a network roundtrip and
// with stable promise identity.
function seedAtomsFromSSR() {
  if (typeof window === "undefined") return;
  const seeds = window.__silcrow_seed || window.__pilcrow_props;
  if (!seeds || typeof seeds !== "object") return;
  for (const key in seeds) {
    if (!Object.prototype.hasOwnProperty.call(seeds, key)) continue;
    if (BLOCKED_ATOM_KEYS.has(key)) continue;
    const value = seeds[key];
    let pathKey = key;
    try { pathKey = new URL(key, location.origin).pathname; } catch (e) {}
    getOrCreateAtom(routeAtoms, pathKey, undefined).set(value);
    prefetchPromises.set(pathKey, Promise.resolve(value));
  }
}

// /patcher.js
// ════════════════════════════════════════════════════════════
// Patcher — Directive-based State, Colon Shorthands & Identity
// ════════════════════════════════════════════════════════════

const instanceCache = new WeakMap();
const validatedTemplates = new WeakSet();
const localBindingsCache = new WeakMap();
const identityMap = new WeakMap(); 
const patchMiddleware = [];

const PATH_RE = /^\.?[A-Za-z0-9_-]+(\.[A-Za-z0-9_-]+)*$/;
function isValidPath(p) { return PATH_RE.test(p); }

const knownProps = {
  value: "string",
  checked: "boolean",
  disabled: "boolean",
  selected: "boolean",
  hidden: "boolean",    
  required: "boolean",  
  readOnly: "boolean",  
  src: "string",
  href: "string",
  selectedIndex: "number",
};

const URL_BINDING_PROPS = new Set([
  "href", "src", "action", "formaction", "xlink:href",
  "poster", "cite", "background"
]);

const BLOCKED_KEYS = new Set(["__proto__", "constructor", "prototype"]);

// ── Internal Utilities ──────────────────────────────────────

function resolvePath(obj, path) {
  if (typeof obj !== "object" || obj === null) return undefined;
  if (!isValidPath(path)) return undefined;
  const parts = path.split(".");
  let cur = obj;
  for (const part of parts) {
    if (BLOCKED_KEYS.has(part)) return undefined;
    if (!Object.prototype.hasOwnProperty.call(cur, part)) return undefined;
    cur = cur[part];
    if (cur === null || cur === undefined) {
      return parts.indexOf(part) === parts.length - 1 ? cur : undefined;
    }
  }
  return cur;
}

function resolveRoot(root) {
  if (typeof root === "string") return document.querySelector(root) || document.body;
  return root || document.body;
}

function getStableId(obj) {
  if (obj === null || typeof obj !== 'object') return String(obj);
  let id = identityMap.get(obj);
  if (!id) {
    id = crypto.randomUUID();
    identityMap.set(obj, id);
  }
  return id;
}

function safeClone(obj) {
  try { return structuredClone(obj); }
  catch { return JSON.parse(JSON.stringify(obj)); }
}

function parseForExpression(expr) {
  const match = expr.match(/^\s*([A-Za-z0-9_-]+)\s+in\s+([A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)*)\s*$/);
  return match ? {alias: match[1], path: match[2]} : null;
}

// ── Binding Engine ──────────────────────────────────────────

function setValue(el, prop, value) {
  if (isOnHandler(prop)) {
    throwErr("Binding to event handler attribute rejected: " + prop);
    return;
  }

  // Spread Directive: s-use="ui"
  if (prop === null) {
    if (value && typeof value === "object" && !Array.isArray(value)) {
      for (const key in value) {
        setValue(el, key, value[key]); 
      }
      return;
    }
    el.textContent = value == null ? "" : String(value);
    return;
  }

  if (prop === "text") {
    el.textContent = value == null ? "" : String(value);
    return;
  }

  if (prop === "show") {
    el.style.display = value ? "" : "none";
    return;
  }

  if (prop === "class") {
    if (value && typeof value === "object" && !Array.isArray(value)) {
      for (const [className, enabled] of Object.entries(value)) {
        el.classList.toggle(className, !!enabled);
      }
    } else {
      el.setAttribute("class", value == null ? "" : String(value));
    }
    return;
  }

  if (prop === "style") {
    if (value && typeof value === "object" && !Array.isArray(value)) {
      for (const [rule, val] of Object.entries(value)) {
        el.style[rule] = val == null ? "" : String(val);
      }
    } else {
      el.setAttribute("style", value == null ? "" : String(value));
    }
    return;
  }

  const name = String(prop).toLowerCase();
  if (URL_BINDING_PROPS.has(name)) {
    const allowDataImage = name === "src" && el.tagName === "IMG";
    if (!hasSafeProtocol(value, allowDataImage)) {
      warn("Rejected unsafe URL in binding: " + prop);
      value = ""; 
    }
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
  } else if (value === false) {
    el.removeAttribute(prop);
  } else if (value === true) {
    el.setAttribute(prop, "");
  } else {
    el.setAttribute(prop, String(value));
  }
}

function parseBind(el) {
  const spreadPath = el.getAttribute("s-use");
  if (spreadPath) return { path: spreadPath, prop: null };

  for (const attr of el.attributes) {
    if (attr.name.startsWith(":") && attr.name !== ":key") {
      const prop = attr.name.slice(1);
      if (prop.startsWith("on") || prop === "style" || prop === "srcdoc") {
        warn('Blocked dangerous binding: :' + prop);
        continue;
      }
      return { path: attr.value, prop };
    }
  }
  return null;
}

function scanBindings(root, alias = null) {
  const bindings = new Map();
  const selector = '[s-use], [\\:text], [\\:class], [\\:style], [\\:show], [\\:value], [\\:disabled], [\\:hidden]';
  
  const elements = [];
  if (root.matches && root.matches(selector)) elements.push(root);
  elements.push(...root.querySelectorAll(selector));

  for (const el of elements) {
    if (el.closest("template")) continue;
    const parsed = parseBind(el);
    if (!parsed) continue;

    const { path, prop } = parsed;

    if (alias && path.startsWith(alias + ".")) {
      const field = path.substring(alias.length + 1);
      if (!bindings.has(field)) bindings.set(field, []);
      bindings.get(field).push({ el, prop });
    } else if (!alias) {
      if (!bindings.has(path)) bindings.set(path, []);
      bindings.get(path).push({ el, prop });
    }
  }
  return bindings;
}

// ── Collection Engine ───────────────────────────────────────

function reconcile(container, template, items, alias, keyPath) {
  const existingBlocks = new Map();
  for (const child of container.children) {
    const k = child.getAttribute(":key");
    if (k) {
      if (!existingBlocks.has(k)) existingBlocks.set(k, []);
      existingBlocks.get(k).push(child);
    }
  }

  const nextKeys = new Set();
  let anchor = template;

  for (const item of items) {
    const key = String(keyPath ? resolvePath(item, keyPath) : getStableId(item));
    if (nextKeys.has(key)) {
      warn('Duplicate :key "' + key + '" in s-for — item skipped');
      continue;
    }
    nextKeys.add(key);

    let block = existingBlocks.get(key);
    if (!block) {
      const frag = template.content.cloneNode(true);
      block = Array.from(frag.children).filter(n => n.nodeType === 1);
      block.forEach(el => el.setAttribute(":key", key));
    }

    block.forEach(node => {
      patchItem(node, item, alias);
      if (anchor.nextElementSibling !== node) anchor.after(node);
      anchor = node;
    });
  }

  for (const [key, nodes] of existingBlocks) {
    if (!nextKeys.has(key)) nodes.forEach(n => n.remove());
  }
}

function patchItem(node, item, alias) {
  let bindings = localBindingsCache.get(node);
  if (!bindings) {
    bindings = scanBindings(node, alias);
    localBindingsCache.set(node, bindings);
  }
  for (const field in item) {
    const targets = bindings.get(field);
    if (targets) targets.forEach(t => setValue(t.el, t.prop, item[field]));
  }
}

function mergeOrRemoveItem(container, template, item, alias, keyPath) {
  const key = String(resolvePath(item, keyPath));
  if (!key) return;

  if (item._remove) {
    for (const child of [...container.children]) {
      if (child.getAttribute(":key") === key) child.remove();
    }
    return;
  }

  const existing = [];
  for (const child of container.children) {
    if (child.getAttribute(":key") === key) existing.push(child);
  }

  if (existing.length > 0) {
    existing.forEach(node => patchItem(node, item, alias));
  } else {
    const frag = template.content.cloneNode(true);
    const block = Array.from(frag.children).filter(n => n.nodeType === 1);
    block.forEach(el => {
      el.setAttribute(":key", key);
      patchItem(el, item, alias);
      container.appendChild(el);
    });
  }
}

// ── Public API & Lifecycle ──────────────────────────────────

function buildMaps(root) {
  const collections = [];
  root.querySelectorAll("template[s-for]").forEach(tpl => {
    const expr = parseForExpression(tpl.getAttribute("s-for"));
    const keyAttr = tpl.getAttribute(":key");
    const keyPath = keyAttr?.startsWith(expr.alias + ".") 
      ? keyAttr.substring(expr.alias.length + 1) 
      : keyAttr;
      
    collections.push({path: expr.path, tpl, alias: expr.alias, keyPath});
  });
  return {scalars: scanBindings(root), collections};
}

function patch(data, root, options = {}) {
  const element = resolveRoot(root);

  let transformedData = data;
  try {
    transformedData = patchMiddleware.reduce((acc, fn) => fn(safeClone(acc)) ?? acc, safeClone(data));
  } catch (err) {
    transformedData = data;
  }

  if (transformedData?._toasts) processToasts(true, transformedData);

  // Smart Unwrap: { data: X } -> X for plain objects
  if (
    transformedData?.data !== undefined &&
    Object.keys(transformedData).length === 1 &&
    typeof transformedData.data === "object" &&
    transformedData.data !== null &&
    !Array.isArray(transformedData.data)
  ) {
    transformedData = transformedData.data;
  }

  let instance = instanceCache.get(element);
  if (!instance || options.invalidate) {
    instance = buildMaps(element);
    instanceCache.set(element, instance);
  }

  for (const [path, bindings] of instance.scalars.entries()) {
    const val = resolvePath(transformedData, path);
    if (val !== undefined) bindings.forEach(b => setValue(b.el, b.prop, val));
  }

  instance.collections.forEach(col => {
    const val = resolvePath(transformedData, col.path);
    if (Array.isArray(val)) {
      reconcile(col.tpl.parentElement, col.tpl, val, col.alias, col.keyPath);
    } else if (val && typeof val === "object" && col.keyPath) {
      mergeOrRemoveItem(col.tpl.parentElement, col.tpl, val, col.alias, col.keyPath);
    }
  });

  element.dispatchEvent(new CustomEvent("silcrow:patched", {
    bubbles: true,
    detail: {paths: Array.from(instance.scalars.keys()), target: element},
  }));
}

function invalidate(root) {
  const element = resolveRoot(root);
  instanceCache.delete(element);
  element.querySelectorAll('[\\:key]').forEach(el => localBindingsCache.delete(el));
}

function stream(root) {
  let pending = null;
  return function(data) {
    pending = data;
    queueMicrotask(() => {
      if (pending === data) {
        patch(pending, root);
        pending = null;
      }
    });
  };
}
// /live.js
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



function registerLiveState(state) {
  liveConnections.set(state.element, state);
  let byUrl = liveConnectionsByUrl.get(state.url);
  if (!byUrl) {
    byUrl = new Set();
    liveConnectionsByUrl.set(state.url, byUrl);
  }
  byUrl.add(state);
}

function unregisterLiveState(state) {
  if (liveConnections.get(state.element) === state) liveConnections.delete(state.element);
  const byUrl = liveConnectionsByUrl.get(state.url);
  if (byUrl) {
    byUrl.delete(state);
    if (byUrl.size === 0) liveConnectionsByUrl.delete(state.url);
  }
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

      // Mirror into stream atom for headless subscribers (React/Solid/etc.)
      const atomData =
        payload && typeof payload === "object" && !Array.isArray(payload) &&
        Object.prototype.hasOwnProperty.call(payload, "target") &&
        Object.prototype.hasOwnProperty.call(payload, "data")
          ? payload.data
          : payload;
      getOrCreateAtom(streamAtoms, hub.url, undefined).patch(atomData);
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

      if (data !== undefined) {
        getOrCreateAtom(streamAtoms, hub.url, undefined).patch(data);
      }
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

/**
 * Scans the DOM for explicit live connection attributes.
 * Strict protocol enforcement.
 */
function initLiveElements() {
  // 1. Server-Sent Events (SSE)
  document.querySelectorAll("[s-sse]").forEach(el => {
    const url = el.getAttribute("s-sse");
    if (url) openLive(el, url);
  });

  // 2. WebSockets (WS/WSS)
  document.querySelectorAll("[s-ws], [s-wss]").forEach(el => {
    const url = el.getAttribute("s-ws") || el.getAttribute("s-wss");
    if (url) openWsLive(el, url);
  });
}

// /ws.js
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
        getOrCreateAtom(streamAtoms, hub.url, undefined).patch(msg.data);
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

function sendWs(data, root) {
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

// ── Boost helpers ──────────────────────────────────────────
function isSafeBoostHref(anchor) {
  const href = anchor.getAttribute("href");
  if (!href || href.startsWith("#")) return false;
  if (anchor.hasAttribute("download")) return false;
  if (anchor.getAttribute("target") === "_blank") return false;
  try {
    const url = new URL(href, location.origin);
    return url.origin === location.origin;
  } catch (e) {
    return false;
  }
}

function getBoostTarget(boostEl) {
  const sel = boostEl.getAttribute("s-target");
  if (sel) {
    const t = document.querySelector(sel);
    if (t) return t;
  }
  return document.body;
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
  // Prefetch promises must also clear so subsequent use() calls observe fresh data.
  // Atoms keep their last-known value (consumers read snapshots until refetch).
  prefetchPromises.clear();
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

      if (
        method === "GET" &&
        !targetSelector &&
        response.headers.get("silcrow-full-reload") === "true"
      ) {
        window.location.assign(response.url || fullUrl);
        return;
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

    // Mirror top-level GET JSON into the route atom for headless consumers.
    // Skip fragment swaps (s-target set), non-GET, and HTML responses.
    if (isJSON && method === "GET" && !targetSelector) {
      try {
        const pathKey = new URL(finalUrl).pathname;
        getOrCreateAtom(routeAtoms, pathKey, undefined).set(swapContent);
        prefetchPromises.set(pathKey, Promise.resolve(swapContent));
      } catch (e) {}
    }

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

// ── Click Handler (opt-in: verb attributes + s-boost) ──────
async function onClick(e) {
  if (e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
  if (e.button !== 0) return;
  if (!e.target || typeof e.target.closest !== "function") return;

  // 1. Opt-in verb attribute elements (s-get, s-post, etc.)
  const el = e.target.closest(VERB_SELECTOR);
  if (el && el.tagName !== "FORM") {
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
    return;
  }

  // 2. s-boost: intercept plain <a href> inside boosted containers
  const anchor = e.target.closest("a[href]");
  if (!anchor || !isSafeBoostHref(anchor)) return;
  const boostEl = anchor.closest("[s-boost]");
  if (!boostEl) return;

  e.preventDefault();
  const boostedUrl = new URL(anchor.getAttribute("href"), location.origin).href;
  const inflight = preloadInflight.get(boostedUrl);
  if (inflight) await inflight;

  navigate(boostedUrl, {
    method: "GET",
    target: getBoostTarget(boostEl),
    skipHistory: anchor.hasAttribute("s-skip-history"),
    sourceEl: anchor,
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
function startPreload(url, wantsHTML) {
  if (responseCache.has(url) || preloadInflight.has(url)) return;
  const controller = new AbortController();
  const promise = fetch(url, {
    headers: {"silcrow-target": "true", "Accept": wantsHTML ? "text/html" : "application/json"},
    signal: controller.signal,
  })
    .then((r) => {
      if (!r.ok) throw new Error(`HTTP ${r.status}`);
      if (r.headers.get("silcrow-full-reload") === "true") return null;
      const contentType = r.headers.get("Content-Type") || "";
      const cacheControl = r.headers.get("silcrow-cache");
      return r.text().then((text) => ({text, contentType, cacheControl}));
    })
    .then((entry) => {
      if (!entry) return;
      const {text, contentType, cacheControl} = entry;
      if (cacheControl !== "no-cache") {
        cacheSet(url, {text, contentType, ts: Date.now()});
      }
    })
    .catch(() => {})
    .finally(() => preloadInflight.delete(url));
  preloadInflight.set(url, promise);
}

function onMouseEnter(e) {
  if (!e.target || typeof e.target.closest !== "function") return;
  const el = e.target.closest("[s-preload]");
  if (!el) return;

  const verb = resolveVerb(el);
  if (verb) {
    startPreload(verb.url, el.hasAttribute("s-html"));
    return;
  }

  // s-boost: preload plain anchors with s-preload inside boosted containers
  if (el.tagName === "A" && isSafeBoostHref(el) && el.closest("[s-boost]")) {
    const url = new URL(el.getAttribute("href"), location.origin).href;
    startPreload(url, el.hasAttribute("s-html"));
  }
}

// /optimistic.js
// ════════════════════════════════════════════════════════════
// Optimistic — snapshot & revert for instant UI feedback
// ════════════════════════════════════════════════════════════

const snapshots = new WeakMap();

function optimisticPatch(data, root) {
  const element = typeof root === "string" ? document.querySelector(root) : root;
  if (!element) {
    warn("Optimistic root not found: " + root);
    return;
  }

  // Snapshot current DOM state
  snapshots.set(element, element.innerHTML);

  // Apply the optimistic data
  patch(data, element);

  document.dispatchEvent(
    new CustomEvent("silcrow:optimistic", {
      bubbles: true,
      detail: {root: element, data},
    })
  );
}

function revertOptimistic(root) {
  const element = typeof root === "string" ? document.querySelector(root) : root;
  if (!element) {
    warn("Revert root not found: " + root);
    return;
  }

  const saved = snapshots.get(element);
  if (saved === undefined) {
    warn("No snapshot to revert for element");
    return;
  }

  element.innerHTML = saved;
  snapshots.delete(element);
  invalidate(element);

  document.dispatchEvent(
    new CustomEvent("silcrow:revert", {
      bubbles: true,
      detail: {root: element},
    })
  );
}
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
})();
