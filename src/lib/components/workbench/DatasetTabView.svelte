<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { downloadFile } = app;
</script>

<div class="dataset-tab-view">
  <div class="dataset-tab-header">
    <div class="dataset-tab-title-group">
      <span class="dataset-icon-badge">📊</span>
      <h1 class="dataset-tab-title">{app.currentTab.name}</h1>
      <span class="preview-pill" title="Sample content only — nothing here is saved or connected to a backend yet">Preview · sample data</span>
      <span class="dataset-count-badge">4 Rows • 4 Columns</span>
    </div>
    <div class="dataset-tab-actions">
      <button type="button" class="btn-env-action" onclick={() => downloadFile("user_id,email,amount,currency\n101,dev1@example.com,500,AED\n102,dev2@example.com,1250,AED\n103,manager@example.com,4500,USD\n104,tester@example.com,250,EUR\n", app.currentTab.name, "text/csv")}>
        <span>Export CSV</span>
      </button>
    </div>
  </div>
  <div class="dataset-tab-body">
    <div class="dataset-table-wrap">
      <table class="env-variables-table">
        <thead>
          <tr>
            <th class="col-type">#</th>
            <th class="col-key">user_id</th>
            <th class="col-key">email</th>
            <th class="col-key">amount</th>
            <th class="col-type">currency</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>1</td>
            <td class="font-mono">101</td>
            <td class="font-mono">dev1@example.com</td>
            <td class="font-mono">500</td>
            <td><span class="env-type-pill">AED</span></td>
          </tr>
          <tr>
            <td>2</td>
            <td class="font-mono">102</td>
            <td class="font-mono">dev2@example.com</td>
            <td class="font-mono">1250</td>
            <td><span class="env-type-pill">AED</span></td>
          </tr>
          <tr>
            <td>3</td>
            <td class="font-mono">103</td>
            <td class="font-mono">manager@example.com</td>
            <td class="font-mono">4500</td>
            <td><span class="env-type-pill">USD</span></td>
          </tr>
          <tr>
            <td>4</td>
            <td class="font-mono">104</td>
            <td class="font-mono">tester@example.com</td>
            <td class="font-mono">250</td>
            <td><span class="env-type-pill">EUR</span></td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</div>
