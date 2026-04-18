/** @import { loadDocs } from '../lib/docs-loader.js' */

const MAX_RESULTS = 15;

export const definition = {
  name: 'searchDocs',
  description:
    'Search Silcrow.js documentation by keyword or phrase. Returns ranked ' +
    'matches across doc titles, summaries, tags, and section headings.',
  inputSchema: {
    type: 'object',
    properties: {
      query: {
        type: 'string',
        description: 'Search query — keyword, attribute name, or phrase',
        minLength: 1,
      },
    },
    required: ['query'],
  },
};

/**
 * @param {{ query: string }} args
 * @param {ReturnType<typeof loadDocs>} store
 */
export function handler({ query }, store) {
  if (!query || !query.trim()) {
    return { query, results: [], total: 0 };
  }

  const docHits     = store.docIndex.search(query, { limit: MAX_RESULTS });
  const sectionHits = store.sectionIndex.search(query, { limit: MAX_RESULTS });

  const seen = new Set();
  const results = [];

  for (const hit of docHits) {
    const { id, title, summary, tags } = hit.item;
    if (seen.has(id)) continue;
    seen.add(id);
    results.push({
      type: 'doc',
      id,
      title,
      summary,
      tags,
      score: roundScore(hit.score),
    });
  }

  for (const hit of sectionHits) {
    const { id, docId, title, summary, level } = hit.item;
    if (seen.has(id)) continue;
    seen.add(id);
    results.push({
      type: 'section',
      id,
      docId,
      title,
      summary,
      level,
      score: roundScore(hit.score),
    });
  }

  results.sort((a, b) => a.score - b.score);

  return { query, results: results.slice(0, MAX_RESULTS), total: results.length };
}

function roundScore(score) {
  return score === undefined ? 0 : Math.round(score * 1000) / 1000;
}
