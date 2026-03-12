// silcrow/safety.js
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
