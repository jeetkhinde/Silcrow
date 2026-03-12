// ./public/silcrow/debug.js
// ════════════════════════════════════════════════════════════
// Debug — shared diagnostics
// ════════════════════════════════════════════════════════════

const DEBUG = document.body.hasAttribute("s-debug");

function warn(msg) {
  if (DEBUG) console.warn("[silcrow]", msg);
}

function throwErr(msg) {
  if (DEBUG) throw new Error("[silcrow] " + msg);
}
