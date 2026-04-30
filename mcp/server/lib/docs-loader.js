import { readFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { buildSearchIndex } from './search-index.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DEFAULT_DOCS_PATH = resolve(__dirname, '../../docs.json');

const SUPPORTED_SCHEMA_VERSION = 1;

/** Singleton store — loaded once per process. */
let store = null;

/**
 * Reads and parses docs.json, validates the schema version, builds the
 * Fuse.js search indices, and returns a frozen store object.
 *
 * @param {string} [docsPath] - Override path to docs.json (useful in tests)
 * @returns {{
 *   manifest: object,
 *   docs: object[],
 *   docIndex: import('fuse.js').default,
 *   sectionIndex: import('fuse.js').default,
 *   sections: object[],
 *   getDocById: (id: string) => object | undefined,
 *   getSectionById: (id: string) => object | undefined,
 * }}
 */
export function loadDocs(docsPath = DEFAULT_DOCS_PATH) {
  if (store) return store;

  let raw;
  try {
    raw = readFileSync(docsPath, 'utf8');
  } catch (err) {
    throw new Error(`docs-loader: cannot read docs file at ${docsPath} — ${err.message}`);
  }

  let manifest;
  try {
    manifest = JSON.parse(raw);
  } catch (err) {
    throw new Error(`docs-loader: docs.json is not valid JSON — ${err.message}`);
  }

  if (manifest.schema_version !== SUPPORTED_SCHEMA_VERSION) {
    throw new Error(
      `docs-loader: unsupported schema_version ${manifest.schema_version} ` +
      `(expected ${SUPPORTED_SCHEMA_VERSION})`
    );
  }

  if (!Array.isArray(manifest.docs) || manifest.docs.length === 0) {
    throw new Error('docs-loader: manifest.docs is empty or missing');
  }

  const { docIndex, sectionIndex, sections } = buildSearchIndex(manifest.docs);

  const docMap     = new Map(manifest.docs.map(d => [d.id, d]));
  const sectionMap = new Map(sections.map(s => [s.id, s]));

  store = Object.freeze({
    manifest,
    docs: manifest.docs,
    docIndex,
    sectionIndex,
    sections,
    getDocById:     (id) => docMap.get(id),
    getSectionById: (id) => sectionMap.get(id),
  });

  return store;
}

/**
 * Drops the singleton — only needed in tests that reload docs mid-run.
 */
export function resetStore() {
  store = null;
}
