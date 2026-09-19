<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconCopy } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { copyTextToClipboard, downloadFile, sampleDocContent } = app;
</script>

<div class="doc-tab-view">
  <div class="doc-tab-header">
    <div class="doc-tab-title-group">
      <span class="doc-icon-badge">📖</span>
      <h1 class="doc-tab-title">{app.currentTab.name}</h1>
      <span class="preview-pill" title="Sample content only — nothing here is saved or connected to a backend yet">Preview · sample data</span>
      <span class="doc-status-badge">Published</span>
    </div>
    <div class="doc-tab-actions">
      <button type="button" class="btn-env-action" onclick={() => downloadFile(sampleDocContent, `${app.currentTab.name}.md`, "text/markdown")}>
        <span>Export Markdown</span>
      </button>
      <button type="button" class="btn-env-action" onclick={() => copyTextToClipboard(sampleDocContent)}>
        {@render iconCopy()}
        <span>Copy Content</span>
      </button>
    </div>
  </div>
  <div class="doc-tab-body">
    <div class="doc-preview-card">
      <div class="doc-markdown-content font-mono">
        <pre class="doc-content-pre">{sampleDocContent}</pre>
      </div>
    </div>
  </div>
</div>
