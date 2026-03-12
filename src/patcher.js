// /patcher.js
// ════════════════════════════════════════════════════════════
// Patcher — reactive data binding & DOM patching (Shorthand Only)
// ════════════════════════════════════════════════════════════

const instanceCache = new WeakMap();
const validatedTemplates = new WeakSet();
const localBindingsCache = new WeakMap();
// 1. Global Middleware Registry
const patchMiddleware = [];
const PATH_RE = /^\.?[A-Za-z0-9_-]+(\.[A-Za-z0-9_-]+)*$/;
function isValidPath(p) {return PATH_RE.test(p);}

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

/**
 * Detects if an element has shorthand reactive attributes (:prop) or s-list.
 */
function hasAnyBinding(el) {
  if (!el.hasAttribute) return false;
  if (el.hasAttribute("s-list")) return true;
  for (const attr of el.attributes) {
    if (attr.name.startsWith(":")) return true;
  }
  return false;
}

/**
 * Validates and adds a binding to the scalar map.
 */
function addBinding(path, prop, el, scalarMap) {
  if (isOnHandler(prop)) return warn("Rejected event binding: " + prop);
  if (!isValidPath(path)) return warn("Invalid path: " + path);

  if (!scalarMap.has(path)) scalarMap.set(path, []);
  scalarMap.get(path).push({el, prop});
}

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

  // Handle :text="path" shorthand for textContent
  if (prop === "text") {
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
  if (hasAnyBinding(root)) result.push(root);

  const descendants = root.querySelectorAll("*");
  for (const el of descendants) {
    if (el.closest("template")) continue;
    if (hasAnyBinding(el)) result.push(el);
  }
  return result;
}

