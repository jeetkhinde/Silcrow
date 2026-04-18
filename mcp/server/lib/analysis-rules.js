import {
  KNOWN_ATTRIBUTES,
  KNOWN_BINDINGS,
  KNOWN_API_METHODS,
  HTTP_VERB_ATTRIBUTES,
  DEPRECATED_ATTRIBUTES,
} from './silcrow-catalog.js';

/**
 * Each rule receives the parsed constructs + raw code and returns an issue
 * object or null. Severity: 'error' | 'warning' | 'info'.
 *
 * @typedef {{ severity: string, rule: string, message: string, doc?: string }} Issue
 */

/** @type {Array<(ctx: import('./code-parser.js').parseConstructs, code: string) => Issue[]>} */
const RULES = [

  // Unknown s-* attributes
  function unknownAttribute({ attributes }) {
    const issues = [];
    for (const name of attributes.keys()) {
      if (!KNOWN_ATTRIBUTES.has(name) && !DEPRECATED_ATTRIBUTES.has(name)) {
        issues.push({
          severity: 'error',
          rule:     'unknown-attribute',
          message:  `"${name}" is not a recognised Silcrow attribute.`,
          doc:      'attributes',
        });
      }
    }
    return issues;
  },

  // Deprecated attributes
  function deprecatedAttribute({ attributes }) {
    const issues = [];
    for (const [name, info] of DEPRECATED_ATTRIBUTES) {
      if (attributes.has(name)) {
        issues.push({
          severity: 'warning',
          rule:     'deprecated-attribute',
          message:  `"${name}" is deprecated. Use ${info.replacement} instead.`,
          doc:      info.doc,
        });
      }
    }
    return issues;
  },

  // s-for present but no :key binding in the code
  function sForMissingKey({ attributes, bindings }) {
    if (!attributes.has('s-for')) return [];
    if (bindings.has(':key')) return [];
    return [{
      severity: 'warning',
      rule:     'sfor-missing-key',
      message:
        's-for loop detected but no :key binding found. Add :key="<unique-value>" ' +
        'on the loop element for efficient DOM reconciliation.',
      doc: 'runtime',
    }];
  },

  // Multiple HTTP verb attributes in the same snippet — likely on one element
  function multipleVerbsOnElement({ attributes }, code) {
    const verbsFound = [...HTTP_VERB_ATTRIBUTES].filter(v => attributes.has(v));
    if (verbsFound.length < 2) return [];

    // Narrow: check whether any single HTML tag contains two verb attrs
    const tagRe = /<[^>]+>/g;
    const issues = [];
    let m;
    while ((m = tagRe.exec(code)) !== null) {
      const tag = m[0];
      const inTag = [...HTTP_VERB_ATTRIBUTES].filter(v => tag.includes(v));
      if (inTag.length > 1) {
        issues.push({
          severity: 'warning',
          rule:     'multiple-http-verbs',
          message:
            `An element has multiple HTTP verb attributes (${inTag.join(', ')}). ` +
            'Only the first matching verb will fire.',
          doc: 'navigator',
        });
      }
    }
    return issues;
  },

  // s-html carries XSS risk — always flag for awareness
  function sHtmlSecurityNote({ attributes }) {
    if (!attributes.has('s-html')) return [];
    return [{
      severity: 'info',
      rule:     'shtml-xss-note',
      message:
        's-html injects raw HTML into the DOM. Ensure the content comes from a ' +
        'trusted server source; never inject unescaped user input.',
      doc: 'attributes',
    }];
  },

  // s-ws without wss:// — plain WebSocket over non-TLS
  function wsNotSecure({ attributes }, code) {
    if (!attributes.has('s-ws')) return [];
    if (/s-ws\s*=\s*["']wss:\/\//.test(code)) return [];
    return [{
      severity: 'info',
      rule:     'ws-not-secure',
      message:
        's-ws is configured without a wss:// URL. Use wss:// in production to ' +
        'encrypt the WebSocket connection.',
      doc: 'live',
    }];
  },

  // Unknown Silcrow.* API calls
  function unknownApiCall({ apiCalls }) {
    const issues = [];
    for (const name of apiCalls.keys()) {
      if (!KNOWN_API_METHODS.has(name)) {
        issues.push({
          severity: 'error',
          rule:     'unknown-api-call',
          message:  `"${name}" is not a recognised Silcrow API method.`,
          doc:      'javascript-api',
        });
      }
    }
    return issues;
  },
];

/**
 * @param {ReturnType<import('./code-parser.js').parseConstructs>} constructs
 * @param {string} code
 * @returns {Issue[]}
 */
export function applyRules(constructs, code) {
  return RULES.flatMap(rule => rule(constructs, code) ?? []);
}

/**
 * Maps each detected construct to its documentation page.
 * @param {ReturnType<import('./code-parser.js').parseConstructs>} constructs
 * @returns {string[]} Unique doc IDs, sorted
 */
export function relevantDocs(constructs) {
  const docs = new Set();
  for (const name of constructs.attributes.keys()) {
    const doc = KNOWN_ATTRIBUTES.get(name) ?? DEPRECATED_ATTRIBUTES.get(name)?.doc;
    if (doc) docs.add(doc);
  }
  for (const name of constructs.bindings.keys()) {
    const doc = KNOWN_BINDINGS.get(name);
    if (doc) docs.add(doc);
  }
  for (const name of constructs.apiCalls.keys()) {
    const doc = KNOWN_API_METHODS.get(name);
    if (doc) docs.add(doc);
  }
  return [...docs].sort();
}
