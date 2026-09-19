<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconCopy } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { copyTextToClipboard, downloadFile, sampleOpenApiJson } = app;
</script>

<div class="spec-tab-view">
  <div class="spec-tab-header">
    <div class="spec-tab-title-group">
      <span class="spec-icon-badge">⚡</span>
      <h1 class="spec-tab-title">{app.currentTab.name}</h1>
      <span class="preview-pill" title="Sample content only — nothing here is saved or connected to a backend yet">Preview · sample data</span>
      <span class="spec-version-badge">OpenAPI 3.1.0</span>
      <span class="spec-valid-badge">✓ Valid</span>
    </div>
    <div class="spec-tab-actions">
      <button type="button" class="btn-env-action" onclick={() => downloadFile(sampleOpenApiJson, app.currentTab.name, "application/json")}>
        <span>Download Spec</span>
      </button>
      <button type="button" class="btn-env-action" onclick={() => copyTextToClipboard(sampleOpenApiJson)}>
        {@render iconCopy()}
        <span>Copy JSON</span>
      </button>
    </div>
  </div>
  <div class="spec-tab-body">
    <div class="spec-endpoints-summary">
      <div class="spec-summary-item"><span class="spec-summary-label">Servers</span><span class="spec-summary-val">2 active</span></div>
      <div class="spec-summary-item"><span class="spec-summary-label">Paths</span><span class="spec-summary-val">3 endpoints</span></div>
      <div class="spec-summary-item"><span class="spec-summary-label">Format</span><span class="spec-summary-val">JSON Schema</span></div>
    </div>
    <div class="spec-code-card font-mono">
      <pre class="spec-code-pre">{sampleOpenApiJson}</pre>
    </div>
  </div>
</div>
