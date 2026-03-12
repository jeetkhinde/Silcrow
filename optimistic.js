// silcrow/optimistic.js
// ════════════════════════════════════════════════════════════
// Optimistic — snapshot & revert for instant UI feedback
// ════════════════════════════════════════════════════════════

const snapshots = new WeakMap();

function optimisticPatch(root, data) {
  const element = typeof root === "string" ? document.querySelector(root) : root;
  if (!element) {
    warn("Optimistic root not found: " + root);
    return;
  }

  // Snapshot current DOM state
  snapshots.set(element, element.innerHTML);

  // Apply the optimistic data
  patch(data, element);

  document.dispatchEvent(
    new CustomEvent("silcrow:optimistic", {
      bubbles: true,
      detail: {root: element, data},
    })
  );
}

function revertOptimistic(root) {
  const element = typeof root === "string" ? document.querySelector(root) : root;
  if (!element) {
    warn("Revert root not found: " + root);
    return;
  }

  const saved = snapshots.get(element);
  if (saved === undefined) {
    warn("No snapshot to revert for element");
    return;
  }

  element.innerHTML = saved;
  snapshots.delete(element);
  invalidate(element);

  document.dispatchEvent(
    new CustomEvent("silcrow:revert", {
      bubbles: true,
      detail: {root: element},
    })
  );
}