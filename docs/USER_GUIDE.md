# Light Postman — User Guide

## 1. Introduction
Light Postman is a fast, local-first API client and testing suite built with Rust, Tauri 2, and SvelteKit. It offers full Postman compatibility, offline-first SQLite persistence, sandboxed JavaScript tests, and local AI assistance without recurring cloud subscriptions or forced accounts.

---

## 2. Getting Started

### Launching the Application
- **Development**: Run `npm run tauri dev`
- **Installed Package**: Launch `postman-client` from your system application menu or desktop shortcut.

### Interface Overview
- **Left Sidebar**:
  - **Projects & Collections**: Hierarchy of collections, folders, and HTTP requests.
  - **Environments**: Switch active environments (e.g., Development, Staging, Production).
  - **Git Sync**: Direct Git repository integration, branch switcher, and push/pull sync.
  - **Diagnostics & Status**: Real-time memory footprint, database storage size, and console logs.
- **Top Tab Bar**:
  - Open multiple requests in persistent tabs.
  - Keyboard shortcuts: `Ctrl+T` (new request), `Ctrl+W` (close active tab), `Ctrl+Enter` (send request).
- **Request Pane**:
  - **URL & Method Selector**: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS.
  - **Params / Query**: Key-value table with active checkboxes and automatic URL encoding.
  - **Headers**: Key-value headers with autocompletion and secret masking.
  - **Auth**: None, Bearer Token, Basic Auth, API Key (Header or Query).
  - **Body**: Form-Data, x-www-form-urlencoded, Raw (JSON/Text/HTML/XML), GraphQL, Binary.
  - **Scripts**: Sandboxed Pre-Request and Post-Request (Tests) JavaScript scripts.
  - **Docs & Mock**: Markdown documentation and sample responses.
  - **Code Generation**: Instant snippet generation in cURL (Bash, CMD, PowerShell), Python (Requests), and JavaScript (Fetch).

---

## 3. Variables & Scope Resolution

Light Postman resolves `{{variable_name}}` templates using an unambiguous 6-tier scope hierarchy:

$$\text{Runtime (Scripts)} > \text{Request} > \text{Folder} > \text{Collection} > \text{Environment} > \text{Global}$$

### Dynamic Variables
Built-in dynamic variables are generated fresh at request send time:
- `{{$guid}}`: Generates a random UUID v4.
- `{{$timestamp}}`: Current Unix epoch in seconds.
- `{{$isoTimestamp}}`: Current ISO-8601 / RFC-3339 timestamp.
- `{{$randomInt}}`: Random integer between 0 and 1,000.

---

## 4. Scripting & Test Assertions

Light Postman features an isolated JavaScript runtime (`boa_engine`) with Postman-compatible APIs:

### Pre-Request Scripts
```javascript
// Modify environment variable dynamically before sending
let salt = Math.floor(Math.random() * 10000);
pm.environment.set("request_salt", salt.toString());
console.log("Injected request salt: " + salt);
```

### Post-Request / Test Scripts
```javascript
// Check HTTP status
pm.test("Status code is 200", function () {
    pm.response.to.have.status(200);
});

// Parse response JSON and set token
let json = pm.response.json();
if (json && json.token) {
    pm.environment.set("auth_token", json.token);
    console.log("Saved new auth token");
}
```

---

## 5. Developer Console & Observability

Access the integrated Developer Console at the bottom of the window:
- **Chronological Logs**: Request dispatch, DNS resolution, connection, headers, redirects, cookies, and responses.
- **Correlation IDs**: Trace full lifecycles across pre-request scripts, network calls, and test assertions.
- **Redaction**: Authorization tokens, passwords, and sensitive query parameters are masked with `[REDACTED]`.
- **Export**: Export complete session logs to JSON for bug reports.

---

## 6. AI Assistance (Local & Cloud)

Configure any OpenAI-compatible or Anthropic Claude API provider in **Settings → AI Settings**:
1. **API Generator**: Describe an endpoint in natural language (e.g., "Create a user registration endpoint with email validation") to generate full request specs, headers, and JSON body templates.
2. **Test Generation**: 1-click generation of comprehensive `pm.test` assertions in the Scripts tab.
3. **Docs Generation**: 1-click generation of formatted Markdown endpoint documentation.
4. **Sample Responses**: Instant synthesis of realistic mock responses.
5. **Source Discovery**: Point to a local backend repo (Express, FastAPI, Spring Boot, Gin, etc.) to discover endpoints and import them as collections with 1 click.
