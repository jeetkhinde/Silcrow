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