<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { closeRequestContextMenu, copyAllAsCurlBashAction, copyAllAsCurlCmdAction, copyAllAsCurlPowerShellAction, copyAllAsFetchAction, copyAllAsHarAction, copyAllAsNodeFetchAction, copyAllUrlsAction, copyContextAsCurlBash, copyContextAsCurlCmd, copyContextAsCurlPowerShell, copyContextAsFetch, copyContextAsHar, copyContextAsNodeFetch, copyContextAsPreload, copyContextAsPythonRequests, copyContextUrl } = app;
</script>

{#if app.requestContextMenu.visible}
  <div
    class="dropdown-backdrop"
    style="background: transparent; z-index: 9998;"
    onclick={closeRequestContextMenu}
    oncontextmenu={(e) => { e.preventDefault(); closeRequestContextMenu(); }}
    role="presentation"
  ></div>
  <div
    class="postman-context-menu"
    style="top: {app.requestContextMenu.y}px; left: {app.requestContextMenu.x}px;"
  >
    <button type="button" class="context-menu-item" onclick={() => copyContextUrl(app.requestContextMenu.request)}>
      <span>Copy URL</span>
    </button>
    <button type="button" class="context-menu-item" onclick={() => copyContextAsCurlBash(app.requestContextMenu.request)}>
      <span>Copy as cURL (bash)</span>
    </button>
    <button type="button" class="context-menu-item" onclick={() => copyContextAsCurlCmd(app.requestContextMenu.request)}>
      <span>Copy as cURL (cmd)</span>
    </button>
    <button type="button" class="context-menu-item" onclick={() => copyContextAsCurlPowerShell(app.requestContextMenu.request)}>
      <span>Copy as cURL (PowerShell)</span>
    </button>
    <button type="button" class="context-menu-item" onclick={() => copyContextAsFetch(app.requestContextMenu.request)}>
      <span>Copy as Fetch</span>
    </button>
    <button type="button" class="context-menu-item" onclick={() => copyContextAsNodeFetch(app.requestContextMenu.request)}>
      <span>Copy as Node.js - Fetch</span>
    </button>
    <button type="button" class="context-menu-item" onclick={() => copyContextAsPythonRequests(app.requestContextMenu.request)}>
      <span>Copy as Python - Requests</span>
    </button>
    <button type="button" class="context-menu-item" onclick={() => copyContextAsPreload(app.requestContextMenu.request)}>
      <span>Copy as Preload element</span>
    </button>
    <button type="button" class="context-menu-item" onclick={() => copyContextAsHar(app.requestContextMenu.request)}>
      <span>Copy as HAR (sanitized)</span>
    </button>
    <div class="context-menu-divider"></div>
    <button type="button" class="context-menu-item" onclick={copyAllUrlsAction}>
      <span>Copy all URLs</span>
    </button>
    <button type="button" class="context-menu-item" onclick={copyAllAsCurlBashAction}>
      <span>Copy all as cURL (bash)</span>
    </button>
    <button type="button" class="context-menu-item" onclick={copyAllAsCurlCmdAction}>
      <span>Copy all as cURL (cmd)</span>
    </button>
    <button type="button" class="context-menu-item" onclick={copyAllAsCurlPowerShellAction}>
      <span>Copy all as cURL (PowerShell)</span>
    </button>
    <button type="button" class="context-menu-item" onclick={copyAllAsFetchAction}>
      <span>Copy all as Fetch</span>
    </button>
    <button type="button" class="context-menu-item" onclick={copyAllAsNodeFetchAction}>
      <span>Copy all as Node.js - Fetch</span>
    </button>
    <button type="button" class="context-menu-item" onclick={copyAllAsHarAction}>
      <span>Copy all as HAR (sanitized)</span>
    </button>
  </div>
{/if}
