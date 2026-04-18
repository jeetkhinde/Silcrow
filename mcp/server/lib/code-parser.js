/**
 * Regex-based extractor for Silcrow constructs in HTML/JS snippets.
 * Returns counts, not AST nodes — sufficient for static analysis.
 */

// s-* in attribute position: <tag s-get="..." or s-get (boolean)
const S_ATTR_RE  = /\bs-([\w-]+)/g;

// :binding in attribute position — must be preceded by whitespace or opening tag
// Excludes URLs (://), CSS pseudo-selectors (button:hover), and template literals
const COLON_RE   = /(?<=[\s"'`<]):([\w]+(?::[\w]+)*)/g;

// Silcrow.method( calls in JS
const API_RE     = /\bSilcrow\.([\w]+)\s*\(/g;

/**
 * @param {string} code
 * @returns {{
 *   attributes: Map<string, number>,   // s-* name → occurrence count
 *   bindings:   Map<string, number>,   // :name → occurrence count
 *   apiCalls:   Map<string, number>,   // Silcrow.method → occurrence count
 * }}
 */
export function parseConstructs(code) {
  const attributes = countMatches(code, S_ATTR_RE,  m => `s-${m[1]}`);
  const bindings   = countMatches(code, COLON_RE,   m => `:${m[1]}`);
  const apiCalls   = countMatches(code, API_RE,     m => `Silcrow.${m[1]}`);
  return { attributes, bindings, apiCalls };
}

function countMatches(code, re, key) {
  const map = new Map();
  re.lastIndex = 0;
  let m;
  while ((m = re.exec(code)) !== null) {
    const k = key(m);
    map.set(k, (map.get(k) ?? 0) + 1);
  }
  return map;
}
