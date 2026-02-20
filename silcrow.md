# Silcrow.js is client side library.
Based on the provided `silcrow.js` source code, here is a complete breakdown of all the custom attributes, headers, and events that Silcrow supports.

### 1. Data Binding & Templating Attributes (`s-*`)

These attributes are used for reactive DOM updates and rendering data:

* **`s-bind`**: Binds an element to a data path. It can target the element's text content or a specific property using the syntax `s-bind="path:prop"`.
* **`s-list`**: Applied to a container element to define it as a list that iterates over an array of data. Its value is the data path to the array.
* **`s-template`**: Used alongside `s-list` to specify the ID of the `<template>` element that should be used to render each item in the collection.
* **`s-key`**: Automatically applied internally (and required in your data) to uniquely identify items in a collection for efficient DOM reconciliation.

### 2. Navigation & Routing Attributes (`s-*`)

These attributes control client-side routing, history, and AJAX submissions:

* **`s-action`**: The URL to navigate to or submit to. Adding this attribute opts an element (like a button, link, or form) into Silcrow's routing interception.
* **`s-target`**: A CSS selector specifying which element on the page should be swapped with the incoming HTML or patched with the incoming JSON. If omitted, it defaults to the `document.body` or full page.
* **`s-html`**: Signals to Silcrow that the requested endpoint should return HTML. It modifies the outgoing request headers.
* **`s-timeout`**: Defines a custom timeout duration (in milliseconds) for the specific fetch request. The default is 30,000ms.
* **`s-skip-history`**: When present, prevents the navigation from pushing a new state to the browser's history API.
* **`s-preload`**: When a user hovers (`mouseenter`) over an element with this attribute, Silcrow eagerly fetches and caches the target URL in the background.

### 3. HTTP Headers

Silcrow negotiates with your backend using the following HTTP headers:

**Sent to the Server (Request):**

* **`silcrow-target`**: Always set to `"true"` on Silcrow navigation requests. This allows your backend to distinguish between a standard browser load and a client-side Silcrow swap.
* **`Accept`**: Set to `"text/html"` if the source element has the `s-html` attribute; otherwise, it defaults to `"application/json"`.

**Read from the Server (Response):**

* **`silcrow-cache`**: If the server responds with `silcrow-cache: no-cache`, Silcrow bypasses its internal 5-minute client-side GET cache for that specific response.
* **`Content-Type`**: Silcrow reads this to determine if the response should be parsed as JSON (for data patching) or extracted as HTML (for DOM swapping).

### 4. Global Configuration & State

* **`s-debug`**: If applied to the `document.body` (`<body s-debug>`), Silcrow will output console warnings and throw explicit errors when it encounters invalid paths, missing templates, or forbidden template bindings.
* **`silcrow-loading`**: A CSS class automatically added to the target element while a network request is in flight.
* **`aria-busy="true"`**: Automatically applied to the target element alongside the loading class for accessibility.

### 5. Custom DOM Events

Silcrow dispatches standard DOM events that you can listen to (`document.addEventListener(...)`) to trigger animations, analytics, or custom logic:

* **`silcrow:navigate`**: Fired *before* a request is made. It is cancelable (`e.preventDefault()`), which stops the request entirely.
* **`silcrow:before-swap`**: Fired after the data is received but *before* the DOM is modified. Also cancelable, allowing you to intercept and manually handle the DOM update.
* **`silcrow:load`**: Fired after the DOM has been successfully updated and history has been pushed.
* **`silcrow:error`**: Fired if the network request fails, times out, or throws an exception.
* **`silcrow:patched`**: Fired locally on an element after JSON data binding updates its content.