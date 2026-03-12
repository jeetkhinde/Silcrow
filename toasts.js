// silcrow/toasts.js
// ════════════════════════════════════════════════════════════
// Toasts — notification processing
// ════════════════════════════════════════════════════════════

let toastHandler = null;

function processToasts(isJSON, content = null) {
  if (!toastHandler) return;

  if (isJSON && content && content._toasts) {
    content._toasts.forEach(t => toastHandler(t.message, t.level));
    delete content._toasts;

    if (content.data !== undefined && Object.keys(content).length === 1) {
      Object.assign(content, content.data);
      delete content.data;
    }
  } else if (!isJSON) {
    const match = document.cookie.match(new RegExp('(^|;\\s*)silcrow_toasts=([^;]+)'));
    if (match) {
      try {
        const rawJSON = decodeURIComponent(match[2]);
        const toasts = JSON.parse(rawJSON);
        toasts.forEach(t => toastHandler(t.message, t.level));
      } catch (e) {
        console.error("Failed to parse toasts", e);
      }
      document.cookie = "silcrow_toasts=; Max-Age=0; path=/";
    }
  }
}

function setToastHandler(handler) {
  toastHandler = handler;
  processToasts(false);
}
