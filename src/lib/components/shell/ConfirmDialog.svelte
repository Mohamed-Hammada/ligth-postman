<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconClose } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { t } = app;
</script>

{#if app.confirmDialog.show}
  <div
    class="modal-backdrop"
    onclick={(e) => { if (e.target === e.currentTarget) app.confirmDialog.show = false; }}
    onkeydown={(e) => { if (e.key === "Escape") app.confirmDialog.show = false; }}
    role="dialog"
    aria-modal="true"
    tabindex="0"
  >
    <div class="modal-container">
      <div class="modal-header">
        <h3>{app.confirmDialog.title}</h3>
        <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (app.confirmDialog.show = false)}>{@render iconClose()}</button>
      </div>
      <div class="modal-body">
        <p>{app.confirmDialog.message}</p>
      </div>
      <div class="modal-footer">
        <button type="button" onclick={() => (app.confirmDialog.show = false)}>{t("request.cancel")}</button>
        <button type="button" class="btn-primary" onclick={() => { app.confirmDialog.show = false; app.confirmDialog.onConfirm(); }}>{t("common.confirm")}</button>
      </div>
    </div>
  </div>
{/if}
