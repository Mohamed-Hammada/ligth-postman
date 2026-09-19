<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconCopy } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { copyTextToClipboard } = app;
</script>

<div class="mock-tab-view">
  <div class="mock-tab-header">
    <div class="mock-tab-title-group">
      <span class="mock-icon-badge">📦</span>
      <h1 class="mock-tab-title">{app.currentTab.name}</h1>
      <span class="preview-pill" title="Sample content only — nothing here is saved or connected to a backend yet">Preview · sample data</span>
      <span class="mock-status-pill online">● Running</span>
    </div>
    <div class="mock-tab-actions">
      <button type="button" class="btn-env-action" onclick={() => copyTextToClipboard("http://127.0.0.1:8080/mock/v1")}>
        {@render iconCopy()}
        <span>Copy Mock URL</span>
      </button>
    </div>
  </div>
  <div class="mock-tab-body">
    <div class="mock-url-banner">
      <span class="mock-url-label">Mock Server Endpoint:</span>
      <code class="mock-url-code">http://127.0.0.1:8080/mock/v1</code>
      <span class="mock-latency-badge">⚡ 150ms simulated latency</span>
    </div>
    <div class="mock-endpoints-table-wrap">
      <table class="env-variables-table">
        <thead>
          <tr>
            <th class="col-type">METHOD</th>
            <th class="col-key">MOCK PATH</th>
            <th class="col-type">RESPONSE CODE</th>
            <th class="col-val">SIMULATED PAYLOAD</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><span class="tab-method method-post">POST</span></td>
            <td class="font-mono">/api/v1/auth/login</td>
            <td><span class="spec-valid-badge">200 OK</span></td>
            <td class="font-mono text-secondary">&#123; "token": "mock-jwt-token-xyz", "expiresIn": 3600 &#125;</td>
          </tr>
          <tr>
            <td><span class="tab-method method-post">POST</span></td>
            <td class="font-mono">/api/v1/auth/validate-otp</td>
            <td><span class="spec-valid-badge">200 OK</span></td>
            <td class="font-mono text-secondary">&#123; "status": "VERIFIED", "code": "00" &#125;</td>
          </tr>
          <tr>
            <td><span class="tab-method method-post">POST</span></td>
            <td class="font-mono">/api/v1/card/transaction</td>
            <td><span class="spec-valid-badge">200 OK</span></td>
            <td class="font-mono text-secondary">&#123; "transactionId": "TX-9988231", "status": "SUCCESS" &#125;</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</div>
