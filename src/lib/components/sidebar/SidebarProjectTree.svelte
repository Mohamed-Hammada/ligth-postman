<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconArrowDown, iconArrowUp, iconCheck, iconChevronDown, iconChevronLeft, iconChevronRight, iconClose, iconCollapseAll, iconCopy, iconEdit, iconExpandAll, iconFolder, iconFolderOpen, iconFolderPlus, iconMoreVertical, iconTrash } from "$lib/components/icons.svelte";
  import type { Folder, Project, RequestSummary } from "$lib/api";
  let { app }: { app: App } = $props();
  const { REQUEST_SORT_FIELDS, deleteFolderAction, deleteProject, deleteRequest, deleteSampleResponseAction, duplicateRequest, exportPostmanCollectionAction, exportProjectFileAction, focusOnMount, onFolderDragStart, onFolderDrop, onProjectDragStart, onProjectDrop, onRequestDragStart, onRequestDrop, openRequest, openRequestContextMenu, openSampleResponse, openSecondaryRequest, pickRequestSortField, quickCreateFolder, quickCreateRequest, selectProject, sortRequestList, startRenameFolder, startRenameProject, startRenameRequest, startRenameSampleResponse, submitRenameFolder, submitRenameProject, submitRenameRequest, submitRenameSampleResponse, t, toggleExpandAllFolders, toggleFolderExpanded, toggleProjectExpand, toggleProjectSelection, toggleTreeRequestExpanded } = app;
</script>

