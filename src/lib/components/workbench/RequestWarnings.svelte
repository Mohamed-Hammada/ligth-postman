<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconWarning } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { scheduleHideMissingVarPopover, showMissingVarPopover, t } = app;
</script>

{#if app.urlPreview}
  <div class="url-preview-bar">
    <span class="preview-label">{t("request.resolvesTo")}</span> <code>{app.urlPreview.resolved}</code>
    {#if app.urlPreview.missing.length}
      <span class="warn-inline">{t("request.missing", { list: app.urlPreview.missing.join(", ") })}</span>
    {/if}
  </div>
{/if}

{#if app.requestDiagnostics?.all_missing?.length}
  <div class="warn-banner missing-vars-banner">
    {@render iconWarning()} {t("request.unresolvedVariables")}
    {#each app.requestDiagnostics.all_missing as varName (varName)}
      <span
        class="missing-var-chip"
        role="presentation"
        onmouseenter={(e) => showMissingVarPopover(varName, e.currentTarget as HTMLElement)}
        onmouseleave={() => scheduleHideMissingVarPopover()}
      >
        <strong>{varName}</strong>
      </span>
    {/each}
    <span class="hint">{t("request.unresolvedHint", { scope: app.selectedEnvironmentId ? t("request.scopeEnvironment") : t("request.scopeGlobal") })}</span>
  </div>
{/if}
