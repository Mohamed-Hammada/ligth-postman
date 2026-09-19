<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconCheckCircle, iconClose, iconLock, iconXCircle } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { AI_API_KEY_PLACEHOLDER, AI_MODEL_SUGGESTIONS, AI_PROVIDERS, addAiPreviewToProject, generateWithAi, importDiscoveredEndpointAction, onAiProviderChange, saveAiSettingsAction, scanSourceProjectAction, t, testAiConnectionAction } = app;
</script>

{#if app.showAiPanel}
  <div
    class="modal-backdrop"
    onclick={(e) => { if (e.target === e.currentTarget) app.showAiPanel = false; }}
    onkeydown={(e) => { if (e.key === "Escape") app.showAiPanel = false; }}
    role="dialog"
    aria-modal="true"
    tabindex="0"
  >
    <div class="modal-container modal-wide">
      <div class="modal-header">
        <div class="modal-title-wrap">
          <h3>{t("ai.title")}</h3>
          <span class="modal-sub">{t("ai.subtitle")}</span>
        </div>
        <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (app.showAiPanel = false)}>{@render iconClose()}</button>
      </div>

      <div class="modal-tabs">
        <button
          type="button"
          class="modal-tab-btn"
          class:active={app.aiActiveTab === "generate"}
          onclick={() => (app.aiActiveTab = "generate")}
        >
          {t("ai.tabGenerate")}
        </button>
        <button
          type="button"
          class="modal-tab-btn"
          class:active={app.aiActiveTab === "source"}
          onclick={() => (app.aiActiveTab = "source")}
        >
          {t("ai.tabSource")} {#if app.sourceReport}<span class="badge badge-framework">{app.sourceReport.endpoints.length}</span>{/if}
        </button>
        <button
          type="button"
          class="modal-tab-btn"
          class:active={app.aiActiveTab === "settings"}
          onclick={() => (app.aiActiveTab = "settings")}
        >
          {t("ai.tabSettings")} {#if !app.aiConfigured}<span class="tab-badge-alert">!</span>{/if}
        </button>
      </div>

      {#if app.aiActiveTab === "generate"}
        <div class="modal-body">
          {#if !app.aiConfigured}
            <div class="action-alert warning">
              <span>{t("ai.notConfiguredWarning")}</span>
              <button type="button" class="btn-primary btn-xs" onclick={() => (app.aiActiveTab = "settings")}>{t("ai.configureAi")}</button>
            </div>
          {/if}

          <div class="ai-context-options">
            <label class="checkbox-label" title={t("ai.includeExistingTitle")}>
              <input type="checkbox" bind:checked={app.aiIncludeExistingRequests} />
              {t("ai.includeExisting")}
            </label>
            <label class="checkbox-label" title={t("ai.includeVarsTitle")}>
              <input type="checkbox" bind:checked={app.aiIncludeVariables} />
              {t("ai.includeVars")}
            </label>
          </div>

          <form onsubmit={generateWithAi} class="ai-prompt-form">
            <textarea
              placeholder={t("ai.promptPlaceholder")}
              bind:value={app.aiPrompt}
              class="body-input"
              rows="3"
            ></textarea>
            <div class="params-row">
              <button type="submit" class="btn-primary" disabled={app.aiGenerating || !app.aiPrompt.trim()}>
                {app.aiGenerating ? t("ai.generatingDefinition") : t("ai.generateRequest")}
              </button>
            </div>
          </form>

          {#if app.aiPreview}
            <div class="ai-preview-card">
              <div class="preview-title-row">
                <span class="method method-{app.aiPreview.method.toLowerCase()}">{app.aiPreview.method}</span>
                <span class="preview-name">{app.aiPreview.name}</span>
                <code class="preview-url">{app.aiPreview.url}</code>
              </div>
              {#if app.aiPreview.description}
                <p class="hint preview-desc">{app.aiPreview.description}</p>
              {/if}
              <div class="preview-meta-row">
                <span>{t("ai.headersCount", { count: app.aiPreview.headers.length })}</span>
                <span>{t("ai.queryParamsCount", { count: app.aiPreview.query_params.length })}</span>
                {#if app.aiPreview.body}<span>{t("ai.hasBody")}</span>{/if}
              </div>
              {#if app.aiPreview.body}
                <pre class="body-view preview-body-pre">{app.aiPreview.body}</pre>
              {/if}
              <div class="params-row">
                <button
                  type="button"
                  class="btn-primary"
                  onclick={() => { addAiPreviewToProject(); app.showAiPanel = false; }}
                >
                  {t("ai.addToProject")}
                </button>
                <button type="button" onclick={() => (app.aiPreview = null)}>{t("ai.discard")}</button>
              </div>
            </div>
          {/if}
        </div>

      {:else if app.aiActiveTab === "source"}
        <div class="modal-body">
          <p class="hint">
            {t("ai.sourceDesc")} <em>{t("ai.sourceReadOnly")}</em>
          </p>

          <div class="source-scan-bar">
            <input
              type="text"
              placeholder={t("ai.sourcePathPlaceholder")}
              bind:value={app.sourceDirectoryInput}
              class="url-input"
            />
            <button
              type="button"
              class="btn-primary"
              disabled={app.sourceScanning || !app.sourceDirectoryInput.trim()}
              onclick={scanSourceProjectAction}
            >
              {app.sourceScanning ? t("ai.scanning") : t("ai.scanCodebase")}
            </button>
          </div>

          {#if app.sourceActionFeedback}
            <p class="action-feedback-inline">{app.sourceActionFeedback}</p>
          {/if}

          {#if app.sourceReport}
            <div class="source-summary-panel">
              <div class="source-badges-row">
                <span class="badge">{t("ai.typeLabel", { type: app.sourceReport.project_type })}</span>
                <span class="badge">{t("ai.scannedLabel", { count: app.sourceReport.scanned_files_count })}</span>
                <span class="badge badge-success">{t("ai.foundLabel", { count: app.sourceReport.endpoints.length })}</span>
                {#each app.sourceReport.frameworks as fw}
                  <span class="badge badge-framework">{fw}</span>
                {/each}
                {#if app.sourceReport.has_openapi}
                  <span class="badge badge-openapi">{t("ai.openapiDetected", { path: app.sourceReport.openapi_path ?? t("ai.specDetected") })}</span>
                {/if}
              </div>

              {#if app.sourceReport.endpoints.length > 0}
                <div class="source-filter-row">
                  <input
                    type="text"
                    placeholder={t("ai.filterRoutesPlaceholder")}
                    bind:value={app.sourceFilter}
                    class="url-input"
                  />
                </div>

                <div class="discovered-endpoints-list">
                  {#each app.filteredSourceEndpoints as ep, idx (idx)}
                    <div class="discovered-endpoint-card">
                      <div class="ep-info">
                        <span class="method method-{ep.method.toLowerCase()}">{ep.method}</span>
                        <code class="ep-path">{ep.path}</code>
                        {#if ep.auth_hint}
                          <span class="badge badge-auth" title={t("ai.authDetectedTitle")}>{@render iconLock()} {ep.auth_hint}</span>
                        {/if}
                      </div>
                      <div class="ep-meta">
                        <span class="ep-file">{ep.source_file}{ep.line_number ? `:${ep.line_number}` : ''}</span>
                        {#if ep.description}
                          <span class="ep-summary">{ep.description}</span>
                        {/if}
                      </div>
                      <button
                        type="button"
                        class="btn-xs-primary"
                        disabled={!app.selectedProjectId}
                        onclick={() => { importDiscoveredEndpointAction(ep); app.showAiPanel = false; }}
                        title={t("ai.importEndpointTitle")}
                      >
                        {t("ai.importRequest")}
                      </button>
                    </div>
                  {/each}
                  {#if app.filteredSourceEndpoints.length === 0}
                    <p class="hint text-center">{t("ai.noRoutesMatch", { filter: app.sourceFilter })}</p>
                  {/if}
                </div>
              {:else}
                <p class="hint">{t("ai.noRoutesFound")}</p>
              {/if}
            </div>
          {/if}
        </div>

      {:else if app.aiActiveTab === "settings"}
        <div class="modal-body">
          <div class="ai-settings-grid">
            <div class="settings-field">
              <label for="ai-provider-select"><strong>{t("ai.providerLabel")}</strong></label>
              <select id="ai-provider-select" bind:value={app.aiProviderInput} onchange={onAiProviderChange} class="url-input">
                {#each AI_PROVIDERS as p (p.id)}
                  <option value={p.id}>{t(p.labelKey)}</option>
                {/each}
              </select>
              <span class="hint">{t("ai.providerHint")}</span>
            </div>

            <div class="settings-field">
              <label for="ai-api-key-input">
                <strong>{t("ai.apiKeyLabel")}</strong>
                {#if app.aiConfigured}
                  <span class="badge badge-success">{t("ai.configured")}</span>
                {:else}
                  <span class="badge">{t("ai.notConfigured")}</span>
                {/if}
              </label>
              <div class="password-input-row">
                <input
                  id="ai-api-key-input"
                  type={app.aiShowKey ? "text" : "password"}
                  placeholder={app.aiSettings?.api_key || AI_API_KEY_PLACEHOLDER[app.aiProviderInput]}
                  bind:value={app.aiApiKeyInput}
                  class="url-input"
                />
                <button type="button" class="btn-ghost" onclick={() => (app.aiShowKey = !app.aiShowKey)}>
                  {app.aiShowKey ? t("ai.hide") : t("ai.show")}
                </button>
              </div>
              <span class="hint">{t("ai.apiKeyHint")}</span>
            </div>

            <div class="settings-field">
              <label for="ai-model-input"><strong>{t("ai.modelLabel")}</strong></label>
              <input
                id="ai-model-input"
                type="text"
                list="ai-model-suggestions"
                placeholder={t("ai.modelPlaceholder")}
                bind:value={app.aiModelInput}
                class="url-input"
              />
              <datalist id="ai-model-suggestions">
                {#each AI_MODEL_SUGGESTIONS[app.aiProviderInput] as m (m)}
                  <option value={m}></option>
                {/each}
              </datalist>
              <span class="hint">{t("ai.modelHint")}</span>
            </div>

            <div class="settings-field">
              <label for="ai-base-url-input">
                <strong>{t("ai.baseUrlLabel")}</strong>
                {#if app.aiProviderInput === "custom"}
                  <span class="badge badge-warn">{t("ai.required")}</span>
                {/if}
              </label>
              <input
                id="ai-base-url-input"
                type="text"
                placeholder={app.aiProviderInput === "custom" ? t("ai.baseUrlPlaceholderCustom") : t("ai.baseUrlPlaceholder")}
                bind:value={app.aiBaseUrlInput}
                class="url-input"
              />
              <span class="hint">{app.aiProviderInput === "custom" ? t("ai.baseUrlHintCustom") : t("ai.baseUrlHint")}</span>
            </div>

            <div class="params-row">
              <button type="button" class="btn-primary" onclick={saveAiSettingsAction}>{t("ai.saveSettings")}</button>
              <button type="button" class="btn-secondary" disabled={app.aiTesting} onclick={testAiConnectionAction}>
                {app.aiTesting ? t("ai.testing") : t("ai.testConnection")}
              </button>
            </div>

            {#if app.aiSettingsFeedback}
              <p class="action-feedback-inline text-success">{app.aiSettingsFeedback}</p>
            {/if}
            {#if app.aiTestFeedback}
              <div class="action-alert success">
                <span>{@render iconCheckCircle()} {app.aiTestFeedback}</span>
              </div>
            {/if}
            {#if app.aiTestError}
              <div class="action-alert error">
                <span>{@render iconXCircle()} {app.aiTestError}</span>
              </div>
            {/if}
          </div>
        </div>
      {/if}

      <div class="modal-footer">
        <button type="button" onclick={() => (app.showAiPanel = false)}>{t("common.close")}</button>
      </div>
    </div>
  </div>
{/if}
