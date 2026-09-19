<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconClose } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { t } = app;
</script>

{#if app.showDiffModal}
  <div
    class="modal-backdrop"
    onclick={(e) => { if (e.target === e.currentTarget) app.showDiffModal = false; }}
    onkeydown={(e) => { if (e.key === "Escape") app.showDiffModal = false; }}
    role="dialog"
    aria-modal="true"
    tabindex="0"
  >
    <div class="modal-container">
      <div class="modal-header">
        <h3>{t("diff.title")}</h3>
        <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (app.showDiffModal = false)}>{@render iconClose()}</button>
      </div>
      <div class="modal-body">
        {#if !app.gitDiffContent.trim()}
          <p class="hint">{t("diff.noDiff")}</p>
        {:else}
          <pre class="diff-viewer">{app.gitDiffContent}</pre>
        {/if}
      </div>
      <div class="modal-footer">
        <button type="button" onclick={() => (app.showDiffModal = false)}>{t("common.close")}</button>
      </div>
    </div>
  </div>
{/if}
