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
