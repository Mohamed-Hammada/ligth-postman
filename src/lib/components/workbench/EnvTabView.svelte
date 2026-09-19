<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { commitNewEnvVar, forkCurrentEnvironment, shareCurrentEnvironment, toggleVariableEnabled, updateVariableKey, updateVariableValue } = app;
</script>

<div class="env-tab-view">
  <div class="env-tab-header">
    <div class="env-tab-title-group">
      <span class="env-dot-indicator"></span>
      <h1 class="env-tab-title">{app.currentTab.name}</h1>
    </div>
    <div class="env-tab-actions">
      <button type="button" class="btn-env-action" title="Create a copy of this environment with all its variables" onclick={forkCurrentEnvironment}>
        <span class="action-icon">&#9901;</span>
        <span>Fork</span>
      </button>
      <button type="button" class="btn-env-action" title="Copy the non-secret variables to the clipboard as JSON" onclick={shareCurrentEnvironment}>
        <span>Copy JSON</span>
      </button>
    </div>
  </div>

  <div class="env-tab-desc-row">
    <span class="env-tab-desc">Environments are sets of variables that allow you to customize requests for different setups.</span>
  </div>

  <div class="env-tab-search-bar">
    <div class="env-search-wrapper">
      <span class="env-search-icon">🔍</span>
      <input
        type="search"
        placeholder="Filter variables"
        class="env-var-filter-input"
        bind:value={app.envVarSearchQuery}
      />
    </div>
  </div>

  <div class="env-variables-table-container">
    <table class="env-variables-table">
      <thead>
        <tr>
          <th class="col-check"></th>
          <th class="col-key">VARIABLE</th>
          <th class="col-type">TYPE</th>
          <th class="col-val">VALUE</th>
        </tr>
      </thead>
      <tbody>
        {#each app.filteredEnvironmentVariables as v (v.id)}
          <tr>
            <td class="col-check">
              <input type="checkbox" checked={v.enabled} onchange={() => toggleVariableEnabled(v)} />
            </td>
            <td class="col-key font-mono font-bold">
              <input class="table-cell-input" value={v.key} onblur={(e) => updateVariableKey(v, (e.target as HTMLInputElement).value)} />
            </td>
            <td class="col-type">
              <span class="env-type-pill">{v.is_secret ? "secret" : "default"}</span>
            </td>
            <td class="col-val font-mono">
              <input
                class="table-cell-input"
                type={v.is_secret ? "password" : "text"}
                value={v.value}
                onfocus={(e) => { if (v.is_secret) (e.target as HTMLInputElement).select(); }}
                onblur={(e) => updateVariableValue(v, (e.target as HTMLInputElement).value)}
              />
            </td>
          </tr>
        {/each}
        <tr class="add-row">
          <td class="col-check"><input type="checkbox" disabled /></td>
          <td class="col-key">
            <input
              class="table-cell-input placeholder-row"
              placeholder="Add a new variable"
              bind:value={app.newEnvVarDraft.key}
              onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewEnvVar(); } }}
            />
          </td>
          <td class="col-type"><span class="env-type-pill muted">default</span></td>
          <td class="col-val">
            <input
              class="table-cell-input"
              placeholder=""
              bind:value={app.newEnvVarDraft.value}
              onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commitNewEnvVar(); } }}
            />
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</div>
