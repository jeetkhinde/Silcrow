import Fuse from 'fuse.js';

const DOC_KEYS = [
  { name: 'title',     weight: 0.35 },
  { name: 'summary',   weight: 0.20 },
  { name: 'tags',      weight: 0.25 },
  { name: 'use_cases', weight: 0.10 },
  { name: 'content',   weight: 0.10 },
];

const SECTION_KEYS = [
  { name: 'title',   weight: 0.40 },
  { name: 'summary', weight: 0.30 },
  { name: 'content', weight: 0.30 },
];

const BASE_OPTIONS = {
  includeScore: true,
  includeMatches: true,
  minMatchCharLength: 2,
  threshold: 0.35,
  ignoreLocation: true,
};

/**
 * Builds a flat sections list with docId attached for result linking.
 */
function flattenSections(docs) {
  const sections = [];
  for (const doc of docs) {
    for (const section of doc.sections) {
      sections.push({ ...section, docId: doc.id });
    }
  }
  return sections;
}

/**
 * @param {object[]} docs - Array of SilcrowDoc from the manifest
 * @returns {{ docIndex: Fuse, sectionIndex: Fuse, sections: object[] }}
 */
export function buildSearchIndex(docs) {
  const sections = flattenSections(docs);

  return {
    docIndex:     new Fuse(docs,     { ...BASE_OPTIONS, keys: DOC_KEYS }),
    sectionIndex: new Fuse(sections, { ...BASE_OPTIONS, keys: SECTION_KEYS }),
    sections,
  };
}
