# Light Postman — Import & Export Specifications

## 1. Postman Collection v2.1 Import & Export

Light Postman provides high-fidelity, bidirectional round-tripping with Postman Collection Schema v2.1.0:

### Supported Entities
- **Collections & Folders**: Arbitrary nesting depth with folder-level metadata.
- **Request Definitions**: HTTP methods, URL paths, query parameters, headers.
- **Request Bodies**:
  - Raw (JSON, text, XML, HTML).
  - Form-Data (key-value and file placeholders).
  - URL-Encoded (key-value pairs).
  - GraphQL queries and variables.
- **Authentication**: Bearer token, Basic auth, API key (Header or Query). Unsupported or custom schemes are safely flagged as non-blocking import warnings and fall back to None.
- **Scripts**: Pre-request and test assertion scripts are mapped directly into the project schema.
- **Sample Responses**: Recorded response status, headers, body, and content types are preserved as sample responses.

### Export Format
Exported collections conform strictly to the official Postman v2.1 JSON schema:
```json
{
  "info": {
    "_postman_id": "uuid",
    "name": "Exported Collection",
    "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
  },
  "item": [...]
}
```

---

## 2. Postman Environment Import & Export

### Schema
Light Postman imports and exports Postman environment files containing variable dictionaries:
```json
{
  "id": "env-uuid",
  "name": "Staging Environment",
  "values": [
    {
      "key": "baseUrl",
      "value": "https://staging.api.io",
      "type": "default",
      "enabled": true
    },
    {
      "key": "apiKey",
      "value": "secret_token_val",
      "type": "secret",
      "enabled": true
    }
  ]
}
```
- Variables with `type: "secret"` are imported as masked secrets.
- During export, secrets are preserved or optionally redacted based on user preference.

---

## 3. cURL Command Import

The cURL parser supports multi-platform command structures copied from browser DevTools, terminal sessions, or documentation:
- **Bash / Linux**: Single-quoted arguments, backslash line escapes (`\`).
- **PowerShell**: Double-quoted arguments, backtick line escapes (`` ` ``), `curl.exe` prefix.
- **Windows Command Prompt (CMD)**: Caret line escapes (`^`), double-quoted JSON strings with escaped quotes (`\"`).
- **Extracted Fields**:
  - Target URL and HTTP Method (`-X`, `--request`).
  - Headers (`-H`, `--header`).
  - Query parameters (parsed into distinct editable rows).
  - Authentication (`-u`, `--user`, `--digest`, Bearer headers).
  - Data payload (`-d`, `--data`, `--data-raw`, `--data-binary`).

---

## 4. Source Discovery (OpenAPI & Frameworks)

Point Light Postman to any source code repository:
- **OpenAPI 3.0 / Swagger 2.0**: Automatically locates `openapi.json`, `openapi.yaml`, `swagger.json`, extracts all paths, HTTP operations, and parameter schemas.
- **Code AST Analysis**: Discovers endpoints without requiring running servers across:
  - **Node.js**: Express, Fastify, NestJS, Koa.
  - **Python**: FastAPI, Flask, Django.
  - **Go**: Gin, Echo, Chi.
  - **Java / Kotlin**: Spring Boot (`@GetMapping`, `@PostMapping`, etc.).
  - **PHP**: Laravel (`Route::get`, `Route::post`).
  - **Rust**: Actix-web, Axum.

---

## 5. Native Git Repository Synchronization (`.postman-client/`)

For team collaboration without cloud subscriptions:
- Projects are serialized as clean, human-readable, Git-mergeable JSON files inside the root `.postman-client/` directory.
- Diffs are line-oriented to minimize merge conflicts.
- Built-in Git client provides 1-click `Pull`, `Commit`, and `Push` operations using local Git credentials or personal access tokens.
