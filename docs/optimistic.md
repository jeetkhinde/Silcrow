# **Optimistic Updates**

> **Source Module(s):** [`src/optimistic.js`](../src/optimistic.js)

## **Silcrow.optimistic(data, root)**

Takes a snapshot of the root element's current DOM state, then immediately patches the data. Use this for instant UI feedback before the server confirms:

* **Stability**: Uses the same `:key` identity logic as standard patches, ensuring list items don't jump or flicker.

* **Visuals**: Ideal for toggling `:show` states or updating `:text` counters immediately on click.

```javascript
// User clicks "like" — update immediately
Silcrow.optimistic({ likes: currentLikes + 1, liked: true }, "#post-42");

// Send to server
Silcrow.go("/api/posts/42/like", { method: "POST", target: "#post-42" });

```

## **Silcrow.revert(root)**

Restores the DOM to the state captured by `Silcrow.optimistic()`. Call this when the server request fails:

```javascript
try {
  await fetch("/api/posts/42/like", { method: "POST" });
} catch (err) {
  Silcrow.revert("#post-42");
  showError("Failed to save");
}

```

revert() restores the element's innerHTML and calls Silcrow.invalidate() to rebuild binding maps since the DOM was replaced.

## **Optimistic + Error Handler Pattern**

Combine optimistic updates with the error handler for a clean pattern:

```javascript
Silcrow.onError((err, { url, target }) => {
  // Revert any optimistic updates on the failed target
  Silcrow.revert(target);
});

```
