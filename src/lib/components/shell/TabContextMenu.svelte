<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { closeTabContextMenu, handleTabMenuCloseAllTabs, handleTabMenuCloseOtherTabs, handleTabMenuCloseTab, handleTabMenuDuplicateTab, handleTabMenuForceCloseAllTabs, handleTabMenuForceCloseTab, handleTabMenuNewRequest, handleTabMenuRevealInSidebar } = app;
</script>

{#if app.tabContextMenu.visible}
  <div
    class="dropdown-backdrop"
    style="background: transparent; z-index: 9998;"
    onclick={closeTabContextMenu}
    oncontextmenu={(e) => { e.preventDefault(); closeTabContextMenu(); }}
    role="presentation"
  ></div>
  <div
    class="postman-tab-context-menu"
    style="top: {app.tabContextMenu.y}px; left: {app.tabContextMenu.x}px;"
  >
    <button type="button" class="tab-context-menu-item" onclick={handleTabMenuNewRequest}>
      <span class="tab-menu-label">New Request</span>
      <span class="tab-menu-shortcut">Ctrl+T</span>
    </button>
    <button type="button" class="tab-context-menu-item" onclick={handleTabMenuDuplicateTab}>
      <span class="tab-menu-label">Duplicate Tab</span>
    </button>

    <div class="tab-context-menu-divider"></div>

    <button type="button" class="tab-context-menu-item" onclick={handleTabMenuCloseTab}>
      <span class="tab-menu-label">Close Tab</span>
      <span class="tab-menu-shortcut">Ctrl+W</span>
    </button>
    <button type="button" class="tab-context-menu-item" onclick={handleTabMenuForceCloseTab}>
      <span class="tab-menu-label">Force Close Tab</span>
      <span class="tab-menu-shortcut">Alt+Ctrl+W</span>
    </button>
    <button type="button" class="tab-context-menu-item" onclick={handleTabMenuCloseOtherTabs}>
      <span class="tab-menu-label">Close Other Tabs</span>
    </button>
    <button type="button" class="tab-context-menu-item" onclick={handleTabMenuCloseAllTabs}>
      <span class="tab-menu-label">Close All Tabs</span>
    </button>
    <button type="button" class="tab-context-menu-item" onclick={handleTabMenuForceCloseAllTabs}>
      <span class="tab-menu-label">Force Close All Tabs</span>
    </button>

    <div class="tab-context-menu-divider"></div>

    <button type="button" class="tab-context-menu-item" onclick={handleTabMenuRevealInSidebar}>
      <span class="tab-menu-label">Reveal in Sidebar</span>
    </button>
  </div>
{/if}
