// /patcher.js
// ════════════════════════════════════════════════════════════
// Patcher — Unified ":" Bindings, s-for Fragments & Identity
// ════════════════════════════════════════════════════════════

const instanceCache = new WeakMap();
const validatedTemplates = new WeakSet();
const localBindingsCache = new WeakMap();
const identityMap = new WeakMap(); // Tracks object reference -> stable UUID
const patchMiddleware = [];

const PATH_RE = /^\.?[A-Za-z0-9_-]+(\.[A-Za-z0-9_-]+)*$/;
function isValidPath(p) {return PATH_RE.test(p);}

const knownProps = {
  value: "string", checked: "boolean", disabled: "boolean",
  selected: "boolean", src: "string", href: "string", selectedIndex: "number",
};

// ── Fix 1: URL-bearing attribute set for security gate ─────
const URL_BINDING_PROPS = new Set([
  "href", "src", "action", "formaction", "xlink:href",
  "poster", "cite", "background"
]);

// ── Fix 3: Prototype pollution deny-set ────────────────────
const BLOCKED_KEYS = new Set(["__proto__", "constructor", "prototype"]);

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

// ── Fix 13: Missing utility — resolveRoot ──────────────────
function resolveRoot(root) {
  if (typeof root === "string") return document.querySelector(root) || document.body;
  return root || document.body;
}

// ── Fix 8: Debug warning on implicit identity ──────────────
function getStableId(obj) {
  if (obj === null || typeof obj !== 'object') return String(obj);
  let id = identityMap.get(obj);
  if (!id) {
    warn('s-for block without :key — identity tracking is unreliable for server data');
    id = crypto.randomUUID();
    identityMap.set(obj, id);
  }
  return id;
}

function parseForExpression(expr) {
  const match = expr.match(/^\s*([A-Za-z0-9_-]+)\s+in\s+([A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)*)\s*$/);
  return match ? {alias: match[1], path: match[2]} : null;
}

// ── Fix 1: URL safety gate in setValue ──────────────────────
function setValue(el, prop, value) {
  if (prop === "text") {
    el.textContent = value == null ? "" : String(value);
    return;
  }
  if (prop === "show") {
    el.style.display = value ? "" : "none";
    return;
  }

  // SECURITY: sanitize URL-bearing properties
  if (URL_BINDING_PROPS.has(prop)) {
    const allowDataImage = prop === "src" && el.tagName === "IMG";
    if (!hasSafeProtocol(String(value || ""), allowDataImage)) {
      warn('Blocked unsafe URL for :' + prop + ' — "' + String(value).slice(0, 50) + '"');
      return;
    }
  }

  if (value == null) {
    if (prop in knownProps) {
      el[prop] = knownProps[prop] === "boolean" ? false : (knownProps[prop] === "number" ? 0 : "");
    } else el.removeAttribute(prop);
    return;
  }
  if (prop in knownProps) el[prop] = value;
  else el.setAttribute(prop, String(value));
}

// ── Fix 2 + Fix 10: Block dangerous bindings + validate prop names ──
function scanBindings(root, alias = null) {
  const bindings = new Map();
  const elements = [root, ...root.querySelectorAll("*")];
  for (const el of elements) {
    if (el.closest("template")) continue;
    for (const attr of el.attributes) {
      if (!attr.name.startsWith(":")) continue;
      const prop = attr.name.slice(1);

      // Fix 10: Reject malformed prop names (empty, double-colon, whitespace)
      if (!prop || prop.startsWith(":") || prop !== prop.trim()) {
        warn('Skipping malformed binding attribute: "' + attr.name + '"');
        continue;
      }

      // Fix 2: Reject event handler, style injection, and srcdoc bindings
      if (prop.startsWith("on") || prop === "style" || prop === "srcdoc") {
        warn('Blocked binding to dangerous attribute: :' + prop);
        continue;
      }

      const path = attr.value;

      if (alias && path.startsWith(alias + ".")) {
        const field = path.substring(alias.length + 1);
        if (!bindings.has(field)) bindings.set(field, []);
        bindings.get(field).push({el, prop});
      } else if (!alias && !path.includes(".")) {
        if (!bindings.has(path)) bindings.set(path, []);
        bindings.get(path).push({el, prop});
      }
    }
  }
  return bindings;
}

// ── Fix 7: Duplicate key detection ─────────────────────────
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

    // Fix 7: Detect duplicate keys
    if (nextKeys.has(key)) {
      warn('Duplicate :key "' + key + '" in s-for — second item ignored');
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

function buildMaps(root) {
  const collections = [];
  root.querySelectorAll("template[s-for]").forEach(tpl => {
    const expr = parseForExpression(tpl.getAttribute("s-for"));
    const keyAttr = tpl.getAttribute(":key");
    const keyPath = keyAttr?.startsWith(expr.alias + ".") ? keyAttr.substring(expr.alias.length + 1) : null;
    collections.push({path: expr.path, tpl, alias: expr.alias, keyPath});
  });
  return {scalars: scanBindings(root), collections};
}

// ── Fix 11: Object merge and remove modes ──────────────────
function mergeOrRemoveItem(container, template, item, alias, keyPath) {
  const key = String(resolvePath(item, keyPath));
  if (!key) return;

  if (item._remove) {
    // Remove mode: delete all nodes with this key
    for (const child of [...container.children]) {
      if (child.getAttribute(":key") === key) child.remove();
    }
    return;
  }

  // Merge mode: update existing or append new
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

// ── Fix 4: Deep clone helper for middleware isolation ───────
function safeClone(obj) {
  try { return structuredClone(obj); }
  catch { return JSON.parse(JSON.stringify(obj)); }
}

function patch(data, root, options = {}) {
  const element = resolveRoot(root);

  // Fix 4: Deep-clone for middleware isolation (prevents cache poisoning)
  let transformedData = data;
  try {
    transformedData = patchMiddleware.reduce((acc, fn) => fn(safeClone(acc)) ?? acc, safeClone(data));
  } catch (err) {
    transformedData = data;
  }

  if (transformedData?._toasts) processToasts(true, transformedData);

  // Fix 5: Smart unwrap — only unwrap { data: X } when X is a plain object
  // Primitives and arrays are valid domain values, not envelopes
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

  // Fix 11: Handle array (full sync), object merge, and object remove
  instance.collections.forEach(col => {
    const val = resolvePath(transformedData, col.path);
    if (Array.isArray(val)) {
      reconcile(col.tpl.parentElement, col.tpl, val, col.alias, col.keyPath);
    } else if (val && typeof val === "object" && col.keyPath) {
      mergeOrRemoveItem(col.tpl.parentElement, col.tpl, val, col.alias, col.keyPath);
    }
  });

  // Fix 12: Fire silcrow:patched event
  const patchedPaths = Array.from(instance.scalars.keys());
  element.dispatchEvent(new CustomEvent("silcrow:patched", {
    bubbles: true,
    detail: {paths: patchedPaths},
  }));
}

// ── Fix 9: Invalidate clears localBindingsCache ────────────
function invalidate(root) {
  const element = resolveRoot(root);
  instanceCache.delete(element);
  element.querySelectorAll('[\\:key]').forEach(el => localBindingsCache.delete(el));
}

// ── Fix 13: Missing utility — stream ───────────────────────
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