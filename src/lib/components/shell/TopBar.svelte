<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconArrowLeft, iconArrowRight, iconBell, iconCheck, iconChevronDown, iconClose, iconEdit, iconHome, iconImport, iconSearch, iconSettings, iconSparkle, iconUser } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { focusOnMount, handleWindowClose, handleWindowMaximize, handleWindowMinimize, navigateHistory, openInviteModal, openPalette, quickCreateWorkspace, selectWorkspace, startRenameWorkspace, submitRenameWorkspace, t } = app;
</script>

<header class="topbar">
  <div class="topbar-left">
    <div class="topbar-nav-arrows">
      <button type="button" class="nav-arrow-btn" title={t("nav.back")} aria-label={t("nav.back")} disabled={!app.canGoBack} onclick={() => navigateHistory(-1)}>{@render iconArrowLeft()}</button>
      <button type="button" class="nav-arrow-btn" title={t("nav.forward")} aria-label={t("nav.forward")} disabled={!app.canGoForward} onclick={() => navigateHistory(1)}>{@render iconArrowRight()}</button>
      <button type="button" class="nav-arrow-btn" title={t("nav.home")} aria-label={t("nav.home")} onclick={() => (app.activeScreen = "workspace")}>{@render iconHome()}</button>
    </div>
    <div class="menu-wrap topbar-workspace-wrap">
      <button
        type="button"
        class="topbar-workspace-btn"
        onclick={() => (app.workspacePickerOpen = !app.workspacePickerOpen)}
      >
        <span class="workspace-avatar-icon">{@render iconUser()}</span>
        <span class="workspace-name">{app.workspaces.find((w) => w.id === app.activeWorkspaceId)?.name ?? t("workspace.defaultName")}</span>
        {@render iconChevronDown()}
      </button>
      {#if app.workspacePickerOpen}
        <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (app.workspacePickerOpen = false)}></button>
        <div class="dropdown-menu workspace-picker-menu">
          {#each app.workspaces as ws (ws.id)}
            {#if app.renamingWorkspaceId === ws.id}
              <form class="inline-form" onsubmit={submitRenameWorkspace}>
                <input bind:value={app.renameWorkspaceValue} use:focusOnMount onblur={submitRenameWorkspace} />
                <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
                <button type="button" title={t("sidebar.cancel")} onclick={() => (app.renamingWorkspaceId = null)}>{@render iconClose()}</button>
              </form>
            {:else}
              <div class="dropdown-menu-item workspace-picker-item" class:active={ws.id === app.activeWorkspaceId}>
                <button type="button" class="workspace-picker-item-btn" onclick={() => selectWorkspace(ws.id)}>{ws.name}</button>
                <button type="button" class="icon-btn icon-btn-ghost" title={t("sidebar.rename")} onclick={() => startRenameWorkspace(ws)}>{@render iconEdit()}</button>
              </div>
            {/if}
          {/each}
          <button type="button" class="dropdown-menu-item" onclick={quickCreateWorkspace}>+ {t("workspace.newWorkspace")}</button>
          <div class="dropdown-menu-divider"></div>
          <button type="button" class="dropdown-menu-item" onclick={() => { app.workspacePickerOpen = false; app.activeScreen = "launcher"; }}>{t("rail.launcher")}</button>
          <button type="button" class="dropdown-menu-item" onclick={() => { app.workspacePickerOpen = false; app.activeScreen = "history"; }}>{t("rail.history")}</button>
        </div>
      {/if}
    </div>
  </div>
  <div class="topbar-center">
    <button type="button" class="topbar-search-pill" title={t("topbar.searchPlaceholder")} onclick={openPalette}>
      {@render iconSearch()}
      <span class="topbar-search-text">Search</span>
      <span class="topbar-search-kbd">&#8984;K</span>
    </button>
  </div>
  <div class="topbar-right">
    <button type="button" class="topbar-btn topbar-btn-subtle" onclick={openInviteModal}>{t("invite.button")}</button>
    <button
      type="button"
      class="topbar-btn topbar-btn-ai"
      title={app.aiConfigured ? t("topbar.askAiTitle") : t("topbar.setupAiTitle")}
      onclick={() => (app.showAiPanel = true)}
    >
      {@render iconSparkle()} {app.aiConfigured ? t("topbar.askAi") : t("topbar.setupAi")}
    </button>
    <button
      type="button"
      class="topbar-btn"
      title={t("topbar.importTitle")}
      onclick={() => { app.importActiveTab = "collection"; app.collectionImportReport = null; app.collectionImportError = ""; app.showImportDialog = true; }}
    >
      {@render iconImport()} {t("topbar.import")}
    </button>
    <div class="menu-wrap">
      <button
        type="button"
        class="topbar-icon-btn notif-trigger"
        title={t("notifications.title")}
        aria-label={t("notifications.title")}
        aria-expanded={app.notificationsOpen}
        onclick={() => (app.notificationsOpen = !app.notificationsOpen)}
      >
        {@render iconBell()}
        {#if app.notifications.length > 0}<span class="notif-dot" aria-hidden="true">{app.notifications.length}</span>{/if}
      </button>
      {#if app.notificationsOpen}
        <button type="button" class="dropdown-backdrop notif-backdrop" aria-label={t("common.close")} onclick={() => (app.notificationsOpen = false)}></button>
        <div class="dropdown-menu notif-menu">
          <div class="notif-head">{t("notifications.title")}</div>
          {#each app.notifications as n (n.id)}
            <div class="notif-row notif-{n.level}">
              <div class="notif-row-title">{n.title}</div>
              <div class="notif-row-detail">{n.detail}</div>
              {#if n.action && n.actionLabel}
                <button type="button" class="notif-action" onclick={() => { app.notificationsOpen = false; n.action?.(); }}>{n.actionLabel}</button>
              {/if}
            </div>
          {:else}
            <div class="notif-empty">{t("notifications.empty")}</div>
          {/each}
        </div>
      {/if}
    </div>
    <button
      type="button"
      class="topbar-icon-btn"
      title={t("settings.title")}
      onclick={() => (app.activeScreen = app.activeScreen === "settings" ? "workspace" : "settings")}
    >
      {@render iconSettings()}
    </button>
    <div class="menu-wrap" style="z-index: 60;">
      <button
        type="button"
        class="topbar-avatar"
        title={t("account.localProfile")}
        onclick={() => (app.accountMenuOpen = !app.accountMenuOpen)}
        aria-label={t("account.localProfile")}
        aria-expanded={app.accountMenuOpen}
      >
        <span>D</span>
      </button>
      {#if app.accountMenuOpen}
        <button type="button" class="dropdown-backdrop notif-backdrop" aria-label={t("common.close")} onclick={() => (app.accountMenuOpen = false)}></button>
        <div class="dropdown-menu account-menu">
          <div class="account-head">
            <div class="account-head-name">{t("account.localProfile")}</div>
            <div class="account-head-hint">{t("account.localProfileHint")}</div>
          </div>
          <button type="button" class="dropdown-menu-item" onclick={() => { app.accountMenuOpen = false; app.activeScreen = "settings"; }}>{t("account.preferences")}</button>
          <button type="button" class="dropdown-menu-item" onclick={() => { app.accountMenuOpen = false; app.activeScreen = "git"; }}>{t("account.gitSync")}</button>
        </div>
      {/if}
    </div>
    <div class="window-controls">
      <button type="button" class="win-ctrl-btn" title="Minimize" onclick={handleWindowMinimize}>&minus;</button>
      <button type="button" class="win-ctrl-btn" title="Maximize" onclick={handleWindowMaximize}>&#9634;</button>
      <button type="button" class="win-ctrl-btn win-ctrl-close" title="Close" onclick={handleWindowClose}>&#10005;</button>
    </div>
  </div>
</header>
