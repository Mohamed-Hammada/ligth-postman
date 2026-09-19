<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconSearch, iconTrash } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { clearHistory, formatRelativeTime, openHistoryResponse, openRequest, t } = app;
</script>

<div class="sidebar-filter-bar">
  <div class="sidebar-search-box">
    {@render iconSearch()}
    <input
      type="search"
      placeholder="Search history..."
      bind:value={app.historySearchQuery}
      class="sidebar-search-input"
    />
    {#if app.historySearchQuery.trim()}
      <span class="request-count-badge" style="font-size: 10px; margin-right: 4px;">{app.filteredProjectHistory.length}/{app.projectHistory.length}</span>
    {/if}
  </div>
  {#if app.projectHistory.length > 0}
    <button type="button" class="icon-btn" title="Clear History" onclick={clearHistory}>
      {@render iconTrash()}
    </button>
  {/if}
</div>

<button type="button" class="sidebar-history-viewall" onclick={() => (app.activeScreen = "history")}>{t("history.viewAll")}</button>

<div class="sidebar-history-list" style="max-height: calc(100vh - 160px); overflow-y: auto; padding: 4px 0;">
  {#if app.historyLoading}
    <p class="empty" style="padding: 12px;">{t("history.loading")}</p>
  {:else if app.filteredProjectHistory.length === 0}
    <div class="sidebar-empty-state" style="padding: 24px 12px; text-align: center;">
      <span style="font-size: 24px; opacity: 0.5;">⏱</span>
      <p class="empty" style="margin: 6px 0 0; font-size: 12px;">
        {app.projectHistory.length === 0 ? "No requests sent yet" : "No history matches"}
      </p>
      <p style="font-size: 11px; color: #888; margin: 4px 0 0;">
        Send a request to see it in your history
      </p>
    </div>
  {:else}
    {#each app.filteredProjectHistory as h (h.id)}
      <button
        type="button"
        class="sidebar-history-item"
        onclick={async () => {
          await openRequest(h.request_id);
          await openHistoryResponse(h.id);
          app.activeScreen = "workspace";
          app.responseExpanded = true;
        }}
      >
        <div class="history-item-top">
          <span class="palette-item-method method-{h.method.toLowerCase()}">{h.method}</span>
          <span class="status-chip" class:status-ok={h.status < 400} class:status-err={h.status >= 400}>{h.status}</span>
          <div class="response-stat-spacer"></div>
          <span class="history-duration">{h.duration_ms} ms</span>
        </div>
        <div class="history-item-url" title={h.url}>
          {h.url}
        </div>
        <div class="history-item-bottom">
          <span class="history-req-name">{h.request_name}</span>
          <div class="response-stat-spacer"></div>
          <span class="history-time">{formatRelativeTime(h.created_at)}</span>
        </div>
      </button>
    {/each}
  {/if}
</div>
