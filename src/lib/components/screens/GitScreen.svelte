<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconFileText, iconInboxEmpty } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { adoptLegacyGitSettings, checkGitHubRepoAction, commitAndPushAction, dismissLegacyGitCandidates, downloadProjectFile, exportProjectFileAction, importProjectFileAction, initializeGitRepoAction, loadConflictVersions, pullRepositoryAction, refreshGitStatus, resolveConflictAction, saveGitSettingsAction, saveWorkspaceToRepoAction, t, verifyGitHubTokenAction, viewDiffAction, viewHistoryAction } = app;
</script>

{#if !app.activeWorkspaceId}
  <div class="screen-empty">
    <div class="empty-icon">{@render iconInboxEmpty()}</div>
    <p>{t("rail.loading")}</p>
  </div>
{:else}
  <section class="screen-page">
    <div class="screen-page-header">
      <span class="screen-kicker">{t("git.title")}</span>
      <div class="screen-title-row">
        <h1 class="screen-title">{app.workspaces.find((w) => w.id === app.activeWorkspaceId)?.name ?? t("workspace.defaultName")}</h1>
      </div>
      <p class="screen-subtitle">{t("git.workspaceScopeHint")}</p>
    </div>

    <div class="modal-tabs" style="flex:none; padding: 0 var(--space-6);">
        <button
          type="button"
          class="modal-tab-btn"
          class:active={app.gitActiveTab === "sync"}
          onclick={() => (app.gitActiveTab = "sync")}
        >
          {t("git.repoSync")}
        </button>
        <button
          type="button"
          class="modal-tab-btn"
          class:active={app.gitActiveTab === "conflicts"}
          onclick={() => (app.gitActiveTab = "conflicts")}
        >
          {t("git.conflicts")}
          {#if app.gitStatus?.has_conflicts}
            <span class="tab-badge-alert">{app.gitStatus.conflict_files.length}</span>
          {/if}
        </button>
        <button
          type="button"
          class="modal-tab-btn"
          class:active={app.gitActiveTab === "github"}
          onclick={() => (app.gitActiveTab = "github")}
        >
          {t("git.githubAuth")}
        </button>
        <button
          type="button"
          class="modal-tab-btn"
          class:active={app.gitActiveTab === "projectfile"}
          onclick={() => (app.gitActiveTab = "projectfile")}
        >
          {t("git.projectFile")}
        </button>
      </div>

      <div class="screen-page-body">
        {#if app.gitActionFeedback}
          <div class="action-alert success">{app.gitActionFeedback}</div>
        {/if}
        {#if app.gitActionError}
          <div class="action-alert error">{app.gitActionError}</div>
        {/if}

        {#if app.gitActiveTab === "sync"}
          {#if app.legacyGitCandidates.length}
            <div class="git-panel-section legacy-git-banner">
              <h4>{t("git.legacyFoundTitle")}</h4>
              <p class="hint">{t("git.legacyFoundHint")}</p>
              <ul class="legacy-git-list">
                {#each app.legacyGitCandidates as candidate (candidate.project_id)}
                  <li class="legacy-git-row">
                    <div class="legacy-git-row-info">
                      <strong>{candidate.project_name}</strong>
                      <span class="hint">{candidate.settings.repo_path}</span>
                    </div>
                    <button type="button" class="btn-primary btn-xs" onclick={() => adoptLegacyGitSettings(candidate)}>{t("git.legacyUseThis")}</button>
                  </li>
                {/each}
              </ul>
              <button type="button" class="btn-ghost btn-xs" onclick={dismissLegacyGitCandidates}>{t("git.legacyDismiss")}</button>
            </div>
          {/if}
          <div class="git-panel-section">
            <h4>{t("git.repositorySettings")}</h4>
            <div class="form-row-stacked">
              <label for="git-repo-path-input">{t("git.repoPathLabel")}</label>
              <div class="input-with-actions">
                <input
                  id="git-repo-path-input"
                  type="text"
                  placeholder={t("git.repoPathPlaceholder")}
                  bind:value={app.gitRepoPathInput}
                  class="path-input"
                />
                <button type="button" onclick={saveGitSettingsAction}>{t("git.savePath")}</button>
                <button type="button" onclick={() => refreshGitStatus()} disabled={!app.gitRepoPathInput.trim() || app.gitStatusLoading}>
                  {app.gitStatusLoading ? t("git.checkingStatus") : t("git.checkStatus")}
                </button>
              </div>
              <p class="hint">{t("git.repoPathHint")}</p>
            </div>

            {#if !app.gitStatus || !app.gitStatus.is_repo}
              <div class="alert-box-warning">
                <p><strong>{t("git.notARepoTitle")}</strong> {t("git.notARepoDesc")}</p>
                <button
                  type="button"
                  class="btn-primary"
                  disabled={!app.gitRepoPathInput.trim() || app.gitLoading}
                  onclick={initializeGitRepoAction}
                >
                  {app.gitLoading ? t("git.initializing") : t("git.initializeRepo")}
                </button>
              </div>
            {:else}
              <div class="git-status-card">
                <div class="status-summary-row">
                  <span class="status-label">{t("git.branch")}</span>
                  <strong>{app.gitStatus.branch}</strong>
                  <span class="status-sep">|</span>
                  <span class="status-label">{t("git.status")}</span>
                  <span class="git-badge-kind kind-{app.gitStatus.status_kind}">{app.gitStatus.status_kind.toUpperCase()}</span>
                  {#if app.gitStatus.ahead > 0}
                    <span class="badge-ahead">{t("git.unpushed", { count: app.gitStatus.ahead })}</span>
                  {/if}
                  {#if app.gitStatus.behind > 0}
                    <span class="badge-behind">{t("git.unpulled", { count: app.gitStatus.behind })}</span>
                  {/if}
                </div>

                {#if app.gitStatus.staged_files.length > 0 || app.gitStatus.unstaged_files.length > 0 || app.gitStatus.untracked_files.length > 0}
                  <div class="files-changed-summary">
                    {#if app.gitStatus.staged_files.length > 0}
                      <p class="file-category">{t("git.staged")} <code>{app.gitStatus.staged_files.join(", ")}</code></p>
                    {/if}
                    {#if app.gitStatus.unstaged_files.length > 0}
                      <p class="file-category">{t("git.modified")} <code>{app.gitStatus.unstaged_files.join(", ")}</code></p>
                    {/if}
                    {#if app.gitStatus.untracked_files.length > 0}
                      <p class="file-category">{t("git.untracked")} <code>{app.gitStatus.untracked_files.join(", ")}</code></p>
                    {/if}
                  </div>
                {:else}
                  <p class="working-tree-clean">{t("git.workingTreeClean")}</p>
                {/if}
              </div>

              <div class="git-commit-box">
                <h4>{t("git.manualSync")}</h4>
                <div class="commit-input-row">
                  <input
                    type="text"
                    placeholder={t("git.commitMessagePlaceholder")}
                    bind:value={app.gitCommitMessage}
                  />
                  <button
                    type="button"
                    class="btn-primary"
                    disabled={app.gitLoading}
                    onclick={commitAndPushAction}
                  >
                    {app.gitLoading ? t("git.syncing") : t("git.commitPush")}
                  </button>
                  <button
                    type="button"
                    disabled={app.gitLoading}
                    onclick={pullRepositoryAction}
                  >
                    {app.gitLoading ? t("git.pulling") : t("git.pullRemote")}
                  </button>
                </div>
                <div class="quick-git-actions">
                  <button type="button" class="icon-btn-text" onclick={saveWorkspaceToRepoAction} disabled={app.gitLoading}>
                    {t("git.saveProjectFile")}
                  </button>
                  <button type="button" class="icon-btn-text" onclick={viewDiffAction} disabled={app.gitLoading}>
                    {t("git.viewDiff")}
                  </button>
                  <button type="button" class="icon-btn-text" onclick={viewHistoryAction} disabled={app.gitLoading}>
                    {t("git.commitHistory")}
                  </button>
                </div>
              </div>

              <div class="auto-sync-box">
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={app.gitAutoSyncInput} onchange={saveGitSettingsAction} />
                  <strong>{t("git.enableAutoSync")}</strong>
                </label>
                <p class="hint">{t("git.autoSyncDesc")}</p>
                {#if app.gitSettings?.last_sync_at}
                  <p class="hint">{t("git.lastSynced", { time: new Date(app.gitSettings.last_sync_at).toLocaleString() })}</p>
                {/if}
              </div>
            {/if}
          </div>
        {:else if app.gitActiveTab === "conflicts"}
          <div class="git-panel-section">
            <h4>{t("git.conflictDetection")}</h4>
            {#if !app.gitStatus?.has_conflicts || app.gitStatus.conflict_files.length === 0}
              <div class="clean-box">
                <p>{t("git.noConflicts")}</p>
              </div>
            {:else}
              <div class="conflict-alert-box">
                <p><strong>{t("git.conflictsDetectedTitle")}</strong> {t("git.conflictsDetectedDesc")}</p>
              </div>
              <div class="conflicts-list">
                {#each app.gitStatus.conflict_files as file}
                  <div class="conflict-item-card">
                    <div class="conflict-item-header">
                      <span class="conflict-filename">{@render iconFileText()} {file}</span>
                      <div class="conflict-choices">
                        <button
                          type="button"
                          class="btn-choice"
                          title={t("git.view3wayDiffTitle")}
                          onclick={() => loadConflictVersions(file)}
                        >
                          {app.selectedConflictFile === file && app.conflictVersions ? t("git.viewing3wayDiff") : t("git.view3wayDiff")}
                        </button>
                        <button
                          type="button"
                          class="btn-choice local"
                          title={t("git.keepLocalTitle")}
                          onclick={() => { resolveConflictAction(file, "ours"); if (app.selectedConflictFile === file) { app.selectedConflictFile = null; app.conflictVersions = null; } }}
                          disabled={app.gitLoading}
                        >
                          {t("git.keepLocal")}
                        </button>
                        <button
                          type="button"
                          class="btn-choice remote"
                          title={t("git.keepRemoteTitle")}
                          onclick={() => { resolveConflictAction(file, "theirs"); if (app.selectedConflictFile === file) { app.selectedConflictFile = null; app.conflictVersions = null; } }}
                          disabled={app.gitLoading}
                        >
                          {t("git.keepRemote")}
                        </button>
                      </div>
                    </div>
                    <p class="hint">{t("git.conflictDecideHint")}</p>

                    {#if app.selectedConflictFile === file}
                      <div class="conflict-3way">
                        {#if app.conflictVersionsLoading}
                          <p class="hint">{t("git.loadingVersions")}</p>
                        {:else if app.conflictVersions}
                          <div class="conflict-3way-col">
                            <span class="screen-kicker">{t("git.baseAncestor")}</span>
                            <pre class="body-view conflict-3way-pre">{app.conflictVersions.base ?? t("git.noCommonAncestor")}</pre>
                          </div>
                          <div class="conflict-3way-col">
                            <span class="screen-kicker">{t("git.localOurs")}</span>
                            <pre class="body-view conflict-3way-pre">{app.conflictVersions.local ?? t("git.absentLocally")}</pre>
                          </div>
                          <div class="conflict-3way-col">
                            <span class="screen-kicker">{t("git.remoteTheirs")}</span>
                            <pre class="body-view conflict-3way-pre">{app.conflictVersions.remote ?? t("git.absentRemote")}</pre>
                          </div>
                        {/if}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {:else if app.gitActiveTab === "github"}
          <div class="git-panel-section">
            <h4>{t("git.githubCollab")}</h4>
            <p class="hint">{t("git.githubTokenDesc")}</p>

            <div class="form-row-stacked">
              <label for="github-pat-input">{t("git.patLabel")}</label>
              <div class="input-with-actions">
                <input
                  id="github-pat-input"
                  type={app.githubShowToken ? "text" : "password"}
                  placeholder="ghp_..."
                  bind:value={app.githubTokenInput}
                />
                <button type="button" onclick={() => (app.githubShowToken = !app.githubShowToken)}>
                  {app.githubShowToken ? t("git.hide") : t("git.show")}
                </button>
                <button
                  type="button"
                  class="btn-primary"
                  disabled={!app.githubTokenInput.trim() || app.githubValidating}
                  onclick={() => { saveGitSettingsAction(); verifyGitHubTokenAction(true); }}
                >
                  {app.githubValidating ? t("git.verifying") : t("git.verifyToken")}
                </button>
              </div>
            </div>

            {#if app.githubUser}
              <div class="github-profile-card">
                {#if app.githubUser.avatar_url}
                  <img src={app.githubUser.avatar_url} alt={app.githubUser.login} class="github-avatar" />
                {/if}
                <div class="github-profile-info">
                  <strong>{app.githubUser.name ?? app.githubUser.login}</strong>
                  <span class="hint">@{app.githubUser.login}</span>
                  {#if app.githubUser.email}
                    <span class="hint">{app.githubUser.email}</span>
                  {/if}
                </div>
                <span class="badge badge-success">{t("git.authenticated")}</span>
              </div>
            {/if}

            <div class="form-row-stacked">
              <label for="git-remote-url-input">{t("git.remoteUrlLabel")}</label>
              <div class="input-with-actions">
                <input
                  id="git-remote-url-input"
                  type="text"
                  placeholder="https://github.com/owner/repository.git"
                  bind:value={app.gitRemoteUrlInput}
                />
                <button type="button" onclick={saveGitSettingsAction}>{t("git.saveRemote")}</button>
                <button
                  type="button"
                  disabled={!app.githubTokenInput.trim() || !app.gitRemoteUrlInput.trim() || app.githubValidating}
                  onclick={checkGitHubRepoAction}
                >
                  {t("git.checkPermissions")}
                </button>
              </div>
            </div>

            {#if app.githubRepoInfo}
              <div class="repo-permissions-card">
                <h5>{t("git.repository", { name: app.githubRepoInfo.full_name })}</h5>
                <div class="perm-badges">
                  <span class="perm-badge" class:perm-granted={app.githubRepoInfo.permissions?.pull}>
                    {t("git.readPull", { state: app.githubRepoInfo.permissions?.pull ? t("git.granted") : t("git.denied") })}
                  </span>
                  <span class="perm-badge" class:perm-granted={app.githubRepoInfo.permissions?.push}>
                    {t("git.writePush", { state: app.githubRepoInfo.permissions?.push ? t("git.granted") : t("git.denied") })}
                  </span>
                  <span class="perm-badge" class:perm-granted={app.githubRepoInfo.permissions?.admin}>
                    {t("git.admin", { state: app.githubRepoInfo.permissions?.admin ? t("git.granted") : t("git.denied") })}
                  </span>
                </div>
                <p class="hint">{t("git.defaultBranchLine", { branch: app.githubRepoInfo.default_branch, visibility: app.githubRepoInfo.private ? t("git.private") : t("git.public") })}</p>
              </div>
            {/if}
          </div>
        {:else if app.gitActiveTab === "projectfile"}
          <div class="git-panel-section">
            <h4>{t("git.canonicalFormat")}</h4>
            <p class="hint">{t("git.canonicalFormatDesc")}</p>
            {#if !app.selectedProjectId}
              <p class="hint">{t("git.selectProjectFirst")}</p>
            {/if}

            <div class="projectfile-options">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={app.projectFileMaskSecrets} />
                {t("git.maskSecrets")}
              </label>
              <button type="button" class="btn-primary" onclick={exportProjectFileAction} disabled={!app.selectedProjectId}>
                {t("git.generateJson")}
              </button>
            </div>

            {#if app.projectFileJson}
              <div class="json-preview-box">
                <div class="json-preview-toolbar">
                  <span>light-postman.json</span>
                  <div class="toolbar-actions">
                    <button type="button" onclick={downloadProjectFile}>{t("git.downloadFile")}</button>
                    <button type="button" onclick={importProjectFileAction}>{t("git.importIntoProject")}</button>
                  </div>
                </div>
                <textarea rows="12" bind:value={app.projectFileJson} class="code-area"></textarea>
                {#if app.projectFileStatus}
                  <p class="hint">{app.projectFileStatus}</p>
                {/if}
              </div>
            {/if}
          </div>
        {/if}
      </div>
  </section>
{/if}
