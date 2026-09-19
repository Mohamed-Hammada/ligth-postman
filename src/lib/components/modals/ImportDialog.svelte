<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconClose } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { handleCollectionFileUpload, handleEnvironmentFileUpload, importCurlCommand, importLocalWorkspaceAction, importPostmanCollectionAction, importPostmanEnvironmentAction, t } = app;
</script>

{#if app.showImportDialog}
  <div
    class="modal-backdrop"
    onclick={(e) => { if (e.target === e.currentTarget) app.showImportDialog = false; }}
    onkeydown={(e) => { if (e.key === "Escape") app.showImportDialog = false; }}
    role="dialog"
    aria-modal="true"
    tabindex="0"
  >
    <div class="modal-container modal-wide">
      <div class="modal-header">
        <div class="modal-title-wrap">
          <h3>{t("import.title")}</h3>
          <span class="modal-sub">{t("import.subtitle")}</span>
        </div>
        <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (app.showImportDialog = false)}>{@render iconClose()}</button>
      </div>

      <div class="modal-tabs">
        <button type="button" class="modal-tab-btn" class:active={app.importActiveTab === "collection"} onclick={() => (app.importActiveTab = "collection")}>{t("import.tabCollection")}</button>
        <button type="button" class="modal-tab-btn" class:active={app.importActiveTab === "environment"} onclick={() => (app.importActiveTab = "environment")}>{t("import.tabEnvironment")}</button>
        <button type="button" class="modal-tab-btn" class:active={app.importActiveTab === "curl"} onclick={() => (app.importActiveTab = "curl")}>{t("import.tabCurl")}</button>
        <button type="button" class="modal-tab-btn" class:active={app.importActiveTab === "localWorkspace"} onclick={() => (app.importActiveTab = "localWorkspace")}>{t("import.tabLocalWorkspace")}</button>
      </div>

      <div class="modal-body">
        {#if app.importActiveTab === "collection"}
          <p class="hint">{t("import.collectionHint")}</p>
          <div class="file-dropzone">
            <label class="file-label">
              <span>{t("import.chooseJsonFile")}</span>
              <input type="file" accept=".json,application/json" onchange={handleCollectionFileUpload} />
            </label>
          </div>
          <textarea placeholder={t("import.pasteCollectionPlaceholder")} bind:value={app.collectionImportText} rows="6" class="body-input"></textarea>

          {#if app.selectedProjectId}
            <div class="radio-row">
              <label class="radio-label">
                <input type="radio" name="collectionTargetScreen" value="new" bind:group={app.collectionImportTarget} />
                {t("import.newProject")}
              </label>
              <label class="radio-label">
                <input type="radio" name="collectionTargetScreen" value="current" bind:group={app.collectionImportTarget} />
                {t("import.currentProject")}
              </label>
            </div>
          {/if}

          {#if app.collectionImportError}<p class="error">{app.collectionImportError}</p>{/if}

          {#if app.collectionImportReport}
            <div class="import-report-card">
              <h4>{t("import.complete")}</h4>
              <p>{t("import.project", { name: app.collectionImportReport.project_name })}</p>
              <p>{t("import.requests", { count: app.collectionImportReport.requests_count })}</p>
              <p>{t("import.variables", { count: app.collectionImportReport.variables_count })}</p>
              <p>{t("import.sampleResponses", { count: app.collectionImportReport.sample_responses_count })}</p>
              {#if app.collectionImportReport.warnings.length > 0}
                <div class="warnings-box">
                  <h5>{t("import.compatNotes")}</h5>
                  <ul>
                    {#each app.collectionImportReport.warnings as warn}<li>{warn}</li>{/each}
                  </ul>
                </div>
              {/if}
            </div>
          {/if}

          <div class="params-row">
            <button type="button" class="btn-primary" disabled={!app.collectionImportText.trim() || app.collectionImportLoading} onclick={importPostmanCollectionAction}>
              {app.collectionImportLoading ? t("import.importing") : t("import.importCollection")}
            </button>
          </div>
        {:else if app.importActiveTab === "environment"}
          <p class="hint">{t("import.environmentHint")}</p>
          <div class="file-dropzone">
            <label class="file-label">
              <span>{t("import.chooseJsonFile")}</span>
              <input type="file" accept=".json,application/json" onchange={handleEnvironmentFileUpload} />
            </label>
          </div>
          <textarea placeholder={t("import.pasteEnvironmentPlaceholder")} bind:value={app.environmentImportText} rows="4" class="body-input"></textarea>

          {#if app.environmentImportError}<p class="error">{app.environmentImportError}</p>{/if}

          {#if app.environmentImportReport}
            <div class="import-report-card">
              <h4>{t("import.environmentImported")}</h4>
              <p>{t("import.environment", { name: app.environmentImportReport.environment_name })}</p>
              <p>{t("import.variables", { count: app.environmentImportReport.variables_count })}</p>
              {#if app.environmentImportReport.warnings.length > 0}
                <div class="warnings-box">
                  <h5>{t("import.compatNotes")}</h5>
                  <ul>
                    {#each app.environmentImportReport.warnings as warn}<li>{warn}</li>{/each}
                  </ul>
                </div>
              {/if}
            </div>
          {/if}

          <div class="params-row">
            <button type="button" class="btn-primary" disabled={!app.environmentImportText.trim() || app.environmentImportLoading} onclick={importPostmanEnvironmentAction}>
              {app.environmentImportLoading ? t("import.importing") : t("import.importEnvironment")}
            </button>
          </div>
        {:else if app.importActiveTab === "curl"}
          <p class="hint">{t("import.curlHint")}</p>
          {#if !app.selectedProjectId}
            <p class="screen-empty-inline">{t("import.selectProjectFirst")}</p>
          {:else}
            <form onsubmit={importCurlCommand}>
              <input placeholder={t("import.requestNamePlaceholder")} bind:value={app.curlImportName} />
              <textarea
                placeholder={t("import.curlPlaceholder")}
                bind:value={app.curlImportText}
                rows="6"
                class="body-input"
              ></textarea>
              {#if app.curlImportError}<p class="error">{app.curlImportError}</p>{/if}
              <div class="params-row">
                <button type="submit" class="btn-primary">{t("import.importRequest")}</button>
              </div>
            </form>
          {/if}
        {:else if app.importActiveTab === "localWorkspace"}
          <p class="hint">{t("import.localWorkspaceHint")}</p>
          <input
            type="text"
            placeholder={t("import.localWorkspacePathPlaceholder")}
            bind:value={app.localWorkspacePathInput}
            class="url-input"
          />

          {#if app.localWorkspaceImportError}<p class="error">{app.localWorkspaceImportError}</p>{/if}

          {#if app.localWorkspaceImportReport}
            <div class="import-report-card">
              <h4>{t("import.complete")}</h4>
              <p>{t("import.localWorkspaceProjects", { count: app.localWorkspaceImportReport.projects_created })}</p>
              <p>{t("import.localWorkspaceFolders", { count: app.localWorkspaceImportReport.folders_created })}</p>
              <p>{t("import.requests", { count: app.localWorkspaceImportReport.requests_imported })}</p>
              <p>{t("import.localWorkspaceSamples", { count: app.localWorkspaceImportReport.samples_imported })}</p>
              <p>{t("import.localWorkspaceEnvironments", { count: app.localWorkspaceImportReport.environments_imported })}</p>
              <p>{t("import.variables", { count: app.localWorkspaceImportReport.variables_imported })}</p>
              {#if app.localWorkspaceImportReport.warnings.length > 0}
                <div class="warnings-box">
                  <h5>{t("import.compatNotes")}</h5>
                  <ul>
                    {#each app.localWorkspaceImportReport.warnings as warn}<li>{warn}</li>{/each}
                  </ul>
                </div>
              {/if}
            </div>
          {/if}

          <div class="params-row">
            <button
              type="button"
              class="btn-primary"
              disabled={!app.localWorkspacePathInput.trim() || app.localWorkspaceImportLoading}
              onclick={importLocalWorkspaceAction}
            >
              {app.localWorkspaceImportLoading ? t("import.importing") : t("import.importLocalWorkspace")}
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
