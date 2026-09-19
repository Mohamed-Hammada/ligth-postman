<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconClose } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { t } = app;
</script>

{#if app.showHistoryModal}
  <div
    class="modal-backdrop"
    onclick={(e) => { if (e.target === e.currentTarget) app.showHistoryModal = false; }}
    onkeydown={(e) => { if (e.key === "Escape") app.showHistoryModal = false; }}
    role="dialog"
    aria-modal="true"
    tabindex="0"
  >
    <div class="modal-container">
      <div class="modal-header">
        <h3>{t("diffHistory.title")}</h3>
        <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (app.showHistoryModal = false)}>{@render iconClose()}</button>
      </div>
      <div class="modal-body">
        {#if app.gitHistory.length === 0}
          <p class="hint">{t("diffHistory.noHistory")}</p>
        {:else}
          <div class="history-list">
            {#each app.gitHistory as c}
              <div class="history-item">
                <div class="commit-header">
                  <code class="commit-hash">{c.hash.slice(0, 8)}</code>
                  <span class="commit-author">{c.author}</span>
                  <span class="commit-date">{c.date}</span>
                </div>
                <p class="commit-msg">{c.message}</p>
              </div>
            {/each}
          </div>
        {/if}
      </div>
      <div class="modal-footer">
        <button type="button" onclick={() => (app.showHistoryModal = false)}>{t("common.close")}</button>
      </div>
    </div>
  </div>
{/if}
