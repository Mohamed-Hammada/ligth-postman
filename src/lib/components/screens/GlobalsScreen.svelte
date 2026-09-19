<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { commitNewGlobalVar, exportGlobalVariables, handleAddGlobalVariableClick, toggleVariableEnabled, updateVariableKey, updateVariableValue } = app;
</script>

<div class="globals-screen-container">
  <div class="globals-main-view">
    <div class="globals-header">
      <div class="globals-title-row">
        <h1 class="globals-title">Globals</h1>
        <button type="button" class="btn-link-action" onclick={handleAddGlobalVariableClick}>Add variable</button>
      </div>
      <div class="globals-desc-row">
        <span class="globals-desc">Globals are variables that are available across all workspaces.</span>
      </div>
      <div class="globals-toolbar">
        <div class="globals-search-wrapper">
          <span class="search-icon">🔍</span>
          <input
            type="search"
            placeholder="Filter variables"
            class="globals-search-input"
            bind:value={app.globalsSearch}
          />
        </div>
        <button type="button" class="btn-export-globals" onclick={exportGlobalVariables}>
          <span class="export-icon">&#8682;</span>
          <span>Export</span>
        </button>
      </div>
    </div>

    <div class="globals-table-wrapper">
      <table class="globals-table">
        <thead>
          <tr>
            <th class="col-check"><input type="checkbox" checked title="Select all" /></th>
            <th class="col-key">VARIABLE</th>
            <th class="col-init">INITIAL VALUE</th>
            <th class="col-curr">CURRENT VALUE</th>
          </tr>
        </thead>
        <tbody>
          {#each (app.globalsSearch ? app.filteredProjectVariables.filter(v => v.key.toLowerCase().includes(app.globalsSearch.toLowerCase()) || v.value.toLowerCase().includes(app.globalsSearch.toLowerCase())) : app.filteredProjectVariables) as v (v.id)}
            <tr>
              <td class="col-check">
                <input type="checkbox" checked={v.enabled} onchange={() => toggleVariableEnabled(v)} />
              </td>
              <td class="col-key font-mono font-bold">
                <input class="table-cell-input" value={v.key} onblur={(e) => updateVariableKey(v, (e.target as HTMLInputElement).value)} />
              </td>
              <td class="col-init font-mono">
                <input class="table-cell-input text-secondary" value={v.is_secret ? "••••••••" : v.value} onblur={(e) => updateVariableValue(v, (e.target as HTMLInputElement).value)} />
              </td>
              <td class="col-curr font-mono">
                <input class="table-cell-input" value={v.is_secret ? "••••••••" : v.value} onblur={(e) => updateVariableValue(v, (e.target as HTMLInputElement).value)} />
              </td>
            </tr>
          {/each}
          <tr class="add-row">
            <td class="col-check"><input type="checkbox" disabled /></td>
            <td class="col-key">
              <input
                class="table-cell-input placeholder-row"
                placeholder="Add variable"
                bind:value={app.newGlobalVarDraft.key}
                onkeydown={(e) => { if (e.key === "Enter") commitNewGlobalVar(); }}
              />
            </td>
            <td class="col-init">
              <input
                class="table-cell-input"
                placeholder=""
                bind:value={app.newGlobalVarDraft.value}
                onkeydown={(e) => { if (e.key === "Enter") commitNewGlobalVar(); }}
              />
            </td>
            <td class="col-curr"></td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</div>
