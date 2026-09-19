<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconCheck, iconChevronDown, iconClose, iconCopy, iconSave } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { focusOnMount, openRequestContextMenu, saveRequest, startRenameRequest, submitRenameRequest, t } = app;
</script>

{#if app.selectedRequest}
<div class="breadcrumb-row">
    <div class="breadcrumb-left">
    <span class="http-pill">HTTP</span>
    <div class="breadcrumb-trail">
      <span class="breadcrumb-path">{app.projects.find((p) => p.id === app.selectedProjectId)?.name ?? ""}</span>
      {#each app.selectedRequestFolderChain as folderName (folderName)}
        <span class="breadcrumb-sep">&rsaquo;</span>
        <span class="breadcrumb-path">{folderName}</span>
      {/each}
      <span class="breadcrumb-sep">&rsaquo;</span>
      {#if app.renamingRequestId === app.selectedRequest.id}
        <form class="inline-form" onsubmit={submitRenameRequest}>
          <input bind:value={app.renameRequestValue} use:focusOnMount onblur={submitRenameRequest} />
          <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
          <button type="button" title={t("sidebar.cancel")} onclick={() => (app.renamingRequestId = null)}>{@render iconClose()}</button>
        </form>
      {:else}
        <button
          type="button"
          class="breadcrumb-current"
          title={t("breadcrumb.renameHint")}
          onclick={() => startRenameRequest(app.selectedRequest!.id, app.selectedRequest!.name)}
        >
          {app.selectedRequest.name}
        </button>
      {/if}
    </div>
  </div>
  <div class="breadcrumb-right">
    <div class="btn-save-split">
      <button type="button" class="btn-save-main" onclick={() => saveRequest()}>
        {@render iconSave()} <span>{t("request.saveLabel")}</span>
      </button>
      <button type="button" class="btn-save-caret" onclick={() => saveRequest()}>{@render iconChevronDown()}</button>
    </div>
    <button type="button" class="btn-share" onclick={(e) => openRequestContextMenu(e, app.selectedRequest)}>
      {@render iconCopy()} <span>Share</span>
    </button>
  </div>
</div>
{/if}
