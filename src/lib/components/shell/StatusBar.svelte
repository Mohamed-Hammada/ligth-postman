<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconGitBranch, iconLayout, iconSidebar } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { loadVariables, refreshConsoleEvents, refreshGitStatus, setRightSidebarVisible, setSidebarVisible, t, toggleUtilityRail } = app;
</script>

<footer class="app-status-bar">
  <div class="status-left">
    <button
      type="button"
      class="status-btn icon-only"
      class:active={app.sidebarVisible}
      title={app.sidebarVisible ? "Hide sidebar (Project tree)" : "Show sidebar (Project tree)"}
      onclick={() => setSidebarVisible(!app.sidebarVisible)}
    >
      {@render iconSidebar()}
    </button>
    {#if app.selectedProjectId}
      <button
        type="button"
        class="status-btn git-btn"
        title={t("footer.gitSyncTitle")}
        onclick={() => { app.activeScreen = "git"; if (app.gitRepoPathInput) refreshGitStatus(); }}
      >
        <span class="git-icon">{@render iconGitBranch()}</span>
        <span>{app.gitStatus?.branch || "main"}</span>
        <span class="git-sync-arrows">&#8644;</span>
      </button>
    {/if}
    <button
      type="button"
      class="status-btn console-btn"
      class:active={app.showConsole}
      onclick={() => {
        app.showConsole = !app.showConsole;
        if (app.showConsole) refreshConsoleEvents();
      }}
    >
      <span>{t("console.title")}</span>
      {#if app.consoleErrorCount > 0}<span class="status-badge-err">! {app.consoleErrorCount}</span>{/if}
      {#if app.consoleWarnCount > 0}<span class="status-badge-warn">! {app.consoleWarnCount}</span>{/if}
    </button>
  </div>
  <div class="status-right">
    <button
      type="button"
      class="status-btn"
      class:active={app.activeScreen === "globals"}
      onclick={() => {
        loadVariables();
        app.activeScreen = "globals";
        app.rightPanel = "variables";
        setRightSidebarVisible(true);
      }}
    >
      <span>Globals</span>
    </button>
    <button type="button" class="status-btn" class:active={app.activeScreen === "environments"} onclick={() => { loadVariables(); app.activeScreen = "environments"; }}>
      <span>Environments</span>
    </button>
    <button type="button" class="status-btn" class:active={app.activeScreen === "settings"} onclick={() => (app.activeScreen = app.activeScreen === "settings" ? "workspace" : "settings")}>
      <span>{t("rail.settings")}</span>
    </button>
    <button
      type="button"
      class="status-btn icon-only"
      class:active={app.utilityRailVisible}
      title={app.utilityRailVisible ? "Hide menu items rail" : "Show menu items rail"}
      onclick={toggleUtilityRail}
    >
      {@render iconLayout()}
    </button>
  </div>
</footer>
