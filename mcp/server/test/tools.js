import assert from 'assert/strict';
import { loadDocs, resetStore } from '../lib/docs-loader.js';
import { handler as searchDocs }   from '../tools/search-docs.js';
import { handler as getDoc }       from '../tools/get-doc.js';
import { handler as getSection }   from '../tools/get-section.js';
import { handler as getExamples }  from '../tools/get-examples.js';
import { handler as analyzeUsage } from '../tools/analyze-usage.js';

resetStore();
const store = loadDocs();

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

// ── searchDocs ───────────────────────────────────────────────────────────────

console.log('\nsearchDocs');

test('returns query in response', () => {
  const r = searchDocs({ query: 'routing' }, store);
  assert.equal(r.query, 'routing');
});

test('finds navigator doc for "routing"', () => {
  const r = searchDocs({ query: 'routing' }, store);
  assert.ok(r.results.length > 0);
  const ids = r.results.map(x => x.id);
  assert.ok(ids.includes('navigator'), `expected navigator, got: ${ids.join(', ')}`);
});

test('results include type field (doc or section)', () => {
  const r = searchDocs({ query: 's-get' }, store);
  for (const result of r.results) {
    assert.ok(['doc', 'section'].includes(result.type), `bad type: ${result.type}`);
  }
});

test('section results have docId', () => {
  const r = searchDocs({ query: 's-post' }, store);
  const sections = r.results.filter(x => x.type === 'section');
  assert.ok(sections.length > 0, 'expected at least one section result');
  for (const s of sections) {
    assert.ok(s.docId, `section missing docId: ${JSON.stringify(s)}`);
  }
});

test('results are sorted by score ascending', () => {
  const r = searchDocs({ query: 'websocket' }, store);
  for (let i = 1; i < r.results.length; i++) {
    assert.ok(r.results[i].score >= r.results[i - 1].score,
      `results not sorted at index ${i}`);
  }
});

test('returns empty results for empty query', () => {
  const r = searchDocs({ query: '   ' }, store);
  assert.deepEqual(r.results, []);
  assert.equal(r.total, 0);
});

test('respects MAX_RESULTS cap', () => {
  const r = searchDocs({ query: 's-' }, store);
  assert.ok(r.results.length <= 15);
});

// ── getDoc ───────────────────────────────────────────────────────────────────

console.log('\ngetDoc');

test('returns doc for valid id', () => {
  const r = getDoc({ id: 'attributes' }, store);
  assert.equal(r.id, 'attributes');
  assert.ok(r.title);
  assert.ok(r.summary);
});

test('returns sections array', () => {
  const r = getDoc({ id: 'navigator' }, store);
  assert.ok(Array.isArray(r.sections) && r.sections.length > 0);
});

test('each section has id, title, level, summary, content, examples', () => {
  const r = getDoc({ id: 'runtime' }, store);
  for (const s of r.sections) {
    assert.ok(s.id,       `section missing id`);
    assert.ok(s.title,    `section missing title`);
    assert.ok(s.level,    `section missing level`);
    assert.ok(s.summary,  `section missing summary`);
    assert.ok(s.content !== undefined, `section missing content`);
    assert.ok(s.examples, `section missing examples`);
  }
});

test('examples shape is { html: [], json: [] }', () => {
  const r = getDoc({ id: 'live' }, store);
  assert.ok(Array.isArray(r.examples.html));
  assert.ok(Array.isArray(r.examples.json));
});

test('returns error for unknown id', () => {
  const r = getDoc({ id: 'nonexistent' }, store);
  assert.ok(r.error, 'expected error field');
  assert.ok(r.error.includes('nonexistent'));
});

test('returns error for empty id', () => {
  const r = getDoc({ id: '' }, store);
  assert.ok(r.error);
});

test('id lookup is case-insensitive', () => {
  const r = getDoc({ id: 'NAVIGATOR' }, store);
  assert.equal(r.id, 'navigator');
});

// ── getSection ───────────────────────────────────────────────────────────────

console.log('\ngetSection');

