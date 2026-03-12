// silcrow/url-safety.js
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
