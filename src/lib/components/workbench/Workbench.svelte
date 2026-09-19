<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconFileText, iconFolder } from "$lib/components/icons.svelte";
  import TabsBar from "$lib/components/workbench/TabsBar.svelte";
  import EnvTabView from "$lib/components/workbench/EnvTabView.svelte";
  import DocTabView from "$lib/components/workbench/DocTabView.svelte";
  import SpecTabView from "$lib/components/workbench/SpecTabView.svelte";
  import MockTabView from "$lib/components/workbench/MockTabView.svelte";
  import DatasetTabView from "$lib/components/workbench/DatasetTabView.svelte";
  import FlowTabView from "$lib/components/workbench/FlowTabView.svelte";
  import RequestBreadcrumb from "$lib/components/workbench/RequestBreadcrumb.svelte";
  import RequestBar from "$lib/components/workbench/RequestBar.svelte";
  import VariablePopover from "$lib/components/workbench/VariablePopover.svelte";
  import UrlAutocomplete from "$lib/components/workbench/UrlAutocomplete.svelte";
  import RequestWarnings from "$lib/components/workbench/RequestWarnings.svelte";
  import EditorTabs from "$lib/components/workbench/EditorTabs.svelte";
  import EditorPane from "$lib/components/workbench/EditorPane.svelte";
  import ResponsePane from "$lib/components/workbench/ResponsePane.svelte";
  let { app }: { app: App } = $props();
  const { t } = app;
</script>

<main class="main">
  {#if !app.selectedProjectId && !app.currentTab && app.openTabs.length === 0}
    <div class="empty-state">
      <div class="empty-icon">{@render iconFolder()}</div>
      <p>{t("workspace.selectProject")}</p>
    </div>
  {:else if !app.selectedRequest && !app.currentTab}
    <div class="empty-state">
      <div class="empty-icon">{@render iconFileText()}</div>
      <p>{t("request.selectPrompt")}</p>
    </div>
  {:else}
    <section class="detail">
      <TabsBar {app} />

      {#if app.currentTab?.tabType === "env"}
        <EnvTabView {app} />
      {:else if app.currentTab?.tabType === "doc"}
        <DocTabView {app} />
      {:else if app.currentTab?.tabType === "spec"}
        <SpecTabView {app} />
      {:else if app.currentTab?.tabType === "mock"}
        <MockTabView {app} />
      {:else if app.currentTab?.tabType === "dataset"}
        <DatasetTabView {app} />
      {:else if app.currentTab?.tabType === "flow"}
        <FlowTabView {app} />
      {:else if app.selectedRequest}
        <RequestBreadcrumb {app} />

      <RequestBar {app} />

      <VariablePopover {app} />

      <UrlAutocomplete {app} />

      <RequestWarnings {app} />

      <EditorTabs {app} />

      <div class="editor-body-row">
      <EditorPane {app} />

      <ResponsePane {app} />
      </div>
      {/if}
    </section>
  {/if}
</main>
