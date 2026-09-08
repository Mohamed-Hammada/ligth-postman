<script lang="ts">
  import { onMount } from "svelte";
  import {
    api,
    describeError,
    type Project,
    type RequestFull,
    type RequestSummary,
  } from "$lib/api";

  let projects = $state<Project[]>([]);
  let selectedProjectId = $state<string | null>(null);

  let requests = $state<RequestSummary[]>([]);
  let selectedRequest = $state<RequestFull | null>(null);

  let newProjectName = $state("");
  let newRequestName = $state("");
  let newRequestMethod = $state("GET");
  let newRequestUrl = $state("");

  let errorMessage = $state("");
  let loadingRequests = $state(false);

  onMount(loadProjects);

  async function loadProjects() {
    try {
      projects = await api.listProjects();
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
    loadingRequests = true;
    try {
      requests = await api.listRequests(id);
    } catch (err) {
      errorMessage = describeError(err);
    } finally {
      loadingRequests = false;
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
    } catch (err) {
      errorMessage = describeError(err);
    }
  }

  // Hydrate the full request only when the user actually opens it.
  async function openRequest(id: string) {
    try {
      selectedRequest = await api.getRequest(id);
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
        <li>
          <button
            class="link"
            class:active={project.id === selectedProjectId}
            onclick={() => selectProject(project.id)}
          >
            {project.name}
          </button>
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
              <li>
                <button
                  class="link"
                  class:active={req.id === selectedRequest?.id}
                  onclick={() => openRequest(req.id)}
                >
                  <span class="method">{req.method}</span>
                  <span class="name">{req.name}</span>
                  <span class="url">{req.url}</span>
                </button>
              </li>
            {:else}
              <li class="empty">No requests yet.</li>
            {/each}
          </ul>
        {/if}
      </section>

      {#if selectedRequest}
        <section class="detail">
          <h2>{selectedRequest.name}</h2>
          <p><strong>{selectedRequest.method}</strong> {selectedRequest.url}</p>
          <p class="hint">Headers: {selectedRequest.headers.length}</p>
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
