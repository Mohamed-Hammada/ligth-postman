<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import NoProjectPicker from "$lib/components/screens/NoProjectPicker.svelte";
  import ProjectSwitcher from "$lib/components/screens/ProjectSwitcher.svelte";
  let { app }: { app: App } = $props();
  const { openHistoryResponse, openRequest, refreshProjectHistory, t } = app;
</script>

{#if !app.selectedProjectId}
  <NoProjectPicker {app} screenName={t("rail.history")} />
{:else}
  <section class="screen-page">
    <div class="history-toolbar">
      <span class="history-toolbar-project-label">{t("env.project")}</span>
      <ProjectSwitcher {app} />
      <div class="hr-v"></div>
      <input class="history-search" placeholder={t("history.filterPlaceholder")} bind:value={app.historySearchQuery} />
      <div class="hr-v"></div>
      <span class="screen-empty-inline">{t("history.resultsCount", { shown: app.filteredProjectHistory.length, total: app.projectHistory.length })}</span>
      <div class="response-stat-spacer"></div>
      <button type="button" class="history-filter-chip" class:active={app.historyShowFailuresOnly} onclick={() => (app.historyShowFailuresOnly = !app.historyShowFailuresOnly)}>
        {t("history.failuresOnly")}
      </button>
      <button type="button" class="btn-ghost btn-xs" onclick={refreshProjectHistory}>{t("history.refresh")}</button>
    </div>

    <div class="history-header-row">
      <span>{t("history.colMethod")}</span>
      <span>{t("history.colRequest")}</span>
      <span>{t("history.colStatus")}</span>
      <span>{t("history.colTime")}</span>
      <span>{t("history.colWhen")}</span>
    </div>

    <div class="history-rows">
      {#if app.historyLoading}
        <p class="screen-empty-inline" style="padding: var(--space-4);">{t("history.loading")}</p>
      {:else if app.filteredProjectHistory.length === 0}
        <p class="screen-empty-inline" style="padding: var(--space-4);">
          {app.projectHistory.length === 0 ? t("history.noRequestsSent") : t("history.noResultsMatch")}
        </p>
      {:else}
        {#each app.filteredProjectHistory as h (h.id)}
          <button
            type="button"
            class="history-row"
            onclick={async () => { await openRequest(h.request_id); await openHistoryResponse(h.id); app.activeScreen = "workspace"; app.responseExpanded = true; }}
          >
            <span class="history-method">{h.method}</span>
            <span class="history-path">{h.request_name} · {h.url}</span>
            <span class:status-ok={h.status < 400} class:status-err={h.status >= 400}>{h.status}</span>
            <span>{h.duration_ms} ms</span>
            <span>{new Date(h.created_at).toLocaleString()}</span>
          </button>
        {/each}
      {/if}
    </div>
  </section>
{/if}
