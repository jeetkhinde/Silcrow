import { parseConstructs }                          from '../lib/code-parser.js';
import { applyRules, relevantDocs }                 from '../lib/analysis-rules.js';
import { KNOWN_ATTRIBUTES, KNOWN_BINDINGS,
         KNOWN_API_METHODS, DEPRECATED_ATTRIBUTES } from '../lib/silcrow-catalog.js';

const MAX_CODE_BYTES = 100_000;

export const definition = {
  name: 'analyzeSilcrowUsage',
  description:
    'Statically analyze HTML or JavaScript code for Silcrow.js usage. ' +
    'Identifies attributes, bindings, and API calls in use, flags issues ' +
    '(unknown attributes, deprecated patterns, security notes), and returns ' +
    'links to relevant documentation pages.',
  inputSchema: {
    type: 'object',
    properties: {
      code: {
        type: 'string',
        description: 'HTML or JavaScript code snippet to analyze',
        minLength: 1,
      },
    },
    required: ['code'],
  },
};

/**
 * @param {{ code: string }} args
 * @param {ReturnType<import('../lib/docs-loader.js').loadDocs>} store
 */
export function handler({ code }, store) {
  if (!code || !code.trim()) {
    return { error: 'code is required' };
  }

  if (code.length > MAX_CODE_BYTES) {
    return {
      error: `Input too large (${code.length.toLocaleString()} chars). ` +
             `Maximum is ${MAX_CODE_BYTES.toLocaleString()} characters. ` +
             'Pass a focused snippet rather than an entire file.',
    };
  }

  const constructs = parseConstructs(code);
  const issues     = applyRules(constructs, code);
  const docIds     = relevantDocs(constructs);

  const totalConstructs =
    constructs.attributes.size +
    constructs.bindings.size +
    constructs.apiCalls.size;

  return {
    summary: {
      construct_count: totalConstructs,
      issue_count:     issues.length,
      attributes:      [...constructs.attributes.keys()],
      bindings:        [...constructs.bindings.keys()],
      api_calls:       [...constructs.apiCalls.keys()],
    },
    constructs: [
      ...mapConstructs(constructs.attributes, 'attribute', store),
      ...mapConstructs(constructs.bindings,   'binding',   store),
      ...mapConstructs(constructs.apiCalls,   'api_call',  store),
    ],
    issues,
    relevant_docs: docIds.map(id => {
      const doc = store.getDocById(id);
      return { id, title: doc?.title ?? id };
    }),
  };
}

function resolveDocId(name, type) {
  if (type === 'attribute') {
    return KNOWN_ATTRIBUTES.get(name)
      ?? DEPRECATED_ATTRIBUTES.get(name)?.doc
      ?? null;
  }
  if (type === 'binding')  return KNOWN_BINDINGS.get(name)    ?? null;
  if (type === 'api_call') return KNOWN_API_METHODS.get(name) ?? null;
  return null;
}

function mapConstructs(map, type, store) {
  return [...map.entries()].map(([name, count]) => {
    const docId = resolveDocId(name, type);
    const doc   = docId ? store.getDocById(docId) : undefined;
    return {
      type,
      name,
      count,
      ...(docId ? { doc: docId, doc_title: doc?.title ?? docId } : {}),
    };
  });
}
