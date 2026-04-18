import assert from 'assert/strict';
import { loadDocs, resetStore } from '../lib/docs-loader.js';

let passed = 0;
let failed = 0;

function test(label, fn) {
  try {
    fn();
    console.log(`  ✓ ${label}`);
    passed++;
  } catch (err) {
    console.error(`  ✗ ${label}`);
    console.error(`    ${err.message}`);
    failed++;
  }
}

// ── docs-loader ──────────────────────────────────────────────────────────────

console.log('\ndocs-loader');

resetStore();
const store = loadDocs();

test('returns frozen store', () => {
  assert.ok(Object.isFrozen(store));
});

test('manifest has schema_version 1', () => {
  assert.equal(store.manifest.schema_version, 1);
});

test('docs is non-empty array', () => {
  assert.ok(Array.isArray(store.docs) && store.docs.length > 0);
});

test('each doc has required fields', () => {
  for (const doc of store.docs) {
    assert.ok(doc.id,       `doc missing id`);
    assert.ok(doc.title,    `${doc.id} missing title`);
    assert.ok(doc.summary,  `${doc.id} missing summary`);
    assert.ok(doc.content,  `${doc.id} missing content`);
    assert.ok(Array.isArray(doc.tags),      `${doc.id} tags not array`);
    assert.ok(Array.isArray(doc.sections),  `${doc.id} sections not array`);
  }
});

test('getDocById returns correct doc', () => {
  const doc = store.getDocById('attributes');
  assert.ok(doc, 'attributes doc not found');
  assert.equal(doc.id, 'attributes');
});

test('getDocById returns undefined for unknown id', () => {
  assert.equal(store.getDocById('nonexistent'), undefined);
});

test('sections is flat array with docId attached', () => {
  assert.ok(store.sections.length > 0);
  for (const s of store.sections.slice(0, 5)) {
    assert.ok(s.docId, `section ${s.id} missing docId`);
  }
});

test('getSectionById returns correct section', () => {
  const firstSection = store.sections[0];
  const found = store.getSectionById(firstSection.id);
  assert.ok(found, 'section not found by id');
  assert.equal(found.id, firstSection.id);
});

// ── search-index ─────────────────────────────────────────────────────────────

console.log('\nsearch-index');

test('docIndex.search returns results for "routing"', () => {
  const results = store.docIndex.search('routing');
  assert.ok(results.length > 0, 'expected at least one doc result');
  assert.ok(results[0].item.id, 'result item missing id');
  assert.ok(typeof results[0].score === 'number', 'result missing score');
});

test('docIndex.search returns results for "websocket"', () => {
  const results = store.docIndex.search('websocket');
  assert.ok(results.length > 0, 'expected results for websocket');
});

test('sectionIndex.search returns results for "s-get"', () => {
  const results = store.sectionIndex.search('s-get');
  assert.ok(results.length > 0, 'expected section results for s-get');
  assert.ok(results[0].item.docId, 'section result missing docId');
});

test('singleton: loadDocs() returns same store on second call', () => {
  const store2 = loadDocs();
  assert.equal(store, store2);
});

test('resetStore allows reload', () => {
  resetStore();
  const store3 = loadDocs();
  assert.notEqual(store, store3);
});

// ── summary ──────────────────────────────────────────────────────────────────

console.log(`\n${passed + failed} tests: ${passed} passed, ${failed} failed\n`);
if (failed > 0) process.exit(1);
