/**
 * Ground-truth catalog derived from mcp/generated/docs.json.
 * Each entry maps a construct to the doc page that documents it.
 */

export const KNOWN_ATTRIBUTES = new Map([
  ['s-debug',        'attributes'],
  ['s-delete',       'navigator'],
  ['s-for',          'runtime'],
  ['s-get',          'navigator'],
  ['s-html',         'attributes'],
  ['s-patch',        'navigator'],
  ['s-post',         'navigator'],
  ['s-preload',      'navigator'],
  ['s-put',          'navigator'],
  ['s-skip-history', 'navigator'],
  ['s-sse',          'live'],
  ['s-target',       'navigator'],
  ['s-timeout',      'navigator'],
  ['s-use',          'runtime'],
  ['s-ws',           'live'],
  ['s-wss',          'live'],
]);

export const KNOWN_BINDINGS = new Map([
  [':class',    'runtime'],
  [':disabled', 'runtime'],
  [':email',    'runtime'],
  [':hidden',   'runtime'],
  [':key',      'runtime'],
  [':name',     'runtime'],
  [':show',     'runtime'],
  [':style',    'runtime'],
  [':text',     'runtime'],
  [':value',    'runtime'],
]);

export const KNOWN_API_METHODS = new Map([
  ['Silcrow.destroy',    'javascript-api'],
  ['Silcrow.disconnect', 'javascript-api'],
  ['Silcrow.go',         'javascript-api'],
  ['Silcrow.invalidate', 'javascript-api'],
  ['Silcrow.live',       'javascript-api'],
  ['Silcrow.onError',    'javascript-api'],
  ['Silcrow.onRoute',    'javascript-api'],
  ['Silcrow.onToast',    'javascript-api'],
  ['Silcrow.optimistic', 'optimistic'],
  ['Silcrow.patch',      'javascript-api'],
  ['Silcrow.reconnect',  'javascript-api'],
  ['Silcrow.revert',     'optimistic'],
  ['Silcrow.send',       'live'],
  ['Silcrow.stream',     'live'],
  ['Silcrow.use',        'javascript-api'],
]);

export const HTTP_VERB_ATTRIBUTES = new Set([
  's-get', 's-post', 's-put', 's-patch', 's-delete',
]);

/** Attributes removed or renamed in past versions. */
export const DEPRECATED_ATTRIBUTES = new Map([
  ['s-action', { replacement: 's-get or s-post', doc: 'navigator' }],
  ['s-src',    { replacement: 's-get',           doc: 'navigator' }],
  ['s-href',   { replacement: 's-get',           doc: 'navigator' }],
]);
