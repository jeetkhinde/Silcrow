import assert from 'assert/strict';
import { loadDocs, resetStore } from '../lib/docs-loader.js';
import { parseConstructs }      from '../lib/code-parser.js';
import { applyRules }           from '../lib/analysis-rules.js';
import { handler as analyze }   from '../tools/analyze-usage.js';

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

// ── code-parser ──────────────────────────────────────────────────────────────

console.log('\ncode-parser');

const HTML_SAMPLE = `
<ul s-for="items">
  <li :text="name" :class="active"></li>
</ul>
<a s-get="/api/posts" s-target="#content">Load</a>
`;

const JS_SAMPLE = `
Silcrow.patch('#app', data);
Silcrow.onRoute(handler);
`;

test('detects s-* attributes', () => {
  const { attributes } = parseConstructs(HTML_SAMPLE);
  assert.ok(attributes.has('s-for'),    'missing s-for');
  assert.ok(attributes.has('s-get'),    'missing s-get');
  assert.ok(attributes.has('s-target'), 'missing s-target');
});

test('detects colon bindings', () => {
  const { bindings } = parseConstructs(HTML_SAMPLE);
  assert.ok(bindings.has(':text'),  'missing :text');
  assert.ok(bindings.has(':class'), 'missing :class');
});

test('detects Silcrow API calls', () => {
  const { apiCalls } = parseConstructs(JS_SAMPLE);
  assert.ok(apiCalls.has('Silcrow.patch'),   'missing Silcrow.patch');
  assert.ok(apiCalls.has('Silcrow.onRoute'), 'missing Silcrow.onRoute');
});

test('counts occurrences correctly', () => {
  const code = '<a s-get="/a"></a><a s-get="/b"></a>';
  const { attributes } = parseConstructs(code);
  assert.equal(attributes.get('s-get'), 2);
});

test('does not match colons inside URLs', () => {
  const code = '<a href="https://example.com" s-get="/x">link</a>';
  const { bindings } = parseConstructs(code);
  assert.ok(!bindings.has(':'), 'spurious colon match from URL');
  assert.ok(!bindings.has('://'), 'spurious colon match from URL');
});

test('handles empty code gracefully', () => {
  const { attributes, bindings, apiCalls } = parseConstructs('');
  assert.equal(attributes.size, 0);
  assert.equal(bindings.size, 0);
  assert.equal(apiCalls.size, 0);
});

// ── analysis-rules ───────────────────────────────────────────────────────────

console.log('\nanalysis-rules');

test('flags unknown s-* attribute as error', () => {
  const constructs = parseConstructs('<div s-magic="x"></div>');
  const issues = applyRules(constructs, '');
  const issue = issues.find(i => i.rule === 'unknown-attribute');
  assert.ok(issue, 'expected unknown-attribute issue');
  assert.equal(issue.severity, 'error');
  assert.ok(issue.message.includes('s-magic'));
});

test('flags deprecated s-action as warning', () => {
  const constructs = parseConstructs('<a s-action="/x">click</a>');
  const issues = applyRules(constructs, '<a s-action="/x">click</a>');
  const issue = issues.find(i => i.rule === 'deprecated-attribute');
  assert.ok(issue, 'expected deprecated-attribute issue');
  assert.equal(issue.severity, 'warning');
});

test('flags s-for without :key', () => {
  const code = '<ul s-for="items"><li :text="name"></li></ul>';
  const constructs = parseConstructs(code);
  const issues = applyRules(constructs, code);
  const issue = issues.find(i => i.rule === 'sfor-missing-key');
  assert.ok(issue, 'expected sfor-missing-key issue');
  assert.equal(issue.severity, 'warning');
});

test('no sfor-missing-key when :key is present', () => {
  const code = '<ul s-for="items"><li :key="id" :text="name"></li></ul>';
  const constructs = parseConstructs(code);
  const issues = applyRules(constructs, code);
  assert.ok(!issues.find(i => i.rule === 'sfor-missing-key'));
});

test('flags multiple HTTP verb attrs on single element', () => {
  const code = '<a s-get="/x" s-post="/y">click</a>';
  const constructs = parseConstructs(code);
  const issues = applyRules(constructs, code);
  const issue = issues.find(i => i.rule === 'multiple-http-verbs');
  assert.ok(issue, 'expected multiple-http-verbs issue');
  assert.equal(issue.severity, 'warning');
});

