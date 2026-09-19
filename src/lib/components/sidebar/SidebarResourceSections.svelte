<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { openDatasetTab, openDocumentTab, openFlowTab, openMockTab, openSpecTab, sampleDatasets, sampleDocuments, sampleFlows, sampleMockServers, sampleSpecs } = app;
</script>

{#if app.sidebarSectionsVisible.datasets}
<div class="sidebar-accordion-section-title" onclick={() => (app.datasetsAccordionOpen = !app.datasetsAccordionOpen)}>
  <span class="accordion-arrow">{#if app.datasetsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
  <span>DATASETS</span><span class="preview-pill preview-pill-sm" title="Sample content only">preview</span>
  <div class="response-stat-spacer"></div>
  <button type="button" class="icon-btn icon-btn-ghost" title="New Dataset" onclick={(e) => { e.stopPropagation(); openDatasetTab({ id: 'data-' + Date.now(), name: 'test-dataset.csv' }); }}>+</button>
</div>
{#if app.datasetsAccordionOpen}
  <div class="sidebar-sub-list">
    {#each sampleDatasets as d (d.id)}
      <button type="button" class="sidebar-sub-link" onclick={() => openDatasetTab(d)}>
        <span class="sidebar-sub-icon" style="color: #722ed1;">📊</span>
        <span class="sidebar-sub-name">{d.name}</span>
      </button>
    {/each}
  </div>
{/if}
{/if}

{#if app.sidebarSectionsVisible.documents}
<div class="sidebar-accordion-section-title" onclick={() => (app.documentsAccordionOpen = !app.documentsAccordionOpen)}>
  <span class="accordion-arrow">{#if app.documentsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
  <span>DOCUMENTS</span><span class="preview-pill preview-pill-sm" title="Sample content only">preview</span>
  <div class="response-stat-spacer"></div>
  <button type="button" class="icon-btn icon-btn-ghost" title="New Document" onclick={(e) => { e.stopPropagation(); openDocumentTab({ id: 'doc-' + Date.now(), name: 'New Documentation' }); }}>+</button>
</div>
{#if app.documentsAccordionOpen}
  <div class="sidebar-sub-list">
    {#each sampleDocuments as doc (doc.id)}
      <button type="button" class="sidebar-sub-link" onclick={() => openDocumentTab(doc)}>
        <span class="sidebar-sub-icon" style="color: #0cbb52;">📖</span>
        <span class="sidebar-sub-name">{doc.name}</span>
      </button>
    {/each}
  </div>
{/if}
{/if}

{#if app.sidebarSectionsVisible.specs}
<div class="sidebar-accordion-section-title" onclick={() => (app.specsAccordionOpen = !app.specsAccordionOpen)}>
  <span class="accordion-arrow">{#if app.specsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
  <span>SPECS</span><span class="preview-pill preview-pill-sm" title="Sample content only">preview</span>
  <div class="response-stat-spacer"></div>
  <button type="button" class="icon-btn icon-btn-ghost" title="New API Spec" onclick={(e) => { e.stopPropagation(); openSpecTab({ id: 'spec-' + Date.now(), name: 'new-spec.yaml' }); }}>+</button>
</div>
{#if app.specsAccordionOpen}
  <div class="sidebar-sub-list">
    {#each sampleSpecs as spec (spec.id)}
      <button type="button" class="sidebar-sub-link" onclick={() => openSpecTab(spec)}>
        <span class="sidebar-sub-icon" style="color: #108ee9;">⚡</span>
        <span class="sidebar-sub-name">{spec.name}</span>
      </button>
    {/each}
  </div>
{/if}
{/if}

{#if app.sidebarSectionsVisible.mocks}
<div class="sidebar-accordion-section-title" onclick={() => (app.mocksAccordionOpen = !app.mocksAccordionOpen)}>
  <span class="accordion-arrow">{#if app.mocksAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
  <span>MOCKS</span><span class="preview-pill preview-pill-sm" title="Sample content only">preview</span>
  <div class="response-stat-spacer"></div>
  <button type="button" class="icon-btn icon-btn-ghost" title="New Mock Server" onclick={(e) => { e.stopPropagation(); openMockTab({ id: 'mock-' + Date.now(), name: 'Mock Server ' + (sampleMockServers.length + 1) }); }}>+</button>
</div>
{#if app.mocksAccordionOpen}
  <div class="sidebar-sub-list">
    {#each sampleMockServers as m (m.id)}
      <button type="button" class="sidebar-sub-link" onclick={() => openMockTab(m)}>
        <span class="sidebar-sub-icon" style="color: #fa8c16;">📦</span>
        <span class="sidebar-sub-name">{m.name}</span>
      </button>
    {/each}
  </div>
{/if}
{/if}

{#if app.sidebarSectionsVisible.flows}
<div class="sidebar-accordion-section-title" onclick={() => (app.flowsAccordionOpen = !app.flowsAccordionOpen)}>
  <span class="accordion-arrow">{#if app.flowsAccordionOpen}&#709;{:else}&rsaquo;{/if}</span>
  <span>FLOWS</span><span class="preview-pill preview-pill-sm" title="Sample content only">preview</span>
  <div class="response-stat-spacer"></div>
  <button type="button" class="icon-btn icon-btn-ghost" title="New Flow" onclick={(e) => { e.stopPropagation(); openFlowTab({ id: 'flow-' + Date.now(), name: 'New Request Flow' }); }}>+</button>
</div>
{#if app.flowsAccordionOpen}
  <div class="sidebar-sub-list">
    {#each sampleFlows as fl (fl.id)}
      <button type="button" class="sidebar-sub-link" onclick={() => openFlowTab(fl)}>
        <span class="sidebar-sub-icon" style="color: #13c2c2;">🔀</span>
        <span class="sidebar-sub-name">{fl.name}</span>
      </button>
    {/each}
  </div>
{/if}
{/if}
