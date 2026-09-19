<script lang="ts">
  import { createApp } from "$lib/app/controller.svelte";
  import { iconCheckCircle, iconClose, iconXCircle } from "$lib/components/icons.svelte";
  import ConfirmDialog from "$lib/components/shell/ConfirmDialog.svelte";
  import TopBar from "$lib/components/shell/TopBar.svelte";
  import ExpandedResponseView from "$lib/components/screens/ExpandedResponseView.svelte";
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import Workbench from "$lib/components/workbench/Workbench.svelte";
  import RightSidebar from "$lib/components/panels/RightSidebar.svelte";
  import RightRail from "$lib/components/panels/RightRail.svelte";
  import AiPanel from "$lib/components/modals/AiPanel.svelte";
  import DiffModal from "$lib/components/modals/DiffModal.svelte";
  import HistoryModal from "$lib/components/modals/HistoryModal.svelte";
  import GlobalsScreen from "$lib/components/screens/GlobalsScreen.svelte";
  import EnvironmentsScreen from "$lib/components/screens/EnvironmentsScreen.svelte";
  import GitScreen from "$lib/components/screens/GitScreen.svelte";
  import LauncherScreen from "$lib/components/screens/LauncherScreen.svelte";
  import HistoryScreen from "$lib/components/screens/HistoryScreen.svelte";
  import SettingsScreen from "$lib/components/screens/SettingsScreen.svelte";
  import ConsoleDrawer from "$lib/components/shell/ConsoleDrawer.svelte";
  import StatusBar from "$lib/components/shell/StatusBar.svelte";
  import InviteModal from "$lib/components/modals/InviteModal.svelte";
  import ImportDialog from "$lib/components/modals/ImportDialog.svelte";
  import CommandPalette from "$lib/components/shell/CommandPalette.svelte";
  import TabContextMenu from "$lib/components/shell/TabContextMenu.svelte";
  import RequestContextMenu from "$lib/components/shell/RequestContextMenu.svelte";

  const app = createApp();
  const { accentStyleOverride, fontStyleOverride, startRightSidebarResize, surfaceStyleOverride, t, textColorStyleOverride } = app;
</script>

<ConfirmDialog {app} />

<div
  class="app-shell"
  data-theme={app.themeMode}
  style="{surfaceStyleOverride(app.surfaceTint, app.themeMode)} {accentStyleOverride(app.accentColor)} {fontStyleOverride(app.headingFontOverride, app.bodyFontOverride)} {textColorStyleOverride(app.textColorOverride)}"
>
  <TopBar {app} />
  

  

  

  

  <!-- Recursive: a folder can contain child folders (unlimited depth), each independently
       collapsible/expandable via the same expandedFolderIds set as its parent — clicking a
       folder toggles it open/closed regardless of how deep it's nested. -->
  

  <!-- Lightweight, read-mostly counterpart to folderNode/requestRow for a project that's
       expanded in the sidebar but not the active one — browse and open, no rename/delete/create
       (open a request here promotes its project to active, which gets you the full tree). -->
  

  

  

  <div class="screen-area">
  {#if app.activeScreen === "workspace" && app.responseExpanded}
    <ExpandedResponseView {app} />
  {:else}
    <div class="app">
      {#if app.exportFeedback}
        <div class="success-banner"><span class="banner-message">{@render iconCheckCircle()} {app.exportFeedback}</span></div>
      {/if}
      {#if app.errorMessage}
        <div class="error-banner">
          <span class="banner-message">{@render iconXCircle()} {app.errorMessage}</span>
          <button type="button" class="dismiss-btn" title={t("error.dismiss")} onclick={() => (app.errorMessage = "")}>{@render iconClose()}</button>
        </div>
      {/if}

      <div class="workspace">
        <!-- The sidebar stays mounted on every screen, so choosing Globals / Environments / Git /
             Settings swaps only the pane beside it — the navigation tree never disappears. -->
        <Sidebar {app} />

        {#if app.activeScreen === "workspace"}
          <Workbench {app} />

          {#if app.selectedProjectId && app.rightSidebarVisible}
            <div
              class="right-sidebar-resize-handle"
              class:resizing={app.rightSidebarResizing}
              onmousedown={startRightSidebarResize}
              onkeydown={(e) => {
                if (e.key === "ArrowLeft") app.rightSidebarWidth = Math.min(640, app.rightSidebarWidth + 16);
                else if (e.key === "ArrowRight") app.rightSidebarWidth = Math.max(260, app.rightSidebarWidth - 16);
              }}
              role="slider"
              aria-orientation="vertical"
              aria-label={t("rightSidebar.resizeHandle")}
              aria-valuenow={app.rightSidebarWidth}
              aria-valuemin={260}
              aria-valuemax={640}
              tabindex="0"
            ></div>
            <RightSidebar {app} />
          {/if}

          <!-- Persistent Right Utility Rail matching p3, p4 -->
          {#if app.utilityRailVisible}
            <RightRail {app} />
          {/if}
        {:else}
          <div class="screen-host">
            {#if app.activeScreen === "globals"}
              <GlobalsScreen {app} />
            {:else if app.activeScreen === "environments"}
              <EnvironmentsScreen {app} />
            {:else if app.activeScreen === "git"}
              <GitScreen {app} />
            {:else if app.activeScreen === "launcher"}
              <LauncherScreen {app} />
            {:else if app.activeScreen === "history"}
              <HistoryScreen {app} />
            {:else if app.activeScreen === "settings"}
              <SettingsScreen {app} />
            {/if}
          </div>
        {/if}
      </div>

      <AiPanel {app} />

      <DiffModal {app} />

      <HistoryModal {app} />
    </div>
  {/if}
  </div>

  <ConsoleDrawer {app} />

  <StatusBar {app} />

  <InviteModal {app} />

  <ImportDialog {app} />

  <CommandPalette {app} />

  {#if app.textCopiedNotice}
    <div style="position: fixed; bottom: 40px; right: 24px; z-index: 99999; background: #0cbb52; color: #fff; padding: 8px 16px; border-radius: 4px; font-weight: 500; font-size: 13px; box-shadow: 0 4px 12px rgba(0,0,0,0.4); display: flex; align-items: center; gap: 8px;">
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 8.5l3.5 3.5L13 4"/></svg>
      <span>{app.textCopiedNotice}</span>
    </div>
  {/if}

  <TabContextMenu {app} />

  <RequestContextMenu {app} />
</div>
