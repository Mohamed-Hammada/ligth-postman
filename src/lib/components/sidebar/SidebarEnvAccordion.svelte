<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { isEnvFavorite, openEnvironmentTab, quickCreateEnvironment, toggleEnvExpand, toggleEnvFavorite } = app;
</script>

{#if app.sidebarSectionsVisible.environments}
<div class="sidebar-accordion-section-title" onclick={() => (app.environmentsAccordionOpen = !app.environmentsAccordionOpen)}>
  <span class="accordion-arrow">{#if app.environmentsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
  <span>ENVIRONMENTS</span>
  <div class="response-stat-spacer"></div>
  <button type="button" class="icon-btn icon-btn-ghost" title="New Environment" onclick={(e) => { e.stopPropagation(); quickCreateEnvironment(); }}>+</button>
</div>
{#if app.environmentsAccordionOpen}
  <div class="sidebar-env-list">
    <!-- Starred / Favorite Environments Section -->
    {#if app.favoriteEnvs.length > 0}
      <div class="sidebar-fav-header">
        <span>★ FAVORITES ({app.favoriteEnvs.length})</span>
      </div>
      {#each app.favoriteEnvs as env (env.id)}
        <div class="sidebar-env-item-row" class:active={app.selectedEnvironmentId === env.id}>
          <button
            type="button"
            class="env-expand-btn"
            title={app.expandedEnvIds.has(env.id) ? "Collapse variables" : "Expand variables"}
            onclick={(e) => { e.stopPropagation(); toggleEnvExpand(env.id); }}
          >{#if app.expandedEnvIds.has(env.id)}▼{:else}▶{/if}</button>
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
        {#if app.expandedEnvIds.has(env.id)}
          {@const vars = app.envVariablesCache.get(env.id) || []}
          <div class="sidebar-env-vars-container">
            {#each vars as v (v.id)}
              <div class="sidebar-env-var-item" title="{v.key}: {v.is_secret ? '••••••••' : v.value}">
                <span class="var-key-text">{v.key}:</span>
                <span class="var-val-text">{v.is_secret ? "••••••••" : (v.value || '""')}</span>
              </div>
            {:else}
              <div class="sidebar-env-var-item empty">No variables</div>
            {/each}
          </div>
        {/if}
      {/each}
      <div class="sidebar-fav-divider"></div>
    {/if}

    <!-- All Environments -->
    {#each app.allEnvironments as env (env.id)}
      {@const isFav = isEnvFavorite(env.id)}
      <div class="sidebar-env-item-row" class:active={app.selectedEnvironmentId === env.id}>
        <button
          type="button"
          class="env-expand-btn"
          title={app.expandedEnvIds.has(env.id) ? "Collapse variables" : "Expand variables"}
          onclick={(e) => { e.stopPropagation(); toggleEnvExpand(env.id); }}
        >{#if app.expandedEnvIds.has(env.id)}▼{:else}▶{/if}</button>
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
      {#if app.expandedEnvIds.has(env.id)}
        {@const vars = app.envVariablesCache.get(env.id) || []}
        <div class="sidebar-env-vars-container">
          {#each vars as v (v.id)}
            <div class="sidebar-env-var-item" title="{v.key}: {v.is_secret ? '••••••••' : v.value}">
              <span class="var-key-text">{v.key}:</span>
              <span class="var-val-text">{v.is_secret ? "••••••••" : (v.value || '""')}</span>
            </div>
          {:else}
            <div class="sidebar-env-var-item empty">No variables</div>
          {/each}
        </div>
      {/if}
    {/each}
  </div>
{/if}
{/if}
