<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconFolder } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { quickCreateProject, selectProject, t } = app;
</script>

<section class="screen-page">
  <div class="screen-page-header">
    <span class="screen-kicker">{t("launcher.kicker")}</span>
    <h1 class="screen-title">{t("launcher.title")}</h1>
    <p class="screen-subtitle">{t("launcher.subtitle", { count: app.projects.length })}</p>
  </div>

  {#if app.projects.length === 0}
    <div class="screen-empty">
      <div class="empty-icon">{@render iconFolder()}</div>
      <p>{t("launcher.noProjects")}</p>
      <div style="display:flex; gap: var(--space-3); margin-top: var(--space-4);">
        <button type="button" class="btn-primary" onclick={async () => { await quickCreateProject(); app.activeScreen = "workspace"; }}>+ {t("launcher.newProject")}</button>
        <button type="button" class="btn-secondary" onclick={() => { app.collectionImportTarget = "new"; app.importActiveTab = "collection"; app.collectionImportReport = null; app.collectionImportError = ""; app.showImportDialog = true; }}>{t("launcher.importCollection")}</button>
      </div>
    </div>
  {:else}
    <div class="launcher-grid">
      {#each app.projects as p (p.id)}
        <button
          type="button"
          class="launcher-card"
          class:active={p.id === app.selectedProjectId}
          onclick={() => { selectProject(p.id); app.activeScreen = "workspace"; }}
        >
          <span class="screen-kicker" class:current={p.id === app.selectedProjectId}>{p.id === app.selectedProjectId ? t("launcher.openNow") : t("launcher.updated", { date: new Date(p.updated_at).toLocaleDateString() })}</span>
          <div class="launcher-card-name">{p.name}</div>
          <div class="hr"></div>
          <div class="launcher-card-meta">
            <span>{t("launcher.requestCount", { count: app.projectRequestCounts[p.id] ?? 0 })}</span>
          </div>
        </button>
      {/each}
    </div>
  {/if}

  <div class="screen-page-body" style="flex:none; display:flex; gap: var(--space-3);">
    <button type="button" class="btn-primary" onclick={async () => { await quickCreateProject(); app.activeScreen = "workspace"; }}>+ {t("launcher.newProject")}</button>
    <button type="button" class="btn-secondary" onclick={() => { app.collectionImportTarget = "new"; app.importActiveTab = "collection"; app.collectionImportReport = null; app.collectionImportError = ""; app.showImportDialog = true; }}>{t("launcher.importCollection")}</button>
  </div>
</section>