function registerSubtreeBindings(node, scalarMap) {
  for (const el of scanBindableNodes(node)) {
    for (const attr of el.attributes) {
      if (!attr.name.startsWith(":") || attr.name.length <= 1) continue;

      const path = attr.value;
      if (!path.startsWith(".")) {
        addBinding(path, attr.name.slice(1), el, scalarMap);
      }
    }
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
  const node = frag.firstElementChild;
  if (!node) {
    throwErr("Template must contain exactly one element child");
    return document.createElement("div");
  }

  const localBindings = new Map();
  const elements = [node, ...node.querySelectorAll("*")];

  for (const el of elements) {
    for (const attr of el.attributes) {
      if (attr.name.startsWith(":") && attr.value.startsWith('.')) {
        const field = attr.value.substring(1);
        const prop = attr.name.slice(1);
        if (!localBindings.has(field)) localBindings.set(field, []);
        localBindings.get(field).push({el, prop});
      }
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
  return sKey.substring(1);
}

function makeTemplateResolver(container, scalarMap, keyField) {
  const templateId = container.getAttribute("s-template");

  return function resolve(item) {
    let tpl = null;
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

function reconcile(container, items, resolveTemplate, keyField) {
  const existing = new Map();
  for (const child of container.children) {
    if (child.hasAttribute("s-key")) {
      existing.set(child.getAttribute("s-key"), child);
    }
  }

  const validItems = items.filter(i => i != null && typeof i === "object" && hasOwn(i, keyField));
  const nextKeys = new Set();
  let prevNode = null;

  for (const item of validItems) {
    const key = String(item[keyField]);
    nextKeys.add(key);
    let node = existing.get(key) || resolveTemplate(item);
    if (!existing.has(key)) node.setAttribute("s-key", key);

    patchItem(node, item, keyField);

    if (prevNode) {
      if (prevNode.nextElementSibling !== node) prevNode.after(node);
    } else if (container.firstElementChild !== node) {
      container.prepend(node);
    }
    prevNode = node;
  }

  for (const [key, node] of existing) {
    if (!nextKeys.has(key)) node.remove();
  }
}

function patchItem(node, item, keyField) {
  let bindings = localBindingsCache.get(node);
  if (!bindings) {
    bindings = new Map();
    const elements = [node, ...node.querySelectorAll("*")];
    for (const el of elements) {
      for (const attr of el.attributes) {
        if (attr.name.startsWith(":") && attr.value.startsWith('.')) {
          const field = attr.value.substring(1);
          const prop = attr.name.slice(1);
          if (!bindings.has(field)) bindings.set(field, []);
          bindings.get(field).push({el, prop});
        }
      }
    }
    localBindingsCache.set(node, bindings);
  }

  for (const field in item) {
    if (field === keyField) continue;
    const targets = bindings.get(field);
    if (targets) {
      for (const {el, prop} of targets) setValue(el, prop, item[field]);
    }
  }
}

function resolvePath(obj, path) {
  const parts = path.split('.');
  let current = obj;
  for (const part of parts) {
    if (current == null || ['__proto__', 'constructor', 'prototype'].includes(part)) return undefined;
    current = current[part];
  }
  return current;
}

function buildMaps(root) {
  const scalarMap = new Map();
  const collectionMap = new Map();

  registerSubtreeBindings(root, scalarMap);

  const lists = root.hasAttribute("s-list") ? [root] : [];
  const descendants = root.querySelectorAll("[s-list]");
  for (const container of [...lists, ...descendants]) {
    const listName = container.getAttribute("s-list");
    if (isValidPath(listName)) {
      const keyField = getKeyField(container);
      collectionMap.set(listName, {
        container,
        resolveTemplate: makeTemplateResolver(container, scalarMap, keyField),
        keyField,
      });
    }
  }
  return {scalarMap, collectionMap};
}

function mergeItem(container, item, resolveTemplate, keyField) {
  const key = String(item[keyField]);
  let node = null;
  for (const child of container.children) {
    if (child.getAttribute("s-key") === key) {node = child; break;}
  }
  if (!node) {
    node = resolveTemplate(item);
    node.setAttribute("s-key", key);
    container.appendChild(node);
  }
  patchItem(node, item, keyField);
}

function removeItem(container, item, keyField) {
  const key = String(item[keyField]);
  for (const child of container.children) {
    if (child.getAttribute("s-key") === key) {child.remove(); return;}
  }
}

function applyPatch(data, scalarMap, collectionMap) {
  for (const [path, bindings] of scalarMap.entries()) {
    const value = resolvePath(data, path);
    if (value !== undefined) {
      for (const {el, prop} of bindings) setValue(el, prop, value);
    }
  }

  for (const [path, {container, resolveTemplate, keyField}] of collectionMap.entries()) {
    const value = resolvePath(data, path);
    if (Array.isArray(value)) reconcile(container, value, resolveTemplate, keyField);
    else if (value && hasOwn(value, keyField)) {
      if (value._remove) removeItem(container, value, keyField);
      else mergeItem(container, value, resolveTemplate, keyField);
    }
  }
}

function resolveRoot(root) {
  if (typeof root === "string") return document.querySelector(root) || document.createElement("div");
  return root instanceof Element ? root : document.createElement("div");
}

function patch(data, root, options = {}) {
  const element = resolveRoot(root);

  // 1. PIPELINE: Run global transformers
  let transformedData = data;
  try {
    // Use a shallow clone to protect the original response reference
    transformedData = patchMiddleware.reduce((acc, fn) => fn(acc) || acc, {...data});
  } catch (err) {
    warn("Middleware failed: " + err.message);
    transformedData = data; // Fallback to original on error
  }

  // 2. METADATA: Process server-driven toasts (from toasts.js)
  // Pilcrow often sends toasts in the '_toasts' key
  if (transformedData && transformedData._toasts) {
    processToasts(true, transformedData);
  }

  // 3. UNWRAPPING: Smart Data Detection
  // If the object only has a 'data' key after toasts are removed, unwrap it
  if (
    transformedData &&
    typeof transformedData === "object" &&
    transformedData.data !== undefined &&
    Object.keys(transformedData).length === 1
  ) {
    transformedData = transformedData.data;
  }

  // 4. SAFETY GATE: Final Object Check
  const isObject = transformedData !== null &&
    typeof transformedData === 'object' &&
    !Array.isArray(transformedData);

  if (!isObject) {
    warn("Invalid patch data. Expected Object, got: " + typeof transformedData);
    return;
  }

  // 5. EXECUTION: Proceed to DOM Patching
  let instance = instanceCache.get(element);
  if (!instance || options.invalidate) {
    instance = buildMaps(element);
    instanceCache.set(element, instance);
  }

  applyPatch(transformedData, instance.scalarMap, instance.collectionMap);

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
  for (const tpl of element.querySelectorAll('template')) validatedTemplates.delete(tpl);
}

function stream(root) {
  const element = resolveRoot(root);
  let pending = null, scheduled = false;
  return function update(data) {
    pending = data;
    if (scheduled) return;
    scheduled = true;
    queueMicrotask(() => {scheduled = false; patch(pending, element);});
  };
}