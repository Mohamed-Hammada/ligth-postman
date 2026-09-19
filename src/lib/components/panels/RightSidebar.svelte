<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconClose, iconCopy, iconInfo, iconSettings } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { copySnippetToClipboard, copyTextToClipboard, highlightSnippetCode, setRightSidebarVisible, t } = app;
</script>

<aside class="right-sidebar" style="width: {app.rightSidebarWidth}px">
  {#if app.rightPanel === "variables"}
    <div class="right-sidebar-header">
      <span class="right-sidebar-title">All variables</span>
      <div class="response-stat-spacer"></div>
      <button type="button" class="icon-btn" title={t("common.close")} onclick={() => setRightSidebarVisible(false)}>
        {@render iconClose()}
      </button>
    </div>
    <div class="right-drawer-body">
      <div class="env-scope-subhead">
        <span class="env-badge-pill">E</span>
        <span class="env-badge-name">{app.allEnvironments.find((e) => e.id === app.selectedEnvironmentId)?.name ?? "[Alansari] [Remittance] [Masoud] [DEV]"}</span>
      </div>
      <div class="drawer-search-row">
        <span class="drawer-search-icon">🔍</span>
        <input
          type="search"
          placeholder="Filter variables"
          class="drawer-search-input"
          bind:value={app.drawerVarSearch}
        />
      </div>
      <div class="drawer-variables-list">
        {#each app.drawerFilteredVariables as v (v.id)}
          <div class="drawer-variable-item">
            <div class="drawer-var-top">
              <span class="drawer-var-key">{v.key}</span>
              <span class="drawer-var-scope-badge">{v.scopeTag}</span>
            </div>
            <div class="drawer-var-bottom">
              <span class="drawer-var-val">{v.is_secret ? "••••••••" : v.value}</span>
              <button type="button" class="icon-btn icon-btn-ghost copy-btn" title="Copy value" onclick={() => copyTextToClipboard(v.value)}>
                {@render iconCopy()}
              </button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {:else if app.rightPanel === "info"}
    <div class="right-sidebar-header">
      <span class="right-sidebar-title">{@render iconInfo()} {t("bottom.info")}</span>
      <div class="response-stat-spacer"></div>
      <button type="button" class="icon-btn" title={t("common.close")} onclick={() => setRightSidebarVisible(false)}>
        {@render iconClose()}
      </button>
    </div>
    {#if !app.selectedRequest}
      <p class="screen-empty-inline">{t("request.selectPrompt")}</p>
    {:else}
      <div class="bottom-panel">
        <dl class="info-list info-list-grid">
          <dt>{t("bottom.id")}</dt>
          <dd>{app.selectedRequest.id}</dd>
          <dt>{t("bottom.projectId")}</dt>
          <dd>{app.selectedRequest.project_id}</dd>
          <dt>{t("bottom.created")}</dt>
          <dd>{new Date(app.selectedRequest.created_at).toLocaleString()}</dd>
          <dt>{t("bottom.updated")}</dt>
          <dd>{new Date(app.selectedRequest.updated_at).toLocaleString()}</dd>
          <dt>{t("bottom.headersCount")}</dt>
          <dd>{app.selectedRequest.headers.length}</dd>
          <dt>{t("bottom.queryParamsCount")}</dt>
          <dd>{app.selectedRequest.query_params.length}</dd>
        </dl>
      </div>
    {/if}
  {:else if app.rightPanel === "ai"}
    <div class="right-sidebar-header">
      <span class="right-sidebar-title">🪄 Postman AI</span>
      <div class="response-stat-spacer"></div>
      <button type="button" class="icon-btn" title={t("common.close")} onclick={() => setRightSidebarVisible(false)}>
        {@render iconClose()}
      </button>
    </div>
    <div class="right-drawer-body">
      <p class="hint" style="margin-bottom: var(--space-3);">Generate tests, mock payloads, or explain responses with AI.</p>
      <textarea class="form-textarea" placeholder="Ask Postman AI to generate test scripts..." bind:value={app.aiPrompt} rows="4"></textarea>
      <button type="button" class="btn-primary" style="margin-top: var(--space-2); width: 100%;" onclick={() => { app.showAiPanel = true; }}>
        Open AI Assistant
      </button>
    </div>
  {:else if app.rightPanel === "comments"}
    <div class="right-sidebar-header">
      <span class="right-sidebar-title">💬 Comments</span>
      <div class="response-stat-spacer"></div>
      <button type="button" class="icon-btn" title={t("common.close")} onclick={() => setRightSidebarVisible(false)}>
        {@render iconClose()}
      </button>
    </div>
    <div class="right-drawer-body">
      <p class="hint">Comments aren't available yet — this is a local, single-user app with no collaboration backend.</p>
    </div>
  {:else}
    <!-- Code snippet drawer matching p3.png -->
    <div class="right-sidebar-header code-snippet-header">
      <span class="right-sidebar-title">Code snippet</span>
      <div class="response-stat-spacer"></div>
      <div class="menu-wrap">
        <button type="button" class="icon-btn" title="Snippet Settings" onclick={() => (app.showSnippetSettings = !app.showSnippetSettings)}>
          {@render iconSettings()}
        </button>
        {#if app.showSnippetSettings}
          <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (app.showSnippetSettings = false)}></button>
          <div class="dropdown-menu snippet-settings-dropdown" style="right: 0; min-width: 220px; padding: 6px 0;">
            <div style="padding: 6px 12px; font-weight: 600; font-size: 11px; color: #888; border-bottom: 1px solid #333; margin-bottom: 4px;">SNIPPET SETTINGS</div>
            <label class="dropdown-menu-item" style="display: flex; align-items: center; gap: 8px; cursor: pointer; padding: 6px 12px;">
              <input type="checkbox" checked={app.snippetIndentType === "tab"} onchange={(e) => { app.snippetIndentType = e.currentTarget.checked ? "tab" : "space"; }} />
              <span>Use tabs for indent</span>
            </label>
            <label class="dropdown-menu-item" style="display: flex; align-items: center; gap: 8px; cursor: pointer; padding: 6px 12px;">
              <input type="checkbox" checked={app.snippetTrimTrailing} onchange={(e) => { app.snippetTrimTrailing = e.currentTarget.checked; }} />
              <span>Trim trailing spaces</span>
            </label>
          </div>
        {/if}
      </div>
      <button type="button" class="icon-btn" title="Copy snippet to clipboard" onclick={copySnippetToClipboard}>
        {@render iconCopy()}
      </button>
      <button type="button" class="icon-btn" title={t("common.close")} onclick={() => setRightSidebarVisible(false)}>
        {@render iconClose()}
      </button>
    </div>

    <div class="code-snippet-drawer-body">
      <div class="code-snippet-target-row">
        <select bind:value={app.snippetTarget} class="snippet-target-picker">
          <option value="bash">cURL</option>
          <option value="windows_cmd">cURL (Windows CMD)</option>
          <option value="power_shell">cURL (PowerShell)</option>
          <option value="java_script_fetch">JavaScript - Fetch</option>
          <option value="node_fetch">Node.js - Fetch</option>
          <option value="python_requests">Python - Requests</option>
          <option value="preload">Preload element</option>
          <option value="har">HAR (sanitized)</option>
        </select>
        <select bind:value={app.snippetMode} class="snippet-mode-picker">
          <option value="placeholder">Placeholder</option>
          <option value="resolved">Resolved</option>
        </select>
      </div>

      {#if app.snippetError}
        <p class="error">{app.snippetError}</p>
      {:else if app.snippetLoading}
        <p class="hint">{t("bottom.generatingSnippet")}</p>
      {:else if app.snippet}
        <div class="code-snippet-gutter-box">
          <div class="code-snippet-gutter" aria-hidden="true">
            {#each app.snippet.split("\n") as _, idx}
              <span class="gutter-line-no">{idx + 1}</span>
            {/each}
          </div>
          <pre class="code-snippet-pre"><code>{@html highlightSnippetCode(app.snippet)}</code></pre>
        </div>
      {/if}
    </div>
  {/if}
</aside>
