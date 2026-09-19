<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
</script>

<div class="flow-tab-view">
  <div class="flow-tab-header">
    <div class="flow-tab-title-group">
      <span class="flow-icon-badge">🔀</span>
      <h1 class="flow-tab-title">{app.currentTab.name}</h1>
      <span class="preview-pill" title="Sample content only — nothing here is saved or connected to a backend yet">Preview · sample data</span>
      <span class="flow-steps-badge">4 Blocks</span>
    </div>
    <div class="flow-tab-actions">
      <button type="button" class="btn-send" onclick={() => { app.exportFeedback = "Running flow sequence..."; setTimeout(() => { app.exportFeedback = "Flow completed successfully (4/4 blocks passed)"; setTimeout(() => { app.exportFeedback = ""; }, 3000); }, 1200); }}>
        <span>▶ Run Flow</span>
      </button>
    </div>
  </div>
  <div class="flow-tab-body">
    <div class="flow-canvas">
      <div class="flow-block">
        <div class="flow-block-header start">1. Start Trigger</div>
        <div class="flow-block-content">Manual execution on click</div>
      </div>
      <div class="flow-connector">➔</div>
      <div class="flow-block">
        <div class="flow-block-header request">2. Send Request</div>
        <div class="flow-block-content font-mono">POST /api/v1/auth/login</div>
      </div>
      <div class="flow-connector">➔</div>
      <div class="flow-block">
        <div class="flow-block-header evaluate">3. Extract Token</div>
        <div class="flow-block-content font-mono">data.sessionToken</div>
      </div>
      <div class="flow-connector">➔</div>
      <div class="flow-block">
        <div class="flow-block-header request">4. Submit Payment</div>
        <div class="flow-block-content font-mono">POST /api/v1/card/transaction</div>
      </div>
    </div>
  </div>
</div>
