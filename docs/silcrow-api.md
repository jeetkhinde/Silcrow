## HTML Attributes (Directives)

**Bindings**
- `s-use`
- `:text`
- `:class`
- `:style`
- `:show`
- `:value`
- `:checked`
- `:disabled`
- `:selected`
- `:hidden`
- `:required`
- `:readOnly`
- `:src`
- `:href`
- `:selectedIndex`
- `:key`

**Loops**
- `s-for`

**Verbs (Navigation)**
- `s-get`
- `s-post`
- `s-put`
- `s-delete`
- `s-patch`

**Navigation Modifiers**
- `s-target`
- `s-timeout`
- `s-html`
- `s-skip-history`
- `s-preload`

**Live Connections**
- `s-sse`
- `s-ws`
- `s-wss`

**Atoms / Store**
- `s-bind`

**Debug**
- `s-debug`

---

## Response Headers (Server → Client)

- `silcrow-target` (request header)
- `silcrow-patch`
- `silcrow-invalidate`
- `silcrow-navigate`
- `silcrow-sse`
- `silcrow-ws`
- `silcrow-trigger`
- `silcrow-retarget`
- `silcrow-push`
- `silcrow-cache`

---

## SSE Event Types

- `message` (default)
- `patch`
- `html`
- `invalidate`
- `navigate`
- `custom`

## WebSocket Message Types

- `patch`
- `html`
- `invalidate`
- `navigate`
- `custom`

---

## Custom DOM Events

- `silcrow:patched`
- `silcrow:navigate`
- `silcrow:before-swap`
- `silcrow:load`
- `silcrow:error`
- `silcrow:live:connect`
- `silcrow:live:disconnect`
- `silcrow:sse`
- `silcrow:sse:<event>`
- `silcrow:ws:<event>`
- `silcrow:optimistic`
- `silcrow:revert`

---

## Public API (`window.Silcrow`)

**Runtime**
- `patch`
- `invalidate`
- `stream`

**Navigation**
- `go`

**Live**
- `live`
- `send`
- `disconnect`
- `reconnect`

**Headless Store**
- `prefetch`
- `submit`
- `subscribe`
- `snapshot`
- `publish`

**Feedback**
- `optimistic`
- `revert`
- `onToast`

**Extensibility**
- `use`
- `onRoute`
- `onError`

**Lifecycle**
- `destroy`

---

## Atom Scopes

- `route:<pathname>`
- `stream:<url>`
- `<custom-name>`

## SSR Hydration Globals

- `window.__silcrow_seed`
- `window.__pilcrow_props`

---

## Internal Subsystems

**Debug** — `DEBUG`, `warn`, `throwErr`

**URL Safety** — `URL_SAFE_PROTOCOLS`, `URL_ATTRS`, `SAFE_DATA_IMAGE_RE`, `hasSafeProtocol`, `hasSafeSrcSet`

**Safety** — `extractHTML`, `FORBIDDEN_HTML_TAGS`, `hardenBlankTargets`, `sanitizeTree`, `safeSetHTML`

**Toasts** — `processToasts`, `setToastHandler`, `silcrow_toasts` cookie

**Atoms** — `BLOCKED_ATOM_KEYS`, `isPlainMergeable`, `mergePath`, `createAtom`, `routeAtoms`, `streamAtoms`, `scopeAtoms`, `getOrCreateAtom`, `resolveAtomByScope`, `prefetchPromises`, `prefetchRoute`, `evictPrefetch`, `submitAction`, `bindElementToScope`, `unbindElementAtoms`, `initScopeBindings`, `seedAtomsFromSSR`

**Patcher** — `instanceCache`, `validatedTemplates`, `localBindingsCache`, `identityMap`, `patchMiddleware`, `PATH_RE`, `isValidPath`, `knownProps`, `URL_BINDING_PROPS`, `BLOCKED_KEYS`, `resolvePath`, `resolveRoot`, `getStableId`, `safeClone`, `parseForExpression`, `setValue`, `parseBind`, `scanBindings`, `reconcile`, `patchItem`, `mergeOrRemoveItem`, `buildMaps`, `patch`, `invalidate`, `stream`

**Live (SSE)** — `liveConnections`, `liveConnectionsByUrl`, `sseHubs`, `MAX_BACKOFF`, `isLikelyLiveUrl`, `normalizeSSEEndpoint`, `resolveLiveTarget`, `applyLivePatchPayload`, `pauseLiveState`, `resolveLiveStates`, `onSSEEvent`, `createSseHub`, `getOrCreateSseHub`, `removeSseHub`, `openLive`, `unsubscribeSse`, `connectSseHub`, `disconnectLive`, `reconnectLive`, `destroyAllLive`, `initLiveElements`

**WebSocket** — `wsHubs`, `normalizeWsEndpoint`, `createWsHub`, `getOrCreateWsHub`, `removeWsHub`, `connectWsHub`, `dispatchWsMessage`, `unsubscribeWs`, `openWsLive`, `sendWs`

**Navigator** — `VERB_ATTRS`, `VERB_SELECTOR`, `FORM_VERB_SELECTOR`, `DEFAULT_TIMEOUT`, `CACHE_TTL`, `MAX_CACHE`, `abortMap`, `routeHandler`, `errorHandler`, `responseCache`, `preloadInflight`, `resolveVerb`, `getTarget`, `getTimeout`, `showLoading`, `hideLoading`, `cacheSet`, `cacheGet`, `bustCacheOnMutation`, `processSideEffectHeaders`, `buildFetchOptions`, `processResponseHeaders`, `prepareSwapContent`, `finalizeNavigation`, `navigate`, `onClick`, `onSubmit`, `onPopState`, `onMouseEnter`

**Optimistic** — `snapshots`, `optimisticPatch`, `revertOptimistic`

**Lifecycle** — `liveObserver`, `middlewareLocked`, `init`, `destroy`, auto-boot on `DOMContentLoaded`

**CSS Classes** — `silcrow-loading`

**ARIA** — `aria-busy`