# RHTMX Architecture Diagrams

## 1. Request-Response Cycle

```
┌─────────────────────────────────────────────────────────────────┐
│                    Browser/Client (HTMX)                        │
│  User clicks button with hx-post="/users"                       │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      │ HTTP POST Request + Headers
                      │ Content-Type: application/x-www-form-urlencoded
                      │ HX-Request: true
                      │ HX-Target: #user-list
                      │ HX-Swap: beforeend
                      │ Body: name=John&email=john@example.com
                      │
┌─────────────────────▼───────────────────────────────────────────┐
│                    Axum Web Framework                            │
├─────────────────────────────────────────────────────────────────┤
│  1. Route Matching (rhtmx-router)                               │
│     - Match /users to file handler                              │
│     - Extract path parameters                                  │
│                                                                  │
│  2. Request Context Creation                                    │
│     - Parse form data (name, email)                            │
│     - Extract headers (HTMX metadata)                          │
│     - Parse cookies                                            │
│     - Get database pool reference                              │
│                                                                  │
│  3. HTTP Handler Execution                                      │
│     post!()                                                     │
│     fn create_user(req: CreateUserRequest) -> OkResponse {      │
│         let user = db::create_user(&ctx.db, req)?;             │
│         Ok()                                                    │
│             .render(user_card, user)                           │
│             .render_oob("user-count", count_badge, count)      │
│             .toast("User created!")                            │
│     }                                                           │
│                                                                  │
│  4. Template Rendering                                         │
│     - Compile-time HTML generation                            │
│     - Process r-for, r-if directives                          │
│     - Interpolate expressions                                 │
│     - Apply scoped CSS                                        │
│                                                                  │
│  5. Response Building                                          │
│     - Set HTTP status code                                    │
│     - Build response HTML:                                    │
│       • Main swap (innerHTML to #user-list)                   │
│       • OOB update (outerHTML to #user-count)                │
│     - Add HX-Trigger header for toast                         │
│                                                                  │
│  6. Database Interaction                                       │
│     - Execute: INSERT INTO users (name, email) VALUES (?, ?)  │
│     - Get last_insert_rowid()                                 │
│     - Return User struct                                      │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      │ HTTP 200 Response
                      │ Content-Type: text/html
                      │ HX-Trigger: {"showToast": {"message": "User created!"}}
                      │
                      │ Body (HTML):
                      │ <div id="user-123" class="user-card">
                      │   <h3>John Doe</h3>
                      │   <p>john@example.com</p>
                      │   <button hx-delete="/users/123">Delete</button>
                      │ </div>
                      │
                      │ <!-- OOB Update -->
                      │ <div id="user-count" hx-swap-oob="outerHTML">
                      │   <div class="count">Total: 5</div>
                      │ </div>
                      │
┌─────────────────────▼───────────────────────────────────────────┐
│                    Browser/Client (HTMX)                        │
│  HTMX processes response:                                       │
│  - Swaps <div id="user-list"> beforeend with new user card     │
│  - Swaps <div id="user-count"> with new count                  │
│  - Shows toast notification with message                       │
│                                                                  │
│  User sees new user in list and count updated                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Component Rendering Flow

```
┌──────────────────────────────────┐
│  Rust Handler Function            │
│  post!()                          │
│  fn create_user(req: ...) { ... } │
└────────────┬─────────────────────┘
             │
             │ Call: Ok().render(user_card, user_data)
             │
┌────────────▼─────────────────────┐
│  Response Builder (Html type)     │
│  - Store component function       │
│  - Store properties data          │
│  - Queue render operations        │
└────────────┬─────────────────────┘
             │
             │ Build response (into_response)
             │
┌────────────▼─────────────────────┐
│  Component Function Execution     │
│  fn user_card(user: &User) -> Html│
│                                   │
│  html! {                          │
│    <div id="user-{user.id}">      │
│      <h3>{user.name}</h3>         │
│      <p>{user.email}</p>          │
│      <button hx-delete="...">     │
│        Delete                     │
│      </button>                    │
│    </div>                         │
│  }                                │
└────────────┬─────────────────────┘
             │
             │ Compile-time macro expansion
             │
┌────────────▼─────────────────────┐
│  Expanded Rust Code               │
│  let mut __html = String::new();  │
│  __html.push_str(                 │
│    "<div id=\"user-"              │
│  );                               │
│  __html.push_str(&user.id.to_...);│
│  __html.push_str("\">");          │
│  __html.push_str("<h3>");         │
│  __html.push_str(&user.name);     │
│  __html.push_str("</h3>");        │
│  // ... more string building      │
│  Html(__html)                     │
└────────────┬─────────────────────┘
             │
             │ Runtime: String concatenation
             │
┌────────────▼─────────────────────┐
│  Final HTML String                │
│  <div id="user-123">              │
│    <h3>John Doe</h3>              │
│    <p>john@example.com</p>        │
│    <button hx-delete="/users/123" │
│      hx-target="#user-123"        │
│      hx-swap="outerHTML">         │
│      Delete                       │
│    </button>                      │
│  </div>                           │
└──────────────────────────────────┘
```

---

## 3. File-Based Routing Structure

```
Project Root
├── pages/                          ← Routes generated from here
│   ├── index.rhtml                 → GET /
│   ├── _layout.rhtml               → Default layout (wraps all pages)
│   ├── _error.rhtml                → Error page fallback
│   │
│   ├── users/
│   │   ├── index.rhtml             → GET /users
│   │   ├── [id].rhtml              → GET /users/:id (dynamic)
│   │   └── _layout.rhtml           → Layout for /users/* (overrides parent)
│   │
│   ├── blog/
│   │   ├── [slug].rhtml            → GET /blog/:slug
│   │   └── comments/
│   │       └── [post_id].rhtml     → GET /blog/comments/:post_id
│   │
│   └── docs/
│       └── [...path].rhtml         → GET /docs/* (catch-all)
│
├── ui/                             ← Reusable components
│   ├── user_card.rhtml
│   ├── pagination.rhtml
│   └── forms/
│       └── user_form.rhtml
│
└── static/                         ← Static assets
    ├── css/
    ├── js/
    └── images/

Route Resolution Logic:
────────────────────────

1. GET /users/123 
   → Try exact match: users/[id].rhtml
   → Extract param: id = "123"
   → Check layout: users/_layout.rhtml (if exists)
   → Fall back: _layout.rhtml

2. GET /blog/hello-world
   → Try exact match: blog/[slug].rhtml
   → Extract param: slug = "hello-world"
   → Use _layout.rhtml from root

3. GET /docs/tutorial/intro
   → Try exact match: docs/[...path].rhtml
   → Extract param: path = "tutorial/intro"
   → Matches catch-all pattern
```

---

## 4. Data Flow: Database to Response

```
┌─────────────────────────────────────────────────────────────┐
│ SQL Query (in Handler)                                      │
│ sqlx::query_as::<_, User>(                                  │
│   "SELECT id, name, email, age, bio, username FROM users"   │
│ ).fetch_all(&pool).await?                                   │
└────────────┬────────────────────────────────────────────────┘
             │
             │ Returns: Result<Vec<User>, sqlx::Error>
             │
┌────────────▼────────────────────────────────────────────────┐
│ Rust Value: Vec<User>                                       │
│ [                                                            │
│   User {                                                     │
│     id: 1,                                                   │
│     name: "Alice",                                           │
│     email: "alice@example.com",                              │
│     age: 30,                                                 │
│     bio: Some("Developer"),                                  │
│     username: "alice"                                        │
│   },                                                         │
│   User {                                                     │
│     id: 2,                                                   │
│     name: "Bob",                                             │
│     email: "bob@example.com",                                │
│     age: 25,                                                 │
│     bio: None,                                               │
│     username: "bob"                                          │
│   }                                                          │
│ ]                                                            │
└────────────┬────────────────────────────────────────────────┘
             │
             │ Pass to component function
             │
┌────────────▼────────────────────────────────────────────────┐
│ Component Rendering:                                         │
│ fn users_page(users: Vec<User>) -> Html {                    │
│   html! {                                                    │
│     <div class="users">                                      │
│       <h1>Users</h1>                                         │
│       <ul>                                                   │
│         <li r-for="user in users">                           │
│           <div id="user-{user.id}">                          │
│             <h3>{user.name}</h3>                             │
│             <p>{user.email}</p>                              │
│             <button hx-delete="/users/{user.id}">            │
│               Delete                                         │
│             </button>                                        │
│           </div>                                             │
│         </li>                                                │
│       </ul>                                                  │
│     </div>                                                   │
│   }                                                          │
│ }                                                            │
└────────────┬────────────────────────────────────────────────┘
             │
             │ Macro expansion: r-for becomes for loop
             │
┌────────────▼────────────────────────────────────────────────┐
│ Generated Rust Code:                                         │
│ let mut __html = String::new();                              │
│ __html.push_str("<div class=\"users\"><h1>Users</h1><ul>");  │
│                                                              │
│ for user in users {  // r-for expansion                      │
│   __html.push_str("<li><div id=\"user-");                    │
│   __html.push_str(&user.id.to_string());                     │
│   __html.push_str("\"><h3>");                                │
│   __html.push_str(&user.name);                               │
│   __html.push_str("</h3><p>");                               │
│   __html.push_str(&user.email);                              │
│   __html.push_str("</p><button hx-delete=\"/users/");        │
│   __html.push_str(&user.id.to_string());                     │
│   __html.push_str("\">Delete</button></div></li>");          │
│ }                                                            │
│                                                              │
│ __html.push_str("</ul></div>");                              │
│ Html(__html)                                                 │
└────────────┬────────────────────────────────────────────────┘
             │
             │ Runtime execution: String concatenation
             │
┌────────────▼────────────────────────────────────────────────┐
│ Final HTML String:                                           │
│ <div class="users">                                          │
│   <h1>Users</h1>                                             │
│   <ul>                                                       │
│     <li>                                                     │
│       <div id="user-1">                                      │
│         <h3>Alice</h3>                                       │
│         <p>alice@example.com</p>                             │
│         <button hx-delete="/users/1">Delete</button>         │
│       </div>                                                 │
│     </li>                                                    │
│     <li>                                                     │
│       <div id="user-2">                                      │
│         <h3>Bob</h3>                                         │
│         <p>bob@example.com</p>                               │
│         <button hx-delete="/users/2">Delete</button>         │
│       </div>                                                 │
│     </li>                                                    │
│   </ul>                                                      │
│ </div>                                                       │
└────────────┬────────────────────────────────────────────────┘
             │
             │ HTTP Response
             │
┌────────────▼────────────────────────────────────────────────┐
│ Browser receives HTML and HTMX processes it:                │
│ - Parses HTML                                                │
│ - Updates DOM                                                │
│ - Binds HTMX event handlers to new elements                 │
│ - Renders on screen                                          │
└──────────────────────────────────────────────────────────────┘
```

---

## 5. Technology Stack Layers

```
┌─────────────────────────────────────────────────────────────┐
│                     Browser Layer                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ HTMX 1.x - Lightweight AJAX library                  │   │
│  │ - Adds hx-get, hx-post, hx-delete attributes        │   │
│  │ - Handles HTTP requests and DOM swaps                │   │
│  │ - No framework overhead                              │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ HTML/CSS/Vanilla JS only                             │   │
│  │ - No React, Vue, Angular, Svelte                     │   │
│  │ - Progressive enhancement                            │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                          ▲
                          │ HTTP
                          │
┌─────────────────────────────────────────────────────────────┐
│                    Server Layer                             │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐     │
│ │ Axum 0.7 - Fast async web framework                 │     │
│ │ - Routing, middleware, extractors                   │     │
│ │ - Built on Tower ecosystem                          │     │
│ │ - Zero-copy abstractions                            │     │
│ └─────────────────────────────────────────────────────┘     │
│ ┌─────────────────────────────────────────────────────┐     │
│ │ RHTMX-specific layers:                              │     │
│ │ - Router: File-based routing (rhtmx-router)        │     │
│ │ - Macros: HTML generation (rhtmx-macro)            │     │
│ │ - Parser: Template processing (rhtmx-parser)       │     │
│ │ - Renderer: HTML compilation to string             │     │
│ │ - Validation: Form validation (derive macro)        │     │
│ └─────────────────────────────────────────────────────┘     │
│ ┌─────────────────────────────────────────────────────┐     │
│ │ SQLx 0.7 - Compile-time SQL verification           │     │
│ │ - Type-safe database queries                        │     │
│ │ - Connection pooling                                │     │
│ │ - Async/await support with Tokio                    │     │
│ └─────────────────────────────────────────────────────┘     │
│ ┌─────────────────────────────────────────────────────┐     │
│ │ Tokio 1.x - Async runtime                           │     │
│ │ - Multithreaded executor                            │     │
│ │ - Green threads (async tasks)                       │     │
│ │ - Timer, networking, file I/O                       │     │
│ └─────────────────────────────────────────────────────┘     │
│ ┌─────────────────────────────────────────────────────┐     │
│ │ Supporting crates:                                   │     │
│ │ - serde/serde_json: Serialization                   │     │
│ │ - regex: Pattern matching (validators)              │     │
│ │ - notify: File watching (hot reload)                │     │
│ │ - tower-livereload: Browser auto-reload             │     │
│ │ - chrono: Timestamps                                │     │
│ │ - uuid: ID generation                               │     │
│ └─────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
                          ▲
                          │
┌─────────────────────────────────────────────────────────────┐
│                    Database Layer                           │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐     │
│ │ SQLite - Embedded relational database               │     │
│ │ - File-based (rhtmx.db)                             │     │
│ │ - Zero-configuration                                │     │
│ │ - Good for small to medium apps                     │     │
│ │ - Supports: PostgreSQL, MySQL via SQLx              │     │
│ └─────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

---

## 6. State Management Flow

```
Traditional SPA Model (Not RHTMX)
──────────────────────────────────
Browser                    Server                  Database
  │                          │                         │
  ├─ GET /api/users         │                         │
  │─────────────────────────>│                         │
  │                          ├─ SELECT * FROM users   │
  │                          │────────────────────────>│
  │                          │<──── [User, User] ─────│
  │<─── JSON [User, User] ───│                        │
  │                          │                        │
  ├─ Store in Memory         │                        │
  │ (React State)            │                        │
  │                          │                        │
  ├─ Render Components       │                        │
  │ (in Browser)             │                        │
  │                          │                        │
  └─ Display to user         │                        │


RHTMX Model (Server-Driven)
────────────────────────────
Browser                    Server                  Database
  │                          │                         │
  ├─ GET /users              │                         │
  │─────────────────────────>│                         │
  │                          ├─ SELECT * FROM users   │
  │                          │────────────────────────>│
  │                          │<──── [User, User] ─────│
  │                          │                        │
  │                          ├─ Render HTML:          │
  │                          │  <div class="users">   │
  │                          │    <div>Alice</div>    │
  │                          │    <div>Bob</div>      │
  │                          │  </div>                │
  │                          │                        │
  │<─ HTML [pre-rendered] ───│                        │
  │                          │                        │
  ├─ Update DOM              │                        │
  │ (HTMX swaps)             │                        │
  │                          │                        │
  └─ Display to user         │                        │
                             
Key Difference:
- SPA: Browser renders (needs state management)
- RHTMX: Server renders (no client state)
```

---

## 7. Form Validation Flow

```
┌─────────────────────────────────────────────┐
│ Define Form with Validators                 │
├─────────────────────────────────────────────┤
│ #[derive(Validate, Deserialize)]            │
│ struct CreateUserRequest {                  │
│     #[min_length(3)]                        │
│     #[max_length(50)]                       │
│     name: String,                           │
│                                              │
│     #[email]                                │
│     #[no_public_domains]                    │
│     email: String,                          │
│                                              │
│     #[password("strong")]                   │
│     password: String,                       │
│                                              │
│     #[min(18)]                              │
│     #[max(120)]                             │
│     age: i32,                               │
│ }                                           │
└────────────┬────────────────────────────────┘
             │
             │ Macro expansion generates validate() method
             │ at compile time
             │
┌────────────▼────────────────────────────────┐
│ Browser sends POST request with form data   │
│ Content-Type: application/x-www-form-...   │
│ name=John&email=john@example.com&...        │
└────────────┬────────────────────────────────┘
             │
             │ Server receives & deserializes
             │
┌────────────▼────────────────────────────────┐
│ Handler calls validate()                    │
├─────────────────────────────────────────────┤
│ post!()                                     │
│ fn create_user(req: CreateUserRequest) {    │
│     if let Err(errors) = req.validate() {   │
│         return Error()                      │
│             .render(error_component, errors)│
│             .status(BadRequest);            │
│     }                                       │
│     // ... process valid request            │
│ }                                           │
└────────────┬────────────────────────────────┘
             │
             │ validate() method:
             │ 1. Check name length: 3-50 chars
             │ 2. Validate email format
             │ 3. Check email domain
             │ 4. Validate password strength
             │ 5. Check age range: 18-120
             │
┌────────────▼────────────────────────────────┐
│ Validation Results                          │
├─────────────────────────────────────────────┤
│ Ok(()) → Process request                    │
│         → Insert into database              │
│         → Return success response           │
│                                              │
│ Err(HashMap of errors):                     │
│ {                                           │
│   "name": "Must be at least 3 characters",  │
│   "email": "Invalid email address",         │
│   "password": "Password must be at least..."│
│ }                                           │
│         → Render error component            │
│         → Return HTML with form & errors    │
└────────────┬────────────────────────────────┘
             │
             │ HTTP response with HTML
             │
┌────────────▼────────────────────────────────┐
│ Browser receives HTML                       │
│ - HTMX swaps response into form             │
│ - Shows error messages                      │
│ - User corrects and resubmits               │
└──────────────────────────────────────────────┘
```

---

## 8. Comparison: Request Paths

```
HTMX Request (Partial):
──────────────────────
User clicks: <button hx-post="/users" hx-target="#list" hx-swap="beforeend">

Browser sends:
  POST /users
  Headers: HX-Request: true
  Headers: HX-Target: #list
  Body: form data

Server receives:
  1. Identifies as HTMX request
  2. Skips layout rendering
  3. Renders component only
  4. Returns: <div class="user-card">...</div>

HTMX processes:
  1. Insert HTML beforeend to #list
  2. No page reload
  3. Preserves state

───────────────────

Traditional Page Request:
──────────────────────────
User navigates to: /users

Browser sends:
  GET /users
  Standard headers

Server receives:
  1. No HX-Request header
  2. Renders full page with layout
  3. Returns complete HTML with <html>, <head>, <body>

Browser processes:
  1. Full page reload
  2. Replaces entire DOM
  3. Re-executes scripts

───────────────────

API Request (Not RHTMX):
────────────────────────
Browser JavaScript sends:
  GET /api/users
  Headers: Accept: application/json

Server receives:
  1. Routing to API handler
  2. Returns JSON
  3. Custom API layer (different from RHTMX)

JavaScript processes:
  1. Parses JSON
  2. Updates React state
  3. Re-renders components
```