test('no multiple-verb issue when verbs are on separate elements', () => {
  const code = '<a s-get="/x">get</a><form s-post="/y">post</form>';
  const constructs = parseConstructs(code);
  const issues = applyRules(constructs, code);
  assert.ok(!issues.find(i => i.rule === 'multiple-http-verbs'));
});

test('flags s-html as security info', () => {
  const code = '<div s-html="content"></div>';
  const constructs = parseConstructs(code);
  const issues = applyRules(constructs, code);
  const issue = issues.find(i => i.rule === 'shtml-xss-note');
  assert.ok(issue, 'expected shtml-xss-note issue');
  assert.equal(issue.severity, 'info');
});

test('flags s-ws with non-wss URL as info', () => {
  const code = '<div s-ws="ws://localhost:4000/chat"></div>';
  const constructs = parseConstructs(code);
  const issues = applyRules(constructs, code);
  const issue = issues.find(i => i.rule === 'ws-not-secure');
  assert.ok(issue, 'expected ws-not-secure issue');
});

test('no ws-not-secure when wss:// is used', () => {
  const code = '<div s-ws="wss://example.com/chat"></div>';
  const constructs = parseConstructs(code);
  const issues = applyRules(constructs, code);
  assert.ok(!issues.find(i => i.rule === 'ws-not-secure'));
});

test('flags unknown Silcrow API call as error', () => {
  const code = 'Silcrow.explode()';
  const constructs = parseConstructs(code);
  const issues = applyRules(constructs, code);
  const issue = issues.find(i => i.rule === 'unknown-api-call');
  assert.ok(issue, 'expected unknown-api-call issue');
  assert.equal(issue.severity, 'error');
});

// ── analyzeSilcrowUsage tool ─────────────────────────────────────────────────

console.log('\nanalyzeSilcrowUsage');

const FULL_SAMPLE = `
<ul s-for="posts">
  <li :key="id" :text="title"></li>
</ul>
<a s-get="/api/posts" s-target="#feed" s-preload>Load</a>
`;

test('returns error for empty code', () => {
  const r = analyze({ code: '' }, store);
  assert.ok(r.error);
});

test('summary contains construct_count and issue_count', () => {
  const r = analyze({ code: FULL_SAMPLE }, store);
  assert.ok(typeof r.summary.construct_count === 'number');
  assert.ok(typeof r.summary.issue_count     === 'number');
});

test('summary.attributes lists detected s-* names', () => {
  const r = analyze({ code: FULL_SAMPLE }, store);
  assert.ok(r.summary.attributes.includes('s-for'));
  assert.ok(r.summary.attributes.includes('s-get'));
});

test('constructs array has type/name/count fields', () => {
  const r = analyze({ code: FULL_SAMPLE }, store);
  for (const c of r.constructs) {
    assert.ok(c.type,            `construct missing type`);
    assert.ok(c.name,            `construct missing name`);
    assert.ok(typeof c.count === 'number', `construct missing count`);
  }
});

test('known constructs include doc and doc_title', () => {
  const r = analyze({ code: FULL_SAMPLE }, store);
  const sGet = r.constructs.find(c => c.name === 's-get');
  assert.ok(sGet,           's-get construct not found');
  assert.ok(sGet.doc,       's-get missing doc');
  assert.ok(sGet.doc_title, 's-get missing doc_title');
});

test('relevant_docs is populated for known constructs', () => {
  const r = analyze({ code: FULL_SAMPLE }, store);
  assert.ok(r.relevant_docs.length > 0, 'expected relevant docs');
  const ids = r.relevant_docs.map(d => d.id);
  assert.ok(ids.includes('navigator'), 'expected navigator in relevant docs');
  assert.ok(ids.includes('runtime'),   'expected runtime in relevant docs');
});

test('issues array is present (may be empty for clean code)', () => {
  const r = analyze({ code: FULL_SAMPLE }, store);
  assert.ok(Array.isArray(r.issues));
});

test('detects real issues in bad code', () => {
  const bad = '<div s-magic="x" s-action="/y"></div>';
  const r = analyze({ code: bad }, store);
  assert.ok(r.issues.length > 0, 'expected issues for bad code');
  const rules = r.issues.map(i => i.rule);
  assert.ok(rules.includes('unknown-attribute'),  'expected unknown-attribute');
  assert.ok(rules.includes('deprecated-attribute'), 'expected deprecated-attribute');
});

// ── summary ──────────────────────────────────────────────────────────────────

console.log(`\n${passed + failed} tests: ${passed} passed, ${failed} failed\n`);
if (failed > 0) process.exit(1);
