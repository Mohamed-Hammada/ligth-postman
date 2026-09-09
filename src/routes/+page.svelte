<script lang="ts">
  import { onMount } from "svelte";
  import {
    api,
    describeError,
    type Auth,
    type Environment,
    type GeneratedApiDefinition,
    type Project,
    type QueryParam,
    type RequestFull,
    type RequestSummary,
    type ResolvedTemplate,
    type ResponseMeta,
    type ResponseSummary,
    type SnippetMode,
  } from "$lib/api";

  let projects = $state<Project[]>([]);
  let selectedProjectId = $state<string | null>(null);

  let requests = $state<RequestSummary[]>([]);
  let selectedRequest = $state<RequestFull | null>(null);

  let environments = $state<Environment[]>([]);
  let selectedEnvironmentId = $state<string | null>(null);
  let newEnvironmentName = $state("");
  let urlPreview = $state<ResolvedTemplate | null>(null);

  let newProjectName = $state("");
  let newRequestName = $state("");
  let newRequestMethod = $state("GET");
  let newRequestUrl = $state("");

  let renamingProjectId = $state<string | null>(null);
  let renameProjectValue = $state("");

  // Mirrors the active request while it's being edited; reset whenever a
  // different request is opened (README §5 "editor state" step).
  let editName = $state("");
  let editMethod = $state("GET");
  let editUrl = $state("");
  let editQueryParams = $state<QueryParam[]>([]);
  let editAuthType = $state<Auth["type"]>("none");
  let editAuthBearerToken = $state("");
  let editAuthBasicUsername = $state("");
  let editAuthBasicPassword = $state("");
  let editAuthApiKeyKey = $state("");
  let editAuthApiKeyValue = $state("");
  let editAuthApiKeyLocation = $state<"header" | "query">("header");

  let snippetMode = $state<SnippetMode>("placeholder");
  let snippet = $state("");
  let snippetError = $state("");

  let errorMessage = $state("");
  let loadingRequests = $state(false);

  let sending = $state(false);
  let activeResponse = $state<ResponseMeta | null>(null);
  let activeResponseBody = $state("");
  let activeResponseTruncated = $state(false);
  let responseHistory = $state<ResponseSummary[]>([]);

  let aiConfigured = $state(false);
  let aiPrompt = $state("");
  let aiGenerating = $state(false);
  let aiPreview = $state<GeneratedApiDefinition | null>(null);

  $effect(() => {
    if (selectedRequest) {
      editName = selectedRequest.name;
      editMethod = selectedRequest.method;
      editUrl = selectedRequest.url;
      editQueryParams = selectedRequest.query_params.map((p) => ({ ...p }));

      const auth = selectedRequest.auth;
      editAuthType = auth.type;
      editAuthBearerToken = auth.type === "bearer" ? auth.token : "";
      editAuthBasicUsername = auth.type === "basic" ? auth.username : "";
      editAuthBasicPassword = auth.type === "basic" ? auth.password : "";
      editAuthApiKeyKey = auth.type === "api_key" ? auth.key : "";
      editAuthApiKeyValue = auth.type === "api_key" ? auth.value : "";
      editAuthApiKeyLocation = auth.type === "api_key" ? auth.location : "header";

      snippet = "";
      snippetError = "";
    }
  });

  function buildAuthFromEditFields(): Auth {
    switch (editAuthType) {
      case "bearer":
        return { type: "bearer", token: editAuthBearerToken };
      case "basic":
        return { type: "basic", username: editAuthBasicUsername, password: editAuthBasicPassword };
      case "api_key":
        return { type: "api_key", key: editAuthApiKeyKey, value: editAuthApiKeyValue, location: editAuthApiKeyLocation };
      default:
        return { type: "none" };
    }
  }

  async function copyAsCurl() {
    if (!selectedRequest) return;
    snippetError = "";
    try {
      snippet = await api.generateCurlSnippet(selectedRequest.id, selectedEnvironmentId, snippetMode);
    } catch (err) {
      snippetError = describeError(err);
    }
  }

  async function copySnippetToClipboard() {
    if (!snippet) return;
    try {
      await navigator.clipboard.writeText(snippet);
    } catch (err) {
      snippetError = describeError(err);
    }
  }

  // Live preview of what {{vars}} in the URL resolve to for the currently selected
  // environment — same resolver code path the HTTP engine will use to build the real
  // request (README §14 "Stored Request -> ... -> Final HTTP Request").
  $effect(() => {
    const projectId = selectedProjectId;
    const requestId = selectedRequest?.id ?? null;
    const template = editUrl;
    if (!projectId || !template) {
      urlPreview = null;
      return;
    }
    api
      .resolvePreview(projectId, selectedEnvironmentId, requestId, template)
      .then((result) => (urlPreview = result))
      .catch(() => (urlPreview = null));
  });

  onMount(() => {
    loadProjects();
    api.isAiConfigured().then((configured) => (aiConfigured = configured));
  });

  async function loadProjects() {
    try {
      projects = await api.listProjects();
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function generateWithAi(event: Event) {
    event.preventDefault();
    if (!aiPrompt.trim() || aiGenerating) return;
    aiGenerating = true;
    aiPreview = null;
    try {
      aiPreview = await api.generateApiWithAi(aiPrompt.trim());
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      aiGenerating = false;
    }
  }

  // Explicit approval step (LP-0805): nothing from the AI preview is persisted until the
  // user clicks this — it goes through the exact same create_request validation as a
  // manually-typed request.
  async function addAiPreviewToProject() {
    if (!aiPreview || !selectedProjectId) return;
    try {
      const request = await api.createRequest({
        project_id: selectedProjectId,
        name: aiPreview.name,
        method: aiPreview.method,
        url: aiPreview.url,
        headers: aiPreview.headers,
        query_params: aiPreview.query_params,
        auth: { type: "none" },
        body: aiPreview.body,
      });
      requests = [
        { id: request.id, project_id: request.project_id, name: request.name, method: request.method, url: request.url, updated_at: request.updated_at },
        ...requests,
      ];
      selectedRequest = request;
      activeResponse = null;
      activeResponseBody = "";
      responseHistory = [];
      aiPreview = null;
      aiPrompt = "";
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function createProject(event: Event) {
    event.preventDefault();
    if (!newProjectName.trim()) return;
    try {
      const project = await api.createProject(newProjectName.trim());
      newProjectName = "";
      projects = [project, ...projects];
      await selectProject(project.id);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // Only ever loads request *metadata* for the selected project — bodies/headers
  // stay on disk until a specific request tab is opened (README §4/§20).
  async function selectProject(id: string) {
    selectedProjectId = id;
    selectedRequest = null;
    selectedEnvironmentId = null;
    loadingRequests = true;
    try {
      requests = await api.listRequests(id);
      environments = await api.listEnvironments(id);
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      loadingRequests = false;
    }
  }

  async function createEnvironment(event: Event) {
    event.preventDefault();
    if (!selectedProjectId || !newEnvironmentName.trim()) return;
    try {
      const env = await api.createEnvironment(selectedProjectId, newEnvironmentName.trim());
      newEnvironmentName = "";
      environments = [...environments, env];
      selectedEnvironmentId = env.id;
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function createRequest(event: Event) {
    event.preventDefault();
    if (!selectedProjectId || !newRequestName.trim() || !newRequestUrl.trim()) return;
    try {
      const request = await api.createRequest({
        project_id: selectedProjectId,
        name: newRequestName.trim(),
        method: newRequestMethod,
        url: newRequestUrl.trim(),
        headers: [],
        query_params: [],
        auth: { type: "none" },
        body: null,
      });
      newRequestName = "";
      newRequestUrl = "";
      requests = [
        {
          id: request.id,
          project_id: request.project_id,
          name: request.name,
          method: request.method,
          url: request.url,
          updated_at: request.updated_at,
        },
        ...requests,
      ];
      selectedRequest = request;
      activeResponse = null;
      activeResponseBody = "";
      responseHistory = [];
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // Hydrate the full request only when the user actually opens it.
  async function openRequest(id: string) {
    try {
      selectedRequest = await api.getRequest(id);
      activeResponse = null;
      activeResponseBody = "";
      responseHistory = await api.listResponseSummaries(id);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function sendCurrentRequest() {
    if (!selectedRequest || sending) return;
    const requestId = selectedRequest.id;
    sending = true;
    try {
      const meta = await api.sendRequest(requestId, selectedEnvironmentId);
      activeResponse = meta;
      const body = await api.getResponseBody(meta.id);
      activeResponseBody = body.text;
      activeResponseTruncated = body.truncated;
      responseHistory = await api.listResponseSummaries(requestId);
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      sending = false;
    }
  }

  async function cancelCurrentSend() {
    if (!selectedRequest) return;
    try {
      await api.cancelSend(selectedRequest.id);
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function openHistoryResponse(id: string) {
    try {
      activeResponse = await api.getResponse(id);
      const body = await api.getResponseBody(id);
      activeResponseBody = body.text;
      activeResponseTruncated = body.truncated;
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function startRenameProject(project: Project) {
    renamingProjectId = project.id;
    renameProjectValue = project.name;
  }

  async function submitRenameProject(event: Event) {
    event.preventDefault();
    if (!renamingProjectId || !renameProjectValue.trim()) return;
    try {
      const updated = await api.updateProject({
        id: renamingProjectId,
        name: renameProjectValue.trim(),
      });
      projects = projects.map((p) => (p.id === updated.id ? updated : p));
      renamingProjectId = null;
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  async function deleteProject(id: string) {
    try {
      await api.deleteProject(id);
      projects = projects.filter((p) => p.id !== id);
      if (selectedProjectId === id) {
        selectedProjectId = null;
        requests = [];
        selectedRequest = null;
      }
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // Only sends fields that actually differ from the hydrated request —
  // the backend preserves anything omitted, but there's no reason to send it either.
  async function saveRequest() {
    if (!selectedRequest) return;
    const original = selectedRequest;
    try {
      const updated = await api.updateRequest({
        id: original.id,
        ...(editName !== original.name ? { name: editName } : {}),
        ...(editMethod !== original.method ? { method: editMethod } : {}),
        ...(editUrl !== original.url ? { url: editUrl } : {}),
        ...(JSON.stringify(editQueryParams) !== JSON.stringify(original.query_params)
          ? { query_params: editQueryParams }
          : {}),
        ...(() => {
          const auth = buildAuthFromEditFields();
          return JSON.stringify(auth) !== JSON.stringify(original.auth) ? { auth } : {};
        })(),
      });
      selectedRequest = updated;
      requests = requests.map((r) =>
        r.id === updated.id
          ? { id: updated.id, project_id: updated.project_id, name: updated.name, method: updated.method, url: updated.url, updated_at: updated.updated_at }
          : r,
      );
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  function addQueryParam() {
    editQueryParams = [...editQueryParams, { key: "", value: "", enabled: true }];
  }

  function removeQueryParam(index: number) {
    editQueryParams = editQueryParams.filter((_, i) => i !== index);
  }

  async function deleteRequest(id: string) {
    try {
      await api.deleteRequest(id);
      requests = requests.filter((r) => r.id !== id);
      if (selectedRequest?.id === id) {
        selectedRequest = null;
        activeResponse = null;
        activeResponseBody = "";
        responseHistory = [];
      }
    } catch (err) {
      errorMessage = describeError(err);
    }
  }
</script>

<div class="app">
  <aside class="sidebar">
    <h2>Projects</h2>
    <form onsubmit={createProject}>
      <input placeholder="New project name" bind:value={newProjectName} />
      <button type="submit">Create</button>
    </form>
    <ul>
      {#each projects as project (project.id)}
        <li class="row-item">
          {#if renamingProjectId === project.id}
            <form class="inline-form" onsubmit={submitRenameProject}>
              <input bind:value={renameProjectValue} />
              <button type="submit" title="Save">✓</button>
              <button type="button" title="Cancel" onclick={() => (renamingProjectId = null)}>✕</button>
            </form>
          {:else}
            <button
              class="link"
              class:active={project.id === selectedProjectId}
              onclick={() => selectProject(project.id)}
            >
              {project.name}
            </button>
            <button class="icon-btn" title="Rename" onclick={() => startRenameProject(project)}>✎</button>
            <button class="icon-btn" title="Delete" onclick={() => deleteProject(project.id)}>🗑</button>
          {/if}
        </li>
      {:else}
        <li class="empty">No projects yet.</li>
      {/each}
    </ul>
  </aside>

  <main class="content">
    {#if errorMessage}
      <p class="error">{errorMessage}</p>
    {/if}

    {#if !selectedProjectId}
      <p class="hint">Select or create a project to see its requests.</p>
    {:else}
      {#if aiConfigured}
        <section class="ai-panel">
          <h2>Ask AI</h2>
          <form class="request-form" onsubmit={generateWithAi}>
            <input
              placeholder="Describe the API request you want, e.g. 'get the weather for a city by name'"
              bind:value={aiPrompt}
              class="url-input"
            />
            <button type="submit" disabled={aiGenerating}>{aiGenerating ? "Generating…" : "Generate"}</button>
          </form>

          {#if aiPreview}
            <div class="ai-preview">
              <p><strong>{aiPreview.method}</strong> {aiPreview.name} — <code>{aiPreview.url}</code></p>
              {#if aiPreview.description}
                <p class="hint">{aiPreview.description}</p>
              {/if}
              <p class="hint">
                Headers: {aiPreview.headers.length} · Query params: {aiPreview.query_params.length}
                {#if aiPreview.body}· has body{/if}
              </p>
              <button type="button" onclick={addAiPreviewToProject}>Add to Project</button>
              <button type="button" onclick={() => (aiPreview = null)}>Discard</button>
            </div>
          {/if}
        </section>
      {/if}

      <section>
        <h2>Environment</h2>
        <form class="request-form" onsubmit={createEnvironment}>
          <select bind:value={selectedEnvironmentId}>
            <option value={null}>No environment</option>
            {#each environments as env (env.id)}
              <option value={env.id}>{env.name}</option>
            {/each}
          </select>
          <input placeholder="New environment name" bind:value={newEnvironmentName} />
          <button type="submit">Add environment</button>
        </form>
      </section>

      <section>
        <h2>Requests</h2>
        <form class="request-form" onsubmit={createRequest}>
          <select bind:value={newRequestMethod}>
            <option>GET</option>
            <option>POST</option>
            <option>PUT</option>
            <option>PATCH</option>
            <option>DELETE</option>
            <option>HEAD</option>
            <option>OPTIONS</option>
          </select>
          <input placeholder="Name" bind:value={newRequestName} />
          <input placeholder="URL" bind:value={newRequestUrl} />
          <button type="submit">Add request</button>
        </form>

        {#if loadingRequests}
          <p class="hint">Loading…</p>
        {:else}
          <ul class="requests">
            {#each requests as req (req.id)}
              <li class="row-item">
                <button
                  class="link"
                  class:active={req.id === selectedRequest?.id}
                  onclick={() => openRequest(req.id)}
                >
                  <span class="method">{req.method}</span>
                  <span class="name">{req.name}</span>
                  <span class="url">{req.url}</span>
                </button>
                <button class="icon-btn" title="Delete" onclick={() => deleteRequest(req.id)}>🗑</button>
              </li>
            {:else}
              <li class="empty">No requests yet.</li>
            {/each}
          </ul>
        {/if}
      </section>

      {#if selectedRequest}
        <section class="detail">
          <h2>Edit request</h2>
          <form class="request-form" onsubmit={(e) => { e.preventDefault(); saveRequest(); }}>
            <select bind:value={editMethod}>
              <option>GET</option>
              <option>POST</option>
              <option>PUT</option>
              <option>PATCH</option>
              <option>DELETE</option>
              <option>HEAD</option>
              <option>OPTIONS</option>
            </select>
            <input placeholder="Name" bind:value={editName} />
            <input placeholder="URL" bind:value={editUrl} class="url-input" />
            <button type="submit">Save</button>
            <button type="button" onclick={() => deleteRequest(selectedRequest!.id)}>Delete</button>
          </form>
          {#if urlPreview}
            <p class="hint">
              Resolves to: <code>{urlPreview.resolved}</code>
              {#if urlPreview.missing.length}
                <span class="warn">(missing: {urlPreview.missing.join(", ")})</span>
              {/if}
            </p>
          {/if}

          <h2>Query Params</h2>
          <div class="params-table">
            {#each editQueryParams as param, i (i)}
              <div class="params-row">
                <input type="checkbox" bind:checked={param.enabled} title="Enabled" />
                <input placeholder="Key" bind:value={param.key} />
                <input placeholder="Value" bind:value={param.value} />
                <button type="button" class="icon-btn" title="Remove" onclick={() => removeQueryParam(i)}>🗑</button>
              </div>
            {:else}
              <p class="hint">No query parameters.</p>
            {/each}
            <button type="button" onclick={addQueryParam}>Add param</button>
            <button type="button" onclick={saveRequest}>Save params</button>
          </div>

          <h2>Authorization</h2>
          <div class="params-table">
            <select bind:value={editAuthType}>
              <option value="none">No Auth</option>
              <option value="bearer">Bearer Token</option>
              <option value="basic">Basic Auth</option>
              <option value="api_key">API Key</option>
            </select>
            {#if editAuthType === "bearer"}
              <div class="params-row">
                <input placeholder="Token" bind:value={editAuthBearerToken} />
              </div>
            {:else if editAuthType === "basic"}
              <div class="params-row">
                <input placeholder="Username" bind:value={editAuthBasicUsername} />
                <input placeholder="Password" bind:value={editAuthBasicPassword} />
              </div>
            {:else if editAuthType === "api_key"}
              <div class="params-row">
                <input placeholder="Key" bind:value={editAuthApiKeyKey} />
                <input placeholder="Value" bind:value={editAuthApiKeyValue} />
                <select bind:value={editAuthApiKeyLocation}>
                  <option value="header">Header</option>
                  <option value="query">Query Param</option>
                </select>
              </div>
            {/if}
            <button type="button" onclick={saveRequest}>Save auth</button>
          </div>

          <h2>Code Snippet</h2>
          <div class="params-table">
            <div class="params-row">
              <select bind:value={snippetMode}>
                <option value="placeholder">Placeholder (safe — never resolves {"{{vars}}"})</option>
                <option value="resolved">Resolved (real values, including secrets)</option>
              </select>
              <button type="button" onclick={copyAsCurl}>Generate cURL</button>
            </div>
            {#if snippetError}
              <p class="error">{snippetError}</p>
            {/if}
            {#if snippet}
              <pre class="body-view">{snippet}</pre>
              <button type="button" onclick={copySnippetToClipboard}>Copy to clipboard</button>
            {/if}
          </div>

          <p class="hint">Headers: {selectedRequest.headers.length} · Updated {new Date(selectedRequest.updated_at).toLocaleString()}</p>

          <div class="send-row">
            {#if sending}
              <button type="button" onclick={cancelCurrentSend}>Cancel</button>
              <span class="hint">Sending…</span>
            {:else}
              <button type="button" onclick={sendCurrentRequest}>Send</button>
            {/if}
          </div>

          {#if activeResponse}
            <div class="response">
              <p>
                <strong class:status-ok={activeResponse.status < 400} class:status-err={activeResponse.status >= 400}>
                  {activeResponse.status} {activeResponse.status_text}
                </strong>
                · {activeResponse.duration_ms} ms · {activeResponse.body_size} bytes
              </p>
              <pre class="body-view">{activeResponseBody}</pre>
              {#if activeResponseTruncated}
                <p class="hint">(truncated — body is larger than the preview cap)</p>
              {/if}
            </div>
          {/if}

          {#if responseHistory.length}
            <h2>History</h2>
            <ul class="requests">
              {#each responseHistory as r (r.id)}
                <li class="row-item">
                  <button class="link" onclick={() => openHistoryResponse(r.id)}>
                    <span class="method">{r.status}</span>
                    <span class="name">{r.duration_ms} ms</span>
                    <span class="url">{new Date(r.created_at).toLocaleString()}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </section>
      {/if}
    {/if}
  </main>
</div>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    color: #0f0f0f;
    background-color: #f6f6f6;
  }

  .app {
    display: flex;
    height: 100vh;
  }

  .sidebar {
    width: 240px;
    border-right: 1px solid #ddd;
    padding: 1rem;
    overflow-y: auto;
  }

  .content {
    flex: 1;
    padding: 1rem 1.5rem;
    overflow-y: auto;
  }

  h2 {
    font-size: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #555;
    margin: 0 0 0.5rem;
  }

  ul {
    list-style: none;
    margin: 0.5rem 0;
    padding: 0;
  }

  .link {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 0.4rem 0.5rem;
    border-radius: 6px;
    cursor: pointer;
    font: inherit;
  }

  .link:hover {
    background: #eaeaea;
  }

  .link.active {
    background: #396cd8;
    color: white;
  }

  .row-item {
    display: flex;
    align-items: center;
    gap: 0.2rem;
  }

  .row-item .link {
    flex: 1;
  }

  .icon-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.2rem 0.3rem;
    border-radius: 4px;
    opacity: 0.6;
  }

  .icon-btn:hover {
    opacity: 1;
    background: #eaeaea;
  }

  .inline-form {
    display: flex;
    gap: 0.2rem;
    width: 100%;
  }

  .inline-form input {
    flex: 1;
    min-width: 0;
  }

  .url-input {
    min-width: 220px;
  }

  .params-table {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    margin-bottom: 0.75rem;
  }

  .params-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .params-row input:not([type="checkbox"]) {
    flex: 1;
  }

  .empty,
  .hint {
    color: #888;
    font-size: 0.9rem;
    padding: 0.4rem 0.5rem;
  }

  .error {
    color: #b00020;
    background: #fde7e9;
    padding: 0.5rem;
    border-radius: 6px;
  }

  form {
    display: flex;
    gap: 0.4rem;
    margin-bottom: 0.5rem;
  }

  .request-form {
    flex-wrap: wrap;
  }

  input,
  select,
  button {
    border-radius: 6px;
    border: 1px solid #ccc;
    padding: 0.4rem 0.6rem;
    font: inherit;
  }

  button[type="submit"] {
    cursor: pointer;
    background: #396cd8;
    color: white;
    border-color: #396cd8;
  }

  .requests .method {
    display: inline-block;
    width: 3.2rem;
    font-weight: 600;
    font-size: 0.8rem;
  }

  .requests .url {
    color: #666;
    margin-left: 0.4rem;
    font-size: 0.85rem;
  }

  .detail {
    margin-top: 1.5rem;
    border-top: 1px solid #ddd;
    padding-top: 1rem;
  }

  .ai-panel {
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #ddd;
  }

  .ai-preview {
    background: rgba(57, 108, 216, 0.08);
    border-radius: 6px;
    padding: 0.6rem 0.8rem;
    margin-top: 0.5rem;
  }

  .warn {
    color: #b06000;
  }

  code {
    background: rgba(0, 0, 0, 0.06);
    padding: 0.1rem 0.3rem;
    border-radius: 4px;
  }

  .send-row {
    margin: 0.75rem 0;
  }

  .response {
    margin-top: 0.5rem;
  }

  .status-ok {
    color: #1a7f37;
  }

  .status-err {
    color: #b00020;
  }

  .body-view {
    background: rgba(0, 0, 0, 0.04);
    padding: 0.6rem;
    border-radius: 6px;
    max-height: 300px;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }

  @media (prefers-color-scheme: dark) {
    .body-view {
      background: rgba(255, 255, 255, 0.06);
    }
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }
    .sidebar {
      border-color: #444;
    }
    .link:hover {
      background: #3a3a3a;
    }
    .detail {
      border-color: #444;
    }
    input,
    select,
    button {
      background: #1f1f1f;
      color: #f6f6f6;
      border-color: #444;
    }
  }
</style>
