<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconChevronDown, iconClose, iconCopy, iconEye, iconImport, iconStar } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { closeTabAction, duplicateEnvironment, exportPostmanEnvironmentAction, toggleDefaultEnvironment, focusOnMount, isEnvFavorite, isTabDirty, loadVariables, openRequest, openTabContextMenu, pickEnvironment, quickCreateEnvironment, quickCreateRequest, selectTab, t, toggleEnvFavorite } = app;
</script>

<div class="workbench-header-strip">
  <div class="request-tabs-bar" oncontextmenu={(e) => { if (e.target === e.currentTarget) { e.preventDefault(); openTabContextMenu(e, null); } }}>
    {#each app.openTabs as tab (tab.id)}
      <button
        type="button"
        class="request-tab-pill"
        class:active={(app.activeTabId ? tab.id === app.activeTabId : tab.id === app.selectedRequest?.id)}
        class:dirty={isTabDirty(tab.id)}
        onclick={() => selectTab(tab)}
        oncontextmenu={(e) => { e.preventDefault(); openTabContextMenu(e, tab); }}
        onmousedown={(e) => {
          if (e.button === 1) {
            e.preventDefault();
            closeTabAction(tab.id);
          }
        }}
      >
        {#if tab.tabType === "env"}
          <span class="tab-env-icon" style="color: #ff6c37; font-size: 11px; margin-right: 4px;">📄</span>
        {:else if tab.tabType === "doc"}
          <span class="tab-env-icon" style="color: #0cbb52; font-size: 11px; margin-right: 4px;">📖</span>
        {:else if tab.tabType === "spec"}
          <span class="tab-env-icon" style="color: #108ee9; font-size: 11px; margin-right: 4px;">⚡</span>
        {:else if tab.tabType === "mock"}
          <span class="tab-env-icon" style="color: #fa8c16; font-size: 11px; margin-right: 4px;">📦</span>
        {:else if tab.tabType === "dataset"}
          <span class="tab-env-icon" style="color: #722ed1; font-size: 11px; margin-right: 4px;">📊</span>
        {:else if tab.tabType === "flow"}
          <span class="tab-env-icon" style="color: #13c2c2; font-size: 11px; margin-right: 4px;">🔀</span>
        {:else}
          <span class="tab-method method-{(tab.method || 'GET').toLowerCase()}">{tab.method || 'GET'}</span>
        {/if}
        <span class="tab-title">{tab.name}</span>
        {#if isTabDirty(tab.id)}
          <span class="tab-unsaved-dot" title={t("tab.unsavedChanges")}></span>
        {/if}
        <span
          class="tab-close-btn"
          title={t("tab.closeTab")}
          onclick={(e) => {
            e.stopPropagation();
            closeTabAction(tab.id);
          }}
        >
          {@render iconClose()}
        </span>
      </button>
    {/each}
    <button
      type="button"
      class="tab-add-btn"
      title="New Tab"
      onclick={() => quickCreateRequest(app.selectedProjectId ?? app.projects[0]?.id)}
    >+</button>
    {#if app.openTabs.length > 6}
      <details class="tab-overflow-menu">
        <summary class="tab-overflow-trigger" title={t("tab.allOpenTabs")}>
          {@render iconChevronDown()}
        </summary>
        <div class="tab-overflow-list">
          {#each app.openTabs as tab (tab.id)}
            <button
              type="button"
              class="tab-overflow-item"
              class:active={tab.id === app.selectedRequest?.id}
              onclick={(e) => {
                openRequest(tab.id);
                (e.currentTarget as HTMLElement).closest("details")?.removeAttribute("open");
              }}
              oncontextmenu={(e) => { e.preventDefault(); openTabContextMenu(e, tab); }}
            >
              <span class="tab-method method-{tab.method.toLowerCase()}">{tab.method}</span>
              <span class="request-name">{tab.name}</span>
            </button>
          {/each}
        </div>
      </details>
    {/if}
  </div>

  <div class="workbench-env-picker">
    <div class="menu-wrap">
      <button
        type="button"
        class="env-pill-btn"
        onclick={() => (app.envPickerOpen = !app.envPickerOpen)}
      >
        <span class="env-pill-dot" class:empty={!app.selectedEnvironmentId}></span>
        <span class="env-pill-label">{app.selectedEnvironmentId ? (app.allEnvironments.find((e) => e.id === app.selectedEnvironmentId)?.name ?? app.selectedEnvironmentId) : t("topbar.noEnvironment")}</span>
        {@render iconChevronDown()}
      </button>
      {#if app.envPickerOpen}
        <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (app.envPickerOpen = false)}></button>
        <div class="dropdown-menu env-picker-menu">
          <input
            type="search"
            class="request-search-input env-picker-search"
            placeholder={t("env.searchEnvironments")}
            bind:value={app.envPickerQuery}
            use:focusOnMount
          />
          <button type="button" class="dropdown-menu-item" onclick={() => { app.envPickerOpen = false; quickCreateEnvironment(); }}>{t("topbar.newEnvironment")}</button>
          {#if !app.envPickerQuery}
            <button type="button" class="dropdown-menu-item" class:active={!app.selectedEnvironmentId} onclick={() => pickEnvironment(null)}>{t("topbar.noEnvironment")}</button>
          {/if}
          <div class="env-picker-list">
            <!-- Favorites at the top -->
            {#if app.favoriteEnvs.length > 0 && !app.envPickerQuery}
              <div class="env-screen-group-label" style="display: flex; align-items: center; gap: 4px; color: #f5a623;">
                <span>★</span>
                <span>FAVORITES</span>
              </div>
              {#each app.favoriteEnvs as env (env.id)}
                <div class="env-picker-row" class:active={app.selectedEnvironmentId === env.id}>
                  <button type="button" class="dropdown-menu-item env-picker-item-btn" class:active={app.selectedEnvironmentId === env.id} onclick={() => pickEnvironment(env.id)}>
                    <span>{env.name}</span>
                  </button>
                  <button
                    type="button"
                    class="env-star-btn favorited"
                    title="Remove from favorites"
                    onclick={(e) => { e.stopPropagation(); toggleEnvFavorite(env.id); }}
                  >★</button>
                </div>
              {/each}
              <div class="dropdown-menu-divider" style="margin: 4px 8px;"></div>
            {/if}

            <div class="env-screen-group-label">ALL ENVIRONMENTS</div>
            {#each app.envPickerFilteredEnvironments as env (env.id)}
              {@const isFav = isEnvFavorite(env.id)}
              <div class="env-picker-row" class:active={app.selectedEnvironmentId === env.id}>
                <button type="button" class="dropdown-menu-item env-picker-item-btn" class:active={app.selectedEnvironmentId === env.id} onclick={() => pickEnvironment(env.id)}>
                  <span>{env.name}</span>
                </button>
                <button
                  type="button"
                  class="env-star-btn"
                  class:favorited={isFav}
                  title={isFav ? "Remove from favorites" : "Add to favorites"}
                  onclick={(e) => { e.stopPropagation(); toggleEnvFavorite(env.id); }}
                >{isFav ? "★" : "☆"}</button>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
    <button
      type="button"
      class="icon-btn"
      title={t("topbar.manageVariables")}
      onclick={() => { loadVariables(); app.activeScreen = "environments"; }}
    >
      {@render iconEye()}
    </button>
    {#if app.selectedProjectId && app.selectedEnvironmentId}
      {@const isDefault = app.projects.find((p) => p.id === app.selectedProjectId)?.default_environment_id === app.selectedEnvironmentId}
      <button
        type="button"
        class="icon-btn"
        class:active={isDefault}
        title={isDefault ? t("topbar.unsetDefaultEnv") : t("topbar.setDefaultEnv")}
        aria-pressed={isDefault}
        onclick={toggleDefaultEnvironment}
      >
        {@render iconStar(isDefault)}
      </button>
    {/if}
    {#if app.selectedEnvironmentId}
      <button type="button" class="icon-btn" title={t("topbar.exportEnvironment")} onclick={exportPostmanEnvironmentAction}>
        {@render iconImport()}
      </button>
      <button
        type="button"
        class="icon-btn"
        title={t("topbar.duplicateEnvironment")}
        onclick={async () => {
          const src = app.allEnvironments.find((e) => e.id === app.selectedEnvironmentId);
          if (!src) return;
          const copy = await duplicateEnvironment(src);
          if (copy) app.selectedEnvironmentId = copy.id;
        }}
      >
        {@render iconCopy()}
      </button>
    {/if}
  </div>
</div>
