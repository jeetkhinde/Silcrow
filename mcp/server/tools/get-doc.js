/** @import { loadDocs } from '../lib/docs-loader.js' */

export const definition = {
  name: 'getDoc',
  description:
    'Retrieve a Silcrow.js documentation page by its ID. Returns the full ' +
    'document including all sections with their content and examples. ' +
    'Use searchDocs first to discover valid IDs.',
  inputSchema: {
    type: 'object',
    properties: {
      id: {
        type: 'string',
        description:
          'Document ID (e.g. "attributes", "navigator", "live", "runtime", ' +
          '"events", "http-headers", "javascript-api", "optimistic")',
      },
    },
    required: ['id'],
  },
};

/**
 * @param {{ id: string }} args
 * @param {ReturnType<typeof loadDocs>} store
 */
export function handler({ id }, store) {
  if (!id || !id.trim()) {
    return error('id is required');
  }

  const doc = store.getDocById(id.trim().toLowerCase());
  if (!doc) {
    const available = store.docs.map(d => d.id).join(', ');
    return error(`No doc found with id "${id}". Available: ${available}`);
  }

  return {
    id:        doc.id,
    title:     doc.title,
    summary:   doc.summary,
    tags:      doc.tags,
    use_cases: doc.use_cases,
    examples:  summariseExamples(doc.examples),
    sections:  doc.sections.map(section => ({
      id:       section.id,
      title:    section.title,
      level:    section.level,
      summary:  section.summary,
      content:  section.content,
      examples: summariseExamples(section.examples),
    })),
  };
}

function summariseExamples(examples) {
  return {
    html: (examples?.html ?? []).map(ex => ({ title: ex.title, code: ex.code })),
    json: (examples?.json ?? []).map(ex => ({ title: ex.title, code: ex.code })),
  };
}

function error(message) {
  return { error: message };
}
