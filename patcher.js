// ./public/silcrow/patcher.js
// ════════════════════════════════════════════════════════════
// Patcher — reactive data binding & DOM patching
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

function isUnsafeBoundUrl(el, prop, value) {
  const name = String(prop || "").toLowerCase();
  if (!name) return false;

  if (name === "srcset") {
    return !hasSafeSrcSet(value);
  }

  if (!URL_ATTRS.has(name)) return false;

  const allowDataImage = name === "src" && el.tagName === "IMG";
  return !hasSafeProtocol(value, allowDataImage);
}

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

  if (isUnsafeBoundUrl(el, prop, value)) {
    warn("Rejected unsafe URL in binding: " + prop);
    if (prop in knownProps) {
      el[prop] = "";
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

  const localBindings = new Map();

  if (node.hasAttribute("s-bind")) {
    const parsed = parseBind(node);
    if (parsed?.path.startsWith('.')) {
      const field = parsed.path.substring(1);
      if (!localBindings.has(field)) {
        localBindings.set(field, []);
      }
      localBindings.get(field).push({el: node, prop: parsed.prop});
    }
  }

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

  localBindingsCache.set(node, localBindings);
  registerSubtreeBindings(node, scalarMap);
  return node;
}

function asTemplate(el) {
  return el instanceof HTMLTemplateElement ? el : null;
}

function getKeyField(container) {
  const templateId = container.getAttribute("s-template");
  let tpl = null;
  if (templateId) tpl = asTemplate(document.getElementById(templateId));
  if (!tpl) tpl = asTemplate(container.querySelector(":scope > template"));
  if (!tpl) return "key";

  const elements = [];
  for (const n of tpl.content.children) {
    if (n.nodeType === 1) elements.push(n);
  }
  if (elements.length !== 1) return "key";

  const sKey = elements[0].getAttribute("s-key");
  if (!sKey || !sKey.startsWith(".")) return "key";
  return sKey.substring(1); // strip leading "."
}

function makeTemplateResolver(container, scalarMap, keyField) {
  const templateId = container.getAttribute("s-template");

  return function resolve(item) {
    let tpl = null;

    // Key-prefix template resolution: "special#3" → looks for <template id="special">
    if (item && item[keyField] != null) {
      const keyStr = String(item[keyField]);
      const hashIdx = keyStr.indexOf("#");
      if (hashIdx !== -1) {
        const tplName = keyStr.substring(0, hashIdx);
        tpl = asTemplate(document.getElementById(tplName));
      }
    }

    if (!tpl && templateId) tpl = asTemplate(document.getElementById(templateId));
    if (!tpl) tpl = asTemplate(container.querySelector(":scope > template"));

    if (!tpl) {
      throwErr("No resolvable template for collection");
      return document.createElement("div");
    }

    return cloneTemplate(tpl, scalarMap);
  };
}
const hasOwn = (obj, key) => Object.prototype.hasOwnProperty.call(obj, key);
function isValidCollectionArray(items, keyField) {
  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    if (item == null || typeof item !== "object" || Array.isArray(item)) return false;
    if (!hasOwn(item, keyField)) return false;
  }
  return true;
}

