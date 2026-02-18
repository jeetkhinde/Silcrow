(function () {
  "use strict";

  const DEBUG = document.body.hasAttribute("s-debug");

  const instanceCache = new WeakMap();
  const validatedTemplates = new WeakSet();
  const localBindingsCache = new WeakMap();
  function warn(msg) {
    if (DEBUG) console.warn("[silcrow]", msg);
  }

  function throwErr(msg) {
    if (DEBUG) throw new Error("[silcrow] " + msg);
  }

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

      // Skip local bindings - they're cached in __silcrowLocalBindings
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

    // Store in WeakMap instead of DOM property
    localBindingsCache.set(node, localBindings);  // ← CHANGE THIS

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

  window.Silcrow = {
    patch: function (data, root, options = {}) {
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
    },

    invalidate: function (root) {
      const element = resolveRoot(root);
      instanceCache.delete(element);

      // Optional: Clear template validation cache for this root
      const templates = element.querySelectorAll('template');
      for (const tpl of templates) {
        validatedTemplates.delete(tpl);
      }
    },
    // NEW: Opt-in batching for high-frequency scenarios.
    stream: function (root) {
      const element = resolveRoot(root);
      let pending = null;
      let scheduled = false;

      return function update(data) {
        pending = data;
        if (scheduled) return;

        scheduled = true;
        queueMicrotask(() => {
          scheduled = false;
          Silcrow.patch(pending, element);
        });
      };
    }
  };
})();