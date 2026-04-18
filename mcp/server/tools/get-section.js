/** @import { loadDocs } from '../lib/docs-loader.js' */

export const definition = {
  name: 'getSection',
  description:
    'Retrieve a single section from a Silcrow.js documentation page by its ID. ' +
    'More efficient than getDoc when you already know which section you need. ' +
    'Section IDs have the form "docId/section-slug" (e.g. "navigator/s-get-attribute"). ' +
    'Use searchDocs to discover section IDs.',
  inputSchema: {
    type: 'object',
    properties: {
      id: {
        type: 'string',
        description: 'Section ID in "docId/section-slug" format',
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

  const normalised = id.trim().toLowerCase();
  const section    = store.getSectionById(normalised);

  if (!section) {
    const sample = store.sections.slice(0, 5).map(s => s.id).join(', ');
    return error(
      `No section found with id "${normalised}". ` +
      `Section IDs follow the pattern "docId/section-slug". ` +
      `Examples: ${sample}`,
    );
  }

  const doc = store.getDocById(section.docId);

  return {
    id:           section.id,
    docId:        section.docId,
    docTitle:     doc?.title ?? section.docId,
    title:        section.title,
    level:        section.level,
    summary:      section.summary,
    content:      section.content,
    examples: {
      html: (section.examples?.html ?? []).map(ex => ({ title: ex.title, code: ex.code })),
      json: (section.examples?.json ?? []).map(ex => ({ title: ex.title, code: ex.code })),
    },
  };
}

function error(message) {
  return { error: message };
}