function reconcile(container, items, resolveTemplate, keyField) {
  if (!isValidCollectionArray(items, keyField)) {
    warn("Collection array contains invalid items (missing '" + keyField + "' field), discarding");
    return;
  }

  const existing = new Map();
  for (const child of container.children) {
    if (child.hasAttribute && child.hasAttribute("s-key")) {
      existing.set(child.getAttribute("s-key"), child);
    }
  }

  const validItems = [];
  for (const item of items) {
    if (item[keyField] == null) {
      warn("Collection item missing '" + keyField + "' field, skipping");
      continue;
    }
    validItems.push(item);
  }

  const seen = new Set();
  for (const item of validItems) {
    const k = String(item[keyField]);
    if (seen.has(k)) {
      warn("Duplicate key: " + k);
      return;
    }
    seen.add(k);
  }

  const nextKeys = new Set();
  let prevNode = null;

  for (const item of validItems) {
    const key = String(item[keyField]);
    nextKeys.add(key);

    let node = existing.get(key);

    if (!node) {
      node = resolveTemplate(item);
      node.setAttribute("s-key", key);
    }

    patchItem(node, item, keyField);

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

function patchItem(node, item, keyField) {
  let bindings = localBindingsCache.get(node);

  // Fallback for server-rendered nodes not created via cloneTemplate —
  // scan their [s-bind] attributes and cache the result.
  if (!bindings) {
    bindings = new Map();
    for (const el of scanBindableNodes(node)) {
      const parsed = parseBind(el);
      if (parsed?.path.startsWith('.')) {
        const field = parsed.path.substring(1);
        if (!bindings.has(field)) bindings.set(field, []);
        bindings.get(field).push({el, prop: parsed.prop});
      }
    }
    localBindingsCache.set(node, bindings);
  }

  for (const field in item) {
    if (field === keyField) continue;
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

  // Check root itself for s-list — allows targeting the list element directly
  // (e.g. s-target="#task-list" where #task-list IS the [s-list] container)
  if (root.hasAttribute && root.hasAttribute("s-list") && !root.closest("template")) {
    const listName = root.getAttribute("s-list");
    if (isValidPath(listName)) {
      const keyField = getKeyField(root);
      collectionMap.set(listName, {
        container: root,
        resolveTemplate: makeTemplateResolver(root, scalarMap, keyField),
        keyField,
      });
    } else {
      throwErr("Invalid collection name on root: " + listName);
    }
  }

  const lists = root.querySelectorAll("[s-list]");
  for (const container of lists) {
    const listName = container.getAttribute("s-list");
    if (!isValidPath(listName)) {
      throwErr("Invalid collection name: " + listName);
      continue;
    }

    const keyField = getKeyField(container);
    collectionMap.set(listName, {
      container,
      resolveTemplate: makeTemplateResolver(container, scalarMap, keyField),
      keyField,
    });
  }

  return {scalarMap, collectionMap};
}

// Append or update a single keyed item without touching existing siblings.
// Called when s-list receives a plain object (not array) with the key field present.
function mergeItem(container, item, resolveTemplate, keyField) {
  if (item == null || typeof item !== "object" || !hasOwn(item, keyField) || item[keyField] == null) {
    warn("mergeItem: item must be a non-null object with a '" + keyField + "' field");
    return;
  }

  const key = String(item[keyField]);

  // Find existing DOM node for this key
  let node = null;
  for (const child of container.children) {
    if (child.hasAttribute("s-key") && child.getAttribute("s-key") === key) {
      node = child;
      break;
    }
  }

  if (!node) {
    // New item — clone template, set key, append
    node = resolveTemplate(item);
    node.setAttribute("s-key", key);
    container.appendChild(node);
  }

  patchItem(node, item, keyField);
}

// Remove a single keyed item from the container.
// Called when s-list receives {keyField: ..., _remove: true}.
function removeItem(container, item, keyField) {
  if (item == null || typeof item !== "object" || !hasOwn(item, keyField) || item[keyField] == null) {
    warn("removeItem: item must be a non-null object with a '" + keyField + "' field");
    return;
  }
  const key = String(item[keyField]);
  for (const child of container.children) {
    if (child.hasAttribute("s-key") && child.getAttribute("s-key") === key) {
      child.remove();
      return;
    }
  }
  warn("removeItem: no child found with s-key='" + key + "'");
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

  for (const [path, {container, resolveTemplate, keyField}] of collectionMap.entries()) {
    const value = resolvePath(data, path);
    if (Array.isArray(value)) {
      // Array → full sync: reconcile, reorder, remove stale items
      reconcile(container, value, resolveTemplate, keyField);
    } else if (value !== null && typeof value === "object" && hasOwn(value, keyField) && value._remove === true) {

      // Tombstone → remove: delete single keyed item from the list
      removeItem(container, value, keyField);
    } else if (value !== null && typeof value === "object" && hasOwn(value, keyField)) {
      // Keyed object → merge: append/update single item, leave others untouched
      mergeItem(container, value, resolveTemplate, keyField);
    } else if (value !== undefined) {
      warn("Collection value must be an array (full sync) or keyed object (merge): " + path);
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
