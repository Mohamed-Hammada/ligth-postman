<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconChevronRight, iconClock, iconCube, iconFolder, iconGlobe } from "$lib/components/icons.svelte";
  import SidebarHistory from "$lib/components/sidebar/SidebarHistory.svelte";
  import SidebarEnvironments from "$lib/components/sidebar/SidebarEnvironments.svelte";
  import SidebarSearch from "$lib/components/sidebar/SidebarSearch.svelte";
  import SidebarProjectTree from "$lib/components/sidebar/SidebarProjectTree.svelte";
  import SidebarEnvAccordion from "$lib/components/sidebar/SidebarEnvAccordion.svelte";
  import SidebarResourceSections from "$lib/components/sidebar/SidebarResourceSections.svelte";
  let { app }: { app: App } = $props();
  const { ensureAllEnvVariablesLoaded, refreshProjectHistory, setSidebarVisible, startSidebarResize, t, toggleEnvExpand } = app;
</script>

{#if app.sidebarVisible}
<aside class="sidebar" style="width: {app.sidebarWidth}px">
  <div class="sidebar-top-icons">
    <button type="button" class="sidebar-top-icon-btn" class:active={app.sidebarSection === "collections"} title="Collections & APIs" onclick={() => { app.sidebarSection = "collections"; app.activeScreen = "workspace"; }}>{@render iconCube()}</button>
    <button type="button" class="sidebar-top-icon-btn" class:active={app.sidebarSection === "environments"} title="Environments" onclick={() => { app.sidebarSection = "environments"; app.environmentsAccordionOpen = true; app.activeScreen = "workspace"; ensureAllEnvVariablesLoaded(); const targetId = app.selectedEnvironmentId || app.allEnvironments[0]?.id; if (targetId && !app.expandedEnvIds.has(targetId)) toggleEnvExpand(targetId); }}>{@render iconGlobe()}</button>
    <button type="button" class="sidebar-top-icon-btn" class:active={app.sidebarSection === "history"} title="History" onclick={() => { app.sidebarSection = "history"; app.activeScreen = "workspace"; refreshProjectHistory(); }}>{@render iconClock()}</button>
    <button type="button" class="sidebar-top-icon-btn" class:active={app.sidebarSection === "projects"} title="Projects & Workspaces" onclick={() => { app.sidebarSection = "projects"; app.activeScreen = "workspace"; }}>{@render iconFolder()}</button>
  </div>

  {#if app.sidebarSection === "history"}
    <SidebarHistory {app} />
  {:else if app.sidebarSection === "environments"}
    <SidebarEnvironments {app} />
  {:else}
    <SidebarSearch {app} />

  <SidebarProjectTree {app} />

  <SidebarEnvAccordion {app} />

  <SidebarResourceSections {app} />
  {/if}
</aside>

<div
  class="sidebar-resize-handle"
  class:resizing={app.sidebarResizing}
  onmousedown={startSidebarResize}
  onkeydown={(e) => {
    app.sidebarManuallyResized = true;
    if (e.key === "ArrowLeft") app.sidebarWidth = Math.max(200, app.sidebarWidth - 16);
    else if (e.key === "ArrowRight") app.sidebarWidth = Math.min(600, app.sidebarWidth + 16);
  }}
  role="slider"
  aria-orientation="vertical"
  aria-label="Resize sidebar"
  aria-valuenow={app.sidebarWidth}
  aria-valuemin={200}
  aria-valuemax={600}
  tabindex="0"
></div>
{:else}
<button type="button" class="sidebar-expand-btn" title={t("sidebar.show")} onclick={() => setSidebarVisible(true)}>{@render iconChevronRight()}</button>
{/if}