test('returns section for valid id', () => {
  const first = store.sections[0];
  const r = getSection({ id: first.id }, store);
  assert.equal(r.id, first.id);
  assert.ok(r.title);
  assert.ok(r.docId);
  assert.ok(r.docTitle);
});

test('returns all required fields', () => {
  const first = store.sections[0];
  const r = getSection({ id: first.id }, store);
  assert.ok(r.id       !== undefined, 'missing id');
  assert.ok(r.docId    !== undefined, 'missing docId');
  assert.ok(r.docTitle !== undefined, 'missing docTitle');
  assert.ok(r.title    !== undefined, 'missing title');
  assert.ok(r.level    !== undefined, 'missing level');
  assert.ok(r.summary  !== undefined, 'missing summary');
  assert.ok(r.content  !== undefined, 'missing content');
  assert.ok(r.examples !== undefined, 'missing examples');
  assert.ok(Array.isArray(r.examples.html), 'examples.html not array');
  assert.ok(Array.isArray(r.examples.json), 'examples.json not array');
});

test('does not return full doc sections array', () => {
  const first = store.sections[0];
  const r = getSection({ id: first.id }, store);
  assert.equal(r.sections, undefined, 'should not include sibling sections');
});

test('id lookup is case-insensitive', () => {
  const first = store.sections[0];
  const r = getSection({ id: first.id.toUpperCase() }, store);
  assert.equal(r.id, first.id);
});

test('returns error for unknown id', () => {
  const r = getSection({ id: 'nonexistent/section' }, store);
  assert.ok(r.error);
  assert.ok(r.error.includes('nonexistent/section'));
});

test('returns error for empty id', () => {
  const r = getSection({ id: '' }, store);
  assert.ok(r.error);
});

test('error response includes example section ids', () => {
  const r = getSection({ id: 'bad/id' }, store);
  assert.ok(r.error.includes('/'), 'expected example IDs with slash in error');
});

// ── analyzeSilcrowUsage (size guard) ─────────────────────────────────────────

console.log('\nanalyzeSilcrowUsage size guard');

test('rejects input over 100,000 chars', () => {
  const r = analyzeUsage({ code: 'x'.repeat(100_001) }, store);
  assert.ok(r.error, 'expected error');
  assert.ok(r.error.includes('too large'), `unexpected message: ${r.error}`);
});

test('accepts input at exactly 100,000 chars', () => {
  const r = analyzeUsage({ code: 'x'.repeat(100_000) }, store);
  assert.equal(r.error, undefined, 'should not error at limit');
});

// ── getExamples ──────────────────────────────────────────────────────────────

console.log('\ngetExamples');

test('returns topic in response', () => {
  const r = getExamples({ topic: 's-get' }, store);
  assert.equal(r.topic, 's-get');
});

test('finds examples for "s-get"', () => {
  const r = getExamples({ topic: 's-get' }, store);
  assert.ok(r.examples.length > 0, 'expected examples for s-get');
});

test('each example has docId, type, code', () => {
  const r = getExamples({ topic: 'data binding' }, store);
  for (const ex of r.examples) {
    assert.ok(ex.docId, `example missing docId`);
    assert.ok(['html', 'json'].includes(ex.type), `bad type: ${ex.type}`);
    assert.ok(ex.code,  `example missing code`);
  }
});

test('section examples include sectionTitle', () => {
  const r = getExamples({ topic: 'websocket' }, store);
  const sectionExamples = r.examples.filter(e => e.source === 'section');
  if (sectionExamples.length > 0) {
    for (const ex of sectionExamples) {
      assert.ok(ex.sectionTitle, `section example missing sectionTitle`);
    }
  }
});

test('no duplicate examples', () => {
  const r = getExamples({ topic: 'routing' }, store);
  const codes = r.examples.map(e => e.code);
  const unique = new Set(codes);
  assert.equal(unique.size, codes.length, 'duplicate examples found');
});

test('returns empty for empty topic', () => {
  const r = getExamples({ topic: '' }, store);
  assert.deepEqual(r.examples, []);
});

// ── summary ──────────────────────────────────────────────────────────────────

console.log(`\n${passed + failed} tests: ${passed} passed, ${failed} failed\n`);
if (failed > 0) process.exit(1);