{#if app.sidebarSectionsVisible.collections && app.collectionsAccordionOpen}
      <div class="project-list">
        {#if app.projectSearchScope !== "apis"}
        {#each app.filteredProjects as project (project.id)}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="project-node"
            draggable={app.projectSortField === "custom"}
            class:drag-active={app.projectSortField === "custom"}
            ondragstart={() => onProjectDragStart(project.id)}
            ondragover={(e) => { if (app.projectSortField === "custom") e.preventDefault(); }}
            ondrop={(e) => { e.preventDefault(); onProjectDrop(project.id); }}
          >
            <div class="project-row" class:active={project.id === app.selectedProjectId}>
              {#if app.renamingProjectId === project.id}
                <form class="inline-form" onsubmit={submitRenameProject}>
                  <input bind:value={app.renameProjectValue} use:focusOnMount onblur={submitRenameProject} />
                  <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
                  <button type="button" title={t("sidebar.cancel")} onclick={() => (app.renamingProjectId = null)}>{@render iconClose()}</button>
                </form>
              {:else}
                {@const isOpen = app.expandedProjectIds.has(project.id)}
                <button
                  type="button"
                  class="tree-expand-btn"
                  title={isOpen ? t("sidebar.collapseFolder") : t("sidebar.expandFolder")}
                  onclick={() => toggleProjectExpand(project.id)}
                >{#if isOpen}{@render iconChevronDown()}{:else}{@render iconChevronRight()}{/if}</button>
                <button type="button" class="project-link" onclick={() => toggleProjectSelection(project.id)} ondblclick={() => startRenameProject(project)}>
                  <span class="folder-icon">{#if isOpen}{@render iconFolderOpen()}{:else}{@render iconFolder()}{/if}</span>
                  <span class="project-name">{project.name}</span>
                  {#if app.projectRequestCounts[project.id]}<span class="request-count-badge">{app.projectRequestCounts[project.id]}</span>{/if}
                </button>
                <div class="project-row-actions" class:force-visible={app.openProjectMenuId === project.id}>
                  <button class="icon-btn" title={t("sidebar.addRequest")} onclick={() => quickCreateRequest(project.id)}>+</button>
                  <div class="menu-wrap">
                    <button
                      type="button"
                      class="icon-btn"
                      title={t("sidebar.moreActions")}
                      onclick={() => (app.openProjectMenuId = app.openProjectMenuId === project.id ? null : project.id)}
                    >{@render iconMoreVertical()}</button>
                    {#if app.openProjectMenuId === project.id}
                      <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (app.openProjectMenuId = null)}></button>
                      <div class="dropdown-menu">
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={() => { app.openProjectMenuId = null; quickCreateFolder(project.id); }}
                        >{t("sidebar.addFolder")}</button>
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={async () => {
                            app.openProjectMenuId = null;
                            await selectProject(project.id);
                            app.collectionImportTarget = "current";
                            app.importActiveTab = "collection";
                            app.collectionImportReport = null;
                            app.collectionImportError = "";
                            app.showImportDialog = true;
                          }}
                        >{t("sidebar.importInto")}</button>
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={() => { app.openProjectMenuId = null; exportPostmanCollectionAction(project.id); }}
                        >{t("sidebar.exportCollection")}</button>
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={async () => {
                            app.openProjectMenuId = null;
                            await selectProject(project.id);
                            await exportProjectFileAction();
                            app.gitActiveTab = "projectfile";
                            app.activeScreen = "git";
                          }}
                        >{t("sidebar.exportProjectFile")}</button>
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={() => { app.openProjectMenuId = null; startRenameProject(project); }}
                        >{t("sidebar.rename")}</button>
                        <button
                          type="button"
                          class="dropdown-menu-item"
                          onclick={() => { app.openProjectMenuId = null; deleteProject(project.id); }}
                        >{t("sidebar.delete")}</button>
                      </div>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>

            {#if app.expandedProjectIds.has(project.id) && project.id === app.selectedProjectId}
              <div class="project-requests">
                <div class="request-search-box">
                  <input
                    type="search"
                    placeholder={t("sidebar.searchRequests")}
                    bind:value={app.requestSearchQuery}
                    class="request-search-input"
                  />
                  {#if app.requestSearchQuery}
                    <span class="request-count-badge">{app.filteredRequests.length}/{app.requests.length}</span>
                  {/if}
                  {#if !app.requestSearchQuery && app.folders.length > 0}
                    <button
                      type="button"
                      class="icon-btn"
                      title={app.expandedFolderIds.size < app.folders.length ? t("sidebar.expandAllFolders") : t("sidebar.collapseAllFolders")}
                      onclick={toggleExpandAllFolders}
                    >{#if app.expandedFolderIds.size < app.folders.length}{@render iconExpandAll()}{:else}{@render iconCollapseAll()}{/if}</button>
                  {/if}
                  <div class="menu-wrap">
                    <button
                      type="button"
                      class="icon-btn"
                      title={t("sidebar.sortOptions")}
                      onclick={() => (app.requestSortMenuOpen = !app.requestSortMenuOpen)}
                    >{@render iconMoreVertical()}</button>
                    {#if app.requestSortMenuOpen}
                      <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (app.requestSortMenuOpen = false)}></button>
                      <div class="dropdown-menu">
                        {#each REQUEST_SORT_FIELDS as f (f.field)}
                          <button
                            type="button"
                            class="dropdown-menu-item"
                            class:active={app.requestSortField === f.field}
                            onclick={() => pickRequestSortField(f.field)}
                          >
                            <span>{t(f.label)}</span>
                            {#if app.requestSortField === f.field}
                              <span class="sort-dir-indicator">{#if app.requestSortDir === "asc"}{@render iconArrowUp()}{:else}{@render iconArrowDown()}{/if}</span>
                            {/if}
                          </button>
                        {/each}
                      </div>
                    {/if}
                  </div>
                </div>

                {#if app.loadingRequests}
                  <p class="hint">{t("rail.loading")}</p>
                {:else if app.requestSearchQuery}
                  <ul class="request-list">
                    {#each app.visibleRequests as req (req.id)}
                      {@render requestRow(req)}
                    {:else}
                      <li class="empty">{t("sidebar.noMatchingRequests")}</li>
                    {/each}
                  </ul>

                  {#if app.totalRequestPages > 1}
                    <div class="request-pagination">
                      <button type="button" title={t("sidebar.prevPage")} disabled={app.requestPage === 0} onclick={() => (app.requestPage = Math.max(0, app.requestPage - 1))}>{@render iconChevronLeft()}</button>
                      <span>{app.requestPage + 1} / {app.totalRequestPages}</span>
                      <button type="button" title={t("sidebar.nextPage")} disabled={app.requestPage >= app.totalRequestPages - 1} onclick={() => (app.requestPage = Math.min(app.totalRequestPages - 1, app.requestPage + 1))}>{@render iconChevronRight()}</button>
                    </div>
                  {/if}
                {:else}
                  {#each app.rootFolders as folder (folder.id)}
                    {@render folderNode(project, folder)}
                  {/each}

                  <ul class="request-list">
                    {#each app.rootRequests as req (req.id)}
                      {@render requestRow(req)}
                    {:else}
                      {#if !app.folders.length}
                        <li class="empty">{t("sidebar.noRequestsYet")}</li>
                      {/if}
                    {/each}
                  </ul>
                {/if}
              </div>
            {:else if app.expandedProjectIds.has(project.id) && project.id !== app.selectedProjectId}
              <div class="project-requests project-requests-secondary">
                {#if app.secondaryProjectLoading.has(project.id)}
                  <p class="hint">{t("rail.loading")}</p>
                {:else}
                  {@const cache = app.secondaryProjectCache.get(project.id)}
                  {#if cache}
                    {@const rootFlds = cache.folders.filter((f) => !f.parent_folder_id)}
                    {@const rootReqs = sortRequestList(cache.requests.filter((r) => !r.folder_id))}
                    {#each rootFlds as folder (folder.id)}
                      {@render secondaryFolderNode(project.id, folder, cache)}
                    {/each}
                    <ul class="request-list">
                      {#each rootReqs as req (req.id)}
                        {@render secondaryRequestRow(project.id, req)}
                      {:else}
                        {#if !rootFlds.length}
                          <li class="empty">{t("sidebar.noRequestsYet")}</li>
                        {/if}
                      {/each}
                    </ul>
                  {/if}
                {/if}
              </div>
            {/if}
          </div>
        {:else}
          <p class="empty">{app.projectSearchQuery ? t("sidebar.noMatchingProjects") : t("sidebar.noProjectsYet")}</p>
        {/each}
        {/if}
      </div>
      {/if}

{#snippet requestRow(req: RequestSummary)}
    {@const sampleCount = app.sampleResponsesByRequestId.get(req.id)?.length}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <li
      class="request-item-wrapper"
      draggable={app.requestSortField === "custom"}
      ondragstart={(e) => { e.stopPropagation(); onRequestDragStart(req.id); }}
      ondragover={(e) => { e.stopPropagation(); if (app.requestSortField === "custom") e.preventDefault(); }}
      ondrop={(e) => { e.stopPropagation(); e.preventDefault(); onRequestDrop(req.id, req.folder_id); }}
    >
      <div class="request-item" class:active={req.id === app.selectedRequest?.id} oncontextmenu={(e) => openRequestContextMenu(e, req)}>
        {#if app.renamingRequestId === req.id && req.id !== app.selectedRequest?.id}
          <form class="inline-form" onsubmit={submitRenameRequest}>
            <input bind:value={app.renameRequestValue} use:focusOnMount onblur={submitRenameRequest} />
            <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
            <button type="button" title={t("sidebar.cancel")} onclick={() => (app.renamingRequestId = null)}>{@render iconClose()}</button>
          </form>
        {:else}
          <button
            type="button"
            class="tree-expand-btn"
            title={app.expandedTreeRequestIds.has(req.id) ? t("sidebar.collapseSamples") : t("sidebar.expandSamples")}
            onclick={() => toggleTreeRequestExpanded(req.id)}
          >{#if app.expandedTreeRequestIds.has(req.id)}{@render iconChevronDown()}{:else}{@render iconChevronRight()}{/if}</button>
          <button type="button" class="request-link" onclick={() => openRequest(req.id)} ondblclick={() => startRenameRequest(req.id, req.name)}>
            <span class="sidebar-method method-{req.method.toLowerCase()}">{req.method}</span>
            <span class="request-name">{req.name}</span>
            {#if sampleCount}<span class="tab-badge">{sampleCount}</span>{/if}
          </button>
          <button class="icon-btn icon-btn-ghost" title={t("sidebar.duplicate")} onclick={() => duplicateRequest(req.id)}>{@render iconCopy()}</button>
          <button class="icon-btn icon-btn-ghost" title={t("sidebar.delete")} onclick={() => deleteRequest(req.id)}>{@render iconTrash()}</button>
        {/if}
      </div>
      {#if app.expandedTreeRequestIds.has(req.id)}
        <ul class="sample-tree-list">
          {#each app.sampleResponsesByRequestId.get(req.id) ?? [] as sr (sr.id)}
            <li class="sample-tree-item">
              {#if app.renamingSampleResponseId === sr.id}
                <form class="inline-form" onsubmit={(e) => { e.preventDefault(); submitRenameSampleResponse(req.id); }}>
                  <input bind:value={app.renameSampleResponseValue} use:focusOnMount onblur={() => submitRenameSampleResponse(req.id)} />
                  <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
                  <button type="button" title={t("sidebar.cancel")} onclick={() => (app.renamingSampleResponseId = null)}>{@render iconClose()}</button>
                </form>
              {:else}
                <button
                  type="button"
                  class="sample-tree-link"
                  title={t("sample.badge")}
                  onclick={() => openSampleResponse(req.id, sr)}
                  ondblclick={() => startRenameSampleResponse(sr)}
                >
                  <span class="status-chip" class:status-ok={sr.status < 400} class:status-err={sr.status >= 400}>{sr.status}</span>
                  <span class="sample-tree-name">{sr.name}</span>
                </button>
                <button class="icon-btn icon-btn-ghost" title={t("sidebar.rename")} onclick={() => startRenameSampleResponse(sr)}>{@render iconEdit()}</button>
                <button class="icon-btn icon-btn-ghost" title={t("sample.delete")} onclick={() => deleteSampleResponseAction(req.id, sr.id)}>{@render iconTrash()}</button>
              {/if}
            </li>
          {:else}
            <li class="empty">{t("sidebar.noSamplesYet")}</li>
          {/each}
        </ul>
      {/if}
    </li>
  {/snippet}

{#snippet folderNode(project: Project, folder: Folder)}
    {@const isExpanded = app.expandedFolderIds.has(folder.id)}
    {@const childFolders = app.foldersByParentId.get(folder.id) ?? []}
    {@const childRequests = app.requestsByFolderId.get(folder.id) ?? []}
    <div class="folder-node">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="folder-row"
        draggable={app.requestSortField === "custom"}
        ondragstart={(e) => { e.stopPropagation(); onFolderDragStart(folder.id); }}
        ondragover={(e) => { e.stopPropagation(); if (app.requestSortField === "custom") e.preventDefault(); }}
        ondrop={(e) => { e.stopPropagation(); e.preventDefault(); onFolderDrop(folder.id, folder.parent_folder_id); }}
      >
        {#if app.renamingFolderId === folder.id}
          <form class="inline-form" onsubmit={submitRenameFolder}>
            <input bind:value={app.renameFolderValue} use:focusOnMount onblur={submitRenameFolder} />
            <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
            <button type="button" title={t("sidebar.cancel")} onclick={() => (app.renamingFolderId = null)}>{@render iconClose()}</button>
          </form>
        {:else}
          <button
            type="button"
            class="tree-expand-btn"
            title={isExpanded ? t("sidebar.collapseFolder") : t("sidebar.expandFolder")}
            onclick={() => toggleFolderExpanded(folder.id)}
          >{#if isExpanded}{@render iconChevronDown()}{:else}{@render iconChevronRight()}{/if}</button>
          <button type="button" class="folder-link" onclick={() => toggleFolderExpanded(folder.id)} ondblclick={() => startRenameFolder(folder)}>
            <span class="folder-icon">{#if isExpanded}{@render iconFolderOpen()}{:else}{@render iconFolder()}{/if}</span>
            <span class="project-name">{folder.name}</span>
            {#if childRequests.length}<span class="request-count-badge">{childRequests.length}</span>{/if}
          </button>
          <div class="project-row-actions">
            <button class="icon-btn" title={t("sidebar.addSubfolder")} onclick={() => quickCreateFolder(project.id, folder.id)}>{@render iconFolderPlus()}</button>
            <button class="icon-btn" title={t("sidebar.addRequest")} onclick={() => quickCreateRequest(project.id, folder.id)}>+</button>
            <button class="icon-btn" title={t("sidebar.rename")} onclick={() => startRenameFolder(folder)}>{@render iconEdit()}</button>
            <button class="icon-btn" title={t("sidebar.deleteFolder")} onclick={() => deleteFolderAction(folder.id)}>{@render iconTrash()}</button>
          </div>
        {/if}
      </div>
      {#if isExpanded}
        <div class="folder-children">
          {#each childFolders as child (child.id)}
            {@render folderNode(project, child)}
          {/each}
          <ul class="request-list">
            {#each childRequests as req (req.id)}
              {@render requestRow(req)}
            {:else}
              {#if !childFolders.length}
                <li class="empty">{t("sidebar.noRequestsInFolder")}</li>
              {/if}
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/snippet}

{#snippet secondaryFolderNode(projectId: string, folder: Folder, cache: { requests: RequestSummary[]; folders: Folder[] })}
    {@const isExpanded = app.expandedFolderIds.has(folder.id)}
    {@const childFolders = cache.folders.filter((f) => f.parent_folder_id === folder.id)}
    {@const childRequests = sortRequestList(cache.requests.filter((r) => r.folder_id === folder.id))}
    <div class="folder-node">
      <div class="folder-row">
        <button
          type="button"
          class="tree-expand-btn"
          title={isExpanded ? t("sidebar.collapseFolder") : t("sidebar.expandFolder")}
          onclick={() => toggleFolderExpanded(folder.id)}
        >{#if isExpanded}{@render iconChevronDown()}{:else}{@render iconChevronRight()}{/if}</button>
        <button type="button" class="folder-link" onclick={() => toggleFolderExpanded(folder.id)}>
          <span class="folder-icon">{#if isExpanded}{@render iconFolderOpen()}{:else}{@render iconFolder()}{/if}</span>
          <span class="project-name">{folder.name}</span>
          {#if childRequests.length}<span class="request-count-badge">{childRequests.length}</span>{/if}
        </button>
      </div>
      {#if isExpanded}
        <div class="folder-children">
          {#each childFolders as child (child.id)}
            {@render secondaryFolderNode(projectId, child, cache)}
          {/each}
          <ul class="request-list">
            {#each childRequests as req (req.id)}
              {@render secondaryRequestRow(projectId, req)}
            {:else}
              {#if !childFolders.length}
                <li class="empty">{t("sidebar.noRequestsInFolder")}</li>
              {/if}
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/snippet}

{#snippet secondaryRequestRow(projectId: string, req: RequestSummary)}
    <li class="request-item-wrapper">
      <div class="request-item" oncontextmenu={(e) => openRequestContextMenu(e, req)}>
        <button type="button" class="request-link" onclick={() => openSecondaryRequest(projectId, req.id)}>
          <span class="sidebar-method method-{req.method.toLowerCase()}">{req.method}</span>
          <span class="request-name">{req.name}</span>
        </button>
      </div>
    </li>
  {/snippet}
