<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconArrowDown, iconArrowUp, iconMoreVertical, iconSearch } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { PROJECT_SORT_FIELDS, openRequest, pickProjectSortField, quickCreateProject, selectProject, t, toggleSidebarSearchField } = app;
</script>

<div class="sidebar-filter-bar">
  <div class="sidebar-search-box">
    {@render iconSearch()}
    <input
      type="search"
      placeholder={t("sidebar.searchProjects")}
      bind:value={app.projectSearchQuery}
      class="sidebar-search-input"
    />
    {#if app.projectSearchQuery.trim()}
      <span class="request-count-badge" style="font-size: 10px; margin-right: 4px;">{app.filteredProjects.length}/{app.projects.length}</span>
    {/if}
  </div>
  <button type="button" class="icon-btn" title={t("sidebar.newProject")} onclick={quickCreateProject}>+</button>
  <div class="menu-wrap">
    <button
      type="button"
      class="icon-btn"
      title={t("sidebar.sortOptions")}
      onclick={() => (app.projectSortMenuOpen = !app.projectSortMenuOpen)}
    >{@render iconMoreVertical()}</button>
    {#if app.projectSortMenuOpen}
      <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (app.projectSortMenuOpen = false)}></button>
      <div class="dropdown-menu">
        {#each PROJECT_SORT_FIELDS as f (f.field)}
          <button
            type="button"
            class="dropdown-menu-item"
            class:active={app.projectSortField === f.field}
            onclick={() => pickProjectSortField(f.field)}
          >
            <span>{t(f.label)}</span>
            {#if app.projectSortField === f.field}
              <span class="sort-dir-indicator">{#if app.projectSortDir === "asc"}{@render iconArrowUp()}{:else}{@render iconArrowDown()}{/if}</span>
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Search options row: always visible below the search bar -->
<div class="palette-scope-row sidebar-scope-row">
  <button type="button" class="palette-scope-btn" class:active={app.projectSearchScope === "all"} onclick={() => (app.projectSearchScope = "all")}>{t("palette.scopeAll")}</button>
  <button type="button" class="palette-scope-btn" class:active={app.projectSearchScope === "projects"} onclick={() => (app.projectSearchScope = "projects")}>{t("palette.scopeProjects")}</button>
  <button type="button" class="palette-scope-btn" class:active={app.projectSearchScope === "apis"} onclick={() => (app.projectSearchScope = "apis")}>{t("palette.scopeApis")}</button>
</div>
{#if app.projectSearchScope !== "projects"}
  <div class="palette-scope-row palette-field-row sidebar-scope-row">
    <span class="palette-field-label">{t("palette.fieldsLabel")}</span>
    <button type="button" class="palette-scope-btn" class:active={app.sidebarSearchFields.name} aria-pressed={app.sidebarSearchFields.name} onclick={() => toggleSidebarSearchField("name")}>{t("palette.fieldName")}</button>
    <button type="button" class="palette-scope-btn" class:active={app.sidebarSearchFields.url} aria-pressed={app.sidebarSearchFields.url} onclick={() => toggleSidebarSearchField("url")}>{t("palette.fieldUrl")}</button>
    <button type="button" class="palette-scope-btn" class:active={app.sidebarSearchFields.body} aria-pressed={app.sidebarSearchFields.body} onclick={() => toggleSidebarSearchField("body")}>{t("palette.fieldBody")}</button>
  </div>
{/if}

{#if app.projectSearchScope !== "projects" && app.projectSearchQuery.trim().length >= 2}
  <ul class="sidebar-api-results">
    {#each app.sidebarApiResults as r (r.id)}
      <li>
        <button
          type="button"
          class="palette-item sidebar-api-result-item"
          onclick={async () => {
            await selectProject(r.project_id);
            openRequest(r.id);
          }}
        >
          <span class="palette-item-method">{r.method}</span>
          <span class="palette-item-label"><span class="palette-item-breadcrumb">{r.project_name} ›</span> {r.name}</span>
          <div class="response-stat-spacer"></div>
          <span class="palette-item-hint">{r.url}</span>
        </button>
      </li>
    {:else}
      <li class="empty">{t("palette.noMatches")}</li>
    {/each}
  </ul>
{/if}

{#if app.sidebarSectionsVisible.collections && app.projectSearchScope !== "apis"}
<div class="sidebar-accordion-section-title" onclick={() => (app.collectionsAccordionOpen = !app.collectionsAccordionOpen)}>
  <span class="accordion-arrow">{#if app.collectionsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
  <span>COLLECTIONS</span>
  <div class="response-stat-spacer"></div>
  <button type="button" class="icon-btn icon-btn-ghost" title={t("sidebar.newProject")} onclick={(e) => { e.stopPropagation(); quickCreateProject(); }}>+</button>
</div>
{/if}
