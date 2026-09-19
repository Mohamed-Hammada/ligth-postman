<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconSearch } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { isEnvFavorite, openEnvironmentTab, quickCreateEnvironment, t, toggleEnvExpand, toggleEnvFavorite } = app;
</script>

<div class="sidebar-filter-bar">
  <div class="sidebar-search-box">
    {@render iconSearch()}
    <input
      type="search"
      placeholder={t("env.searchEnvironments")}
      bind:value={app.envSidebarSearchQuery}
      class="sidebar-search-input"
    />
    {#if app.envSidebarSearchQuery.trim()}
      <span class="request-count-badge" style="font-size: 10px; margin-right: 4px;">{app.filteredSidebarEnvironments.length}/{app.allEnvironments.length}</span>
    {/if}
  </div>
  <button type="button" class="icon-btn" title={t("env.newEnvironment")} onclick={quickCreateEnvironment}>+</button>
</div>

<!-- Environments search options row: All, Name, Key, Values -->
<div class="palette-scope-row sidebar-scope-row" style="gap: 4px; padding: 4px 10px 8px;">
  <span style="font-size: 10px; color: #888; text-transform: uppercase; font-weight: 600; margin-right: 2px;">Search in:</span>
  <button
    type="button"
    class="palette-scope-btn"
    class:active={app.envSidebarSearchScope === "all"}
    onclick={() => (app.envSidebarSearchScope = "all")}
  >All</button>
  <button
    type="button"
    class="palette-scope-btn"
    class:active={app.envSidebarSearchScope === "name"}
    onclick={() => (app.envSidebarSearchScope = "name")}
  >Name</button>
  <button
    type="button"
    class="palette-scope-btn"
    class:active={app.envSidebarSearchScope === "key"}
    onclick={() => (app.envSidebarSearchScope = "key")}
  >Key</button>
  <button
    type="button"
    class="palette-scope-btn"
    class:active={app.envSidebarSearchScope === "values"}
    onclick={() => (app.envSidebarSearchScope = "values")}
  >Values</button>
</div>

<div class="sidebar-env-list" style="max-height: calc(100vh - 170px); overflow-y: auto; padding: 4px 0;">
  <div class="sidebar-accordion-section-title" style="cursor: default;">
    <span>ENVIRONMENTS ({app.filteredSidebarEnvironments.length})</span>
    <div class="response-stat-spacer"></div>
    <button type="button" class="icon-btn icon-btn-ghost" title={t("env.newEnvironment")} onclick={quickCreateEnvironment}>+</button>
  </div>

  {#if app.filteredSidebarEnvironments.length === 0}
    <div class="sidebar-empty-state" style="padding: 24px 12px; text-align: center;">
      <span style="font-size: 24px; opacity: 0.5;">🌐</span>
      <p class="empty" style="margin: 6px 0 0; font-size: 12px;">
        {app.allEnvironments.length === 0 ? "No environments yet" : "No environments match"}
      </p>
      {#if app.envSidebarSearchQuery.trim()}
        <p style="font-size: 11px; color: #888; margin: 4px 0 0;">
          No environment matches "{app.envSidebarSearchQuery}" in {app.envSidebarSearchScope.toUpperCase()}
        </p>
        <button
          type="button"
          class="btn-xs-primary"
          style="margin-top: 8px;"
          onclick={() => (app.envSidebarSearchQuery = "")}
        >Clear Search</button>
      {:else}
        <button
          type="button"
          class="btn-xs-primary"
          style="margin-top: 8px;"
          onclick={quickCreateEnvironment}
        >+ Create Environment</button>
      {/if}
    </div>
  {:else}
    <!-- Starred / Favorite Environments Section -->
    {#if app.filteredSidebarFavoriteEnvs.length > 0}
      <div class="sidebar-fav-header">
        <span>★ FAVORITES ({app.filteredSidebarFavoriteEnvs.length})</span>
      </div>
      {#each app.filteredSidebarFavoriteEnvs as env (env.id)}
        {@const q = app.envSidebarSearchQuery.trim().toLowerCase()}
        {@const vars = app.envVariablesCache.get(env.id) || []}
        {@const hasVarMatch = q !== "" && vars.some(v => v.key.toLowerCase().includes(q) || (v.value || "").toLowerCase().includes(q))}
        {@const isAutoExpanded = app.expandedEnvIds.has(env.id) || hasVarMatch}
        <div class="sidebar-env-item-row" class:active={app.selectedEnvironmentId === env.id}>
          <button
            type="button"
            class="env-expand-btn"
            title={isAutoExpanded ? "Collapse variables" : "Expand variables"}
            onclick={(e) => { e.stopPropagation(); toggleEnvExpand(env.id); }}
          >{#if isAutoExpanded}▼{:else}▶{/if}</button>
          <button
            type="button"
            class="sidebar-env-link"
            onclick={() => {
              toggleEnvExpand(env.id);
              openEnvironmentTab(env);
            }}
          >
            <span class="env-cube-icon" style="color: #ff6c37; font-size: 13px;">&#9638;</span>
            <span class="env-name-text">{env.name}</span>
            {#if app.selectedEnvironmentId === env.id}
              <span class="env-check-icon" title="Active">✓</span>
            {/if}
          </button>
          <button
            type="button"
            class="env-star-btn favorited"
            title="Remove from favorites"
            onclick={(e) => { e.stopPropagation(); toggleEnvFavorite(env.id); }}
          >★</button>
        </div>
        {#if isAutoExpanded}
          <div class="sidebar-env-vars-container">
            {#each vars as v (v.id)}
              {@const isKeyMatch = q !== "" && (app.envSidebarSearchScope === "all" || app.envSidebarSearchScope === "key") && v.key.toLowerCase().includes(q)}
              {@const isValMatch = q !== "" && (app.envSidebarSearchScope === "all" || app.envSidebarSearchScope === "values") && (v.value || "").toLowerCase().includes(q)}
              <div class="sidebar-env-var-item" class:matched-var={isKeyMatch || isValMatch} title="{v.key}: {v.is_secret ? '••••••••' : v.value}">
                <span class="var-key-text" style={isKeyMatch ? "color: #ff9800; font-weight: 700;" : ""}>{v.key}:</span>
                <span class="var-val-text" style={isValMatch ? "color: #ffeb3b; font-weight: 700;" : ""}>{v.is_secret ? "••••••••" : (v.value || '""')}</span>
              </div>
            {:else}
              <div class="sidebar-env-var-item empty">No variables</div>
            {/each}
          </div>
        {/if}
      {/each}
      <div class="sidebar-fav-divider"></div>
    {/if}

    <!-- All Environments (matching search) -->
    {#each app.filteredSidebarEnvironments as env (env.id)}
      {@const isFav = isEnvFavorite(env.id)}
      {@const q = app.envSidebarSearchQuery.trim().toLowerCase()}
      {@const vars = app.envVariablesCache.get(env.id) || []}
      {@const hasVarMatch = q !== "" && vars.some(v => v.key.toLowerCase().includes(q) || (v.value || "").toLowerCase().includes(q))}
      {@const isAutoExpanded = app.expandedEnvIds.has(env.id) || hasVarMatch}
      <div class="sidebar-env-item-row" class:active={app.selectedEnvironmentId === env.id}>
        <button
          type="button"
          class="env-expand-btn"
          title={isAutoExpanded ? "Collapse variables" : "Expand variables"}
          onclick={(e) => { e.stopPropagation(); toggleEnvExpand(env.id); }}
        >{#if isAutoExpanded}▼{:else}▶{/if}</button>
        <button
          type="button"
          class="sidebar-env-link"
          onclick={() => {
            toggleEnvExpand(env.id);
            openEnvironmentTab(env);
          }}
        >
          <span class="env-cube-icon" style="color: #ff6c37; font-size: 13px;">&#9638;</span>
          <span class="env-name-text">{env.name}</span>
          {#if app.selectedEnvironmentId === env.id}
            <span class="env-check-icon" title="Active">✓</span>
          {/if}
        </button>
        <button
          type="button"
          class="env-star-btn"
          class:favorited={isFav}
          title={isFav ? "Remove from favorites" : "Add to favorites"}
          onclick={(e) => { e.stopPropagation(); toggleEnvFavorite(env.id); }}
        >{isFav ? "★" : "☆"}</button>
      </div>
      {#if isAutoExpanded}
        <div class="sidebar-env-vars-container">
          {#each vars as v (v.id)}
            {@const isKeyMatch = q !== "" && (app.envSidebarSearchScope === "all" || app.envSidebarSearchScope === "key") && v.key.toLowerCase().includes(q)}
            {@const isValMatch = q !== "" && (app.envSidebarSearchScope === "all" || app.envSidebarSearchScope === "values") && (v.value || "").toLowerCase().includes(q)}
            <div class="sidebar-env-var-item" class:matched-var={isKeyMatch || isValMatch} title="{v.key}: {v.is_secret ? '••••••••' : v.value}">
              <span class="var-key-text" style={isKeyMatch ? "color: #ff9800; font-weight: 700;" : ""}>{v.key}:</span>
              <span class="var-val-text" style={isValMatch ? "color: #ffeb3b; font-weight: 700;" : ""}>{v.is_secret ? "••••••••" : (v.value || '""')}</span>
            </div>
          {:else}
            <div class="sidebar-env-var-item empty">No variables</div>
          {/each}
        </div>
      {/if}
    {/each}
  {/if}
</div>
