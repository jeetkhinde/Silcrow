/** @import { loadDocs } from '../lib/docs-loader.js' */

const MAX_RESULTS = 20;
const SEARCH_LIMIT = 8;

export const definition = {
  name: 'getExamples',
  description:
    'Find Silcrow.js code examples related to a topic. Returns HTML and JSON ' +
    'examples extracted from documentation, ranked by relevance.',
  inputSchema: {
    type: 'object',
    properties: {
      topic: {
        type: 'string',
        description:
          'Topic to find examples for — e.g. "s-get", "websocket", ' +
          '"optimistic update", "data binding", "s-for loop"',
        minLength: 1,
      },
    },
    required: ['topic'],
  },
};

/**
 * @param {{ topic: string }} args
 * @param {ReturnType<typeof loadDocs>} store
 */
export function handler({ topic }, store) {
  if (!topic || !topic.trim()) {
    return { topic, examples: [], total: 0 };
  }

  const seen    = new Set();
  const examples = [];

  const docHits     = store.docIndex.search(topic,     { limit: SEARCH_LIMIT });
  const sectionHits = store.sectionIndex.search(topic, { limit: SEARCH_LIMIT });

  // Collect doc-level examples
  for (const hit of docHits) {
    const doc = hit.item;
    collectExamples(examples, seen, doc.examples, {
      docId:    doc.id,
      docTitle: doc.title,
      source:   'doc',
    });
  }

  // Collect section-level examples (more granular, prefer these)
  for (const hit of sectionHits) {
    const section = hit.item;
    const doc     = store.getDocById(section.docId);
    collectExamples(examples, seen, section.examples, {
      docId:        section.docId,
      docTitle:     doc?.title ?? section.docId,
      source:       'section',
      sectionId:    section.id,
      sectionTitle: section.title,
    });
  }

  // Fallback: also scan all doc examples for literal topic matches in code
  if (examples.length < 3) {
    for (const doc of store.docs) {
      appendLiteralMatches(examples, seen, doc, topic);
    }
  }

  return { topic, examples: examples.slice(0, MAX_RESULTS), total: examples.length };
}

function collectExamples(out, seen, examples, meta) {
  for (const type of ['html', 'json']) {
    for (const ex of (examples?.[type] ?? [])) {
      const key = `${meta.docId}:${type}:${ex.code}`;
      if (seen.has(key)) continue;
      seen.add(key);
      out.push({ ...meta, type, title: ex.title, code: ex.code });
    }
  }
}

function appendLiteralMatches(out, seen, doc, topic) {
  const needle = topic.toLowerCase();

  function scan(examples, meta) {
    for (const type of ['html', 'json']) {
      for (const ex of (examples?.[type] ?? [])) {
        if (!ex.code.toLowerCase().includes(needle)) continue;
        const key = `${meta.docId}:${type}:${ex.code}`;
        if (seen.has(key)) continue;
        seen.add(key);
        out.push({ ...meta, type, title: ex.title, code: ex.code });
      }
    }
  }

  scan(doc.examples, { docId: doc.id, docTitle: doc.title, source: 'doc' });

  for (const section of doc.sections) {
    scan(section.examples, {
      docId:        doc.id,
      docTitle:     doc.title,
      source:       'section',
      sectionId:    section.id,
      sectionTitle: section.title,
    });
  }
}
