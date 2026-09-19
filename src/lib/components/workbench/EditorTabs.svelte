<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconWarning } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { t, withoutEmptyKeyRows } = app;
</script>

<div class="editor-tabs">
  <button type="button" class="editor-tab" class:active={app.activeEditorTab === "docs"} onclick={() => (app.activeEditorTab = "docs")}>
    {t("tab.docs")} {#if app.editDescription}<span class="tab-dot">•</span>{/if}
  </button>
  <button type="button" class="editor-tab" class:active={app.activeEditorTab === "params"} onclick={() => (app.activeEditorTab = "params")}>
    {t("tab.params")}
    {#if app.requestDiagnostics?.query_params_missing?.length}
      <span class="tab-badge-warn" title={t("tab.missingInParams", { list: app.requestDiagnostics.query_params_missing.join(', ') })}>{@render iconWarning()} {app.requestDiagnostics.query_params_missing.length}</span>
    {:else if withoutEmptyKeyRows(app.editQueryParams).length}
      <span class="tab-badge">{withoutEmptyKeyRows(app.editQueryParams).length}</span>
    {/if}
  </button>
  <button type="button" class="editor-tab" class:active={app.activeEditorTab === "auth"} onclick={() => (app.activeEditorTab = "auth")}>
    {t("tab.auth")}
    {#if app.requestDiagnostics?.auth_missing?.length}
      <span class="tab-badge-warn" title={t("tab.missingInAuth", { list: app.requestDiagnostics.auth_missing.join(', ') })}>{@render iconWarning()} {app.requestDiagnostics.auth_missing.length}</span>
    {:else if app.editAuthType !== "none"}
      <span class="tab-dot">•</span>
    {/if}
  </button>
  <button type="button" class="editor-tab" class:active={app.activeEditorTab === "headers"} onclick={() => (app.activeEditorTab = "headers")}>
    {t("tab.headers")}
    {#if withoutEmptyKeyRows(app.editHeaders).length}
      <span class="tab-badge">{withoutEmptyKeyRows(app.editHeaders).length}</span>
    {/if}
    {#if app.editHeaders.some(h => h.enabled && h.key.trim())}
      <span class="tab-dot">•</span>
    {/if}
  </button>
  <button type="button" class="editor-tab" class:active={app.activeEditorTab === "body"} onclick={() => (app.activeEditorTab = "body")}>
    {t("tab.body")}
    {#if app.requestDiagnostics?.body_missing?.length}
      <span class="tab-badge-warn" title={t("tab.missingInBody", { list: app.requestDiagnostics.body_missing.join(', ') })}>{@render iconWarning()} {app.requestDiagnostics.body_missing.length}</span>
    {:else if app.editBody || app.editFormDataItems.length || app.editUrlEncodedItems.length}
      <span class="tab-dot">•</span>
    {/if}
  </button>
  <button type="button" class="editor-tab" class:active={app.activeEditorTab === "scripts"} onclick={() => (app.activeEditorTab = "scripts")}>
    {t("tab.scripts")} {#if app.editPreScript || app.editPostScript}<span class="tab-dot">•</span>{/if}
  </button>
  <button type="button" class="editor-tab" class:active={app.activeEditorTab === "settings"} onclick={() => (app.activeEditorTab = "settings")}>
    {t("tab.settings")}
  </button>
  <button type="button" class="editor-tab" class:active={app.activeEditorTab === "mock"} onclick={() => (app.activeEditorTab = "mock")}>
    {t("tab.mock")} {#if app.sampleResponses.length}<span class="tab-badge">{app.sampleResponses.length}</span>{/if}
  </button>
  <div class="response-stat-spacer"></div>
  <button type="button" class="editor-tab editor-tab-cookies" onclick={() => { app.responseSubTab = "cookies"; }}>
    Cookies
  </button>
</div>
