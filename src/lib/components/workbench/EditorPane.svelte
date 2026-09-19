<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconCheck, iconClose, iconEdit, iconTrash } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { RAW_CONTENT_TYPES, deleteSampleResponseAction, focusOnMount, generateSampleResponseWithAiAction, generateTestsAndDocsWithAiAction, growFormDataItems, growHeaders, growQueryParams, growUrlEncodedItems, handleAutocompleteKeydown, handleBodyMouseLeave, handleBodyMouseMove, handlePaneMouseMoveDelegated, hideAutocompleteSoon, prettifyBody, removeFormDataItem, removeHeader, removeQueryParam, removeUrlEncodedItem, scheduleAutoSave, scheduleHideMissingVarPopover, setRawContentType, startRenameSampleResponse, submitRenameSampleResponse, t, updateAutocompleteFor } = app;
</script>

<div class="editor-pane" onmousemove={handlePaneMouseMoveDelegated} onmouseleave={() => scheduleHideMissingVarPopover()}>
<div class="tab-content">
  {#if app.activeEditorTab === "params"}
    <div class="params-table">
      {#each app.editQueryParams as param, i (i)}
        <div class="params-row">
          <input type="checkbox" bind:checked={param.enabled} title={t("params.enabled")} onchange={scheduleAutoSave} />
          <input placeholder={t("params.key")} bind:value={param.key} oninput={() => { growQueryParams(); scheduleAutoSave(); }} />
          <input placeholder={t("params.value")} bind:value={param.value} oninput={() => { growQueryParams(); scheduleAutoSave(); }} />
          {#if i < app.editQueryParams.length - 1 || param.key.trim()}
            <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeQueryParam(i); scheduleAutoSave(); }}>{@render iconTrash()}</button>
          {/if}
        </div>
      {/each}
    </div>

  {:else if app.activeEditorTab === "headers"}
    <div class="params-table">
      {#each app.editHeaders as header, i (i)}
        <div class="params-row">
          <input type="checkbox" bind:checked={header.enabled} title={t("params.enabled")} onchange={scheduleAutoSave} />
          <input
            placeholder={t("params.key")}
            bind:value={header.key}
            oninput={(e) => {
              growHeaders();
              scheduleAutoSave();
              updateAutocompleteFor(e.currentTarget as HTMLInputElement, "header", false, (v) => (header.key = v));
            }}
            onkeydown={handleAutocompleteKeydown}
            onblur={hideAutocompleteSoon}
          />
          <input
            placeholder={t("params.value")}
            bind:value={header.value}
            oninput={(e) => {
              growHeaders();
              scheduleAutoSave();
              updateAutocompleteFor(e.currentTarget as HTMLInputElement, "var", false, (v) => (header.value = v));
            }}
            onkeydown={handleAutocompleteKeydown}
            onblur={hideAutocompleteSoon}
          />
          <input placeholder={t("headers.description")} bind:value={header.description} oninput={() => { growHeaders(); scheduleAutoSave(); }} />
          {#if i < app.editHeaders.length - 1 || header.key.trim()}
            <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeHeader(i); scheduleAutoSave(); }}>{@render iconTrash()}</button>
          {/if}
        </div>
      {/each}
    </div>

  {:else if app.activeEditorTab === "auth"}
    <div class="params-table">
      <select bind:value={app.editAuthType} onchange={scheduleAutoSave}>
        <option value="none">{t("auth.none")}</option>
        <option value="bearer">{t("auth.bearer")}</option>
        <option value="basic">{t("auth.basic")}</option>
        <option value="api_key">{t("auth.apiKey")}</option>
      </select>
      {#if app.editAuthType === "bearer"}
        <div class="params-row">
          <input placeholder={t("auth.token")} bind:value={app.editAuthBearerToken} oninput={scheduleAutoSave} />
        </div>
      {:else if app.editAuthType === "basic"}
        <div class="params-row">
          <input placeholder={t("auth.username")} bind:value={app.editAuthBasicUsername} oninput={scheduleAutoSave} />
          <input placeholder={t("auth.password")} type="password" bind:value={app.editAuthBasicPassword} oninput={scheduleAutoSave} />
        </div>
      {:else if app.editAuthType === "api_key"}
        <div class="params-row">
          <input placeholder={t("params.key")} bind:value={app.editAuthApiKeyKey} oninput={scheduleAutoSave} />
          <input placeholder={t("params.value")} bind:value={app.editAuthApiKeyValue} oninput={scheduleAutoSave} />
          <select bind:value={app.editAuthApiKeyLocation} onchange={scheduleAutoSave}>
            <option value="header">{t("auth.locationHeader")}</option>
            <option value="query">{t("auth.locationQuery")}</option>
          </select>
        </div>
      {/if}
    </div>

  {:else if app.activeEditorTab === "body"}
    <div class="params-table">
      <div class="body-mode-bar">
        <label class="radio-label">
          <input type="radio" bind:group={app.editBodyType} value="raw" onchange={scheduleAutoSave} /> {t("body.raw")}
        </label>
        <label class="radio-label">
          <input type="radio" bind:group={app.editBodyType} value="form-data" onchange={scheduleAutoSave} /> {t("body.formData")}
        </label>
        <label class="radio-label">
          <input type="radio" bind:group={app.editBodyType} value="x-www-form-urlencoded" onchange={scheduleAutoSave} /> {t("body.urlEncoded")}
        </label>
        <label class="radio-label">
          <input type="radio" bind:group={app.editBodyType} value="binary" onchange={scheduleAutoSave} /> {t("body.binary")}
        </label>
        <label class="radio-label">
          <input type="radio" bind:group={app.editBodyType} value="graphql" onchange={scheduleAutoSave} /> {t("body.graphql")}
        </label>
        {#if app.editBodyType === "raw"}
          <select class="raw-type-select" value={app.rawContentType} onchange={(e) => setRawContentType((e.target as HTMLSelectElement).value)}>
            {#each RAW_CONTENT_TYPES as type (type.id)}
              <option value={type.id}>{t(type.label)}</option>
            {/each}
          </select>
          {#if app.rawContentType === "json"}
            <button type="button" onclick={() => { if (!app.editBody) app.editBody = "{\n  \n}"; scheduleAutoSave(); }}>{t("body.jsonTemplate")}</button>
          {/if}
          <button type="button" class="btn-danger-ghost" onclick={() => { app.editBody = ""; scheduleAutoSave(); }}>{t("body.clearBody")}</button>
          {#if app.editBody.trim()}
            <button type="button" onclick={prettifyBody} title={t("body.prettifyTitle")}>{t("body.prettify")}</button>
          {/if}
          {#if app.bodyPrettifyFeedback}<span class="warn-inline">{app.bodyPrettifyFeedback}</span>{/if}
        {/if}
      </div>

      {#if app.editBodyType === "raw"}
        <div class="code-editor-shell">
          <div class="code-gutter" aria-hidden="true">
            {#each (app.editBody || "").split("\n") as _, lineIdx (lineIdx)}
              <span class="gutter-num">{lineIdx + 1}</span>
            {/each}
          </div>
          <textarea
            placeholder={t("body.rawPlaceholder")}
            bind:value={app.editBody}
            class="body-input code-editor-input"
            rows="8"
            spellcheck="false"
            oninput={(e) => { scheduleAutoSave(); updateAutocompleteFor(e.currentTarget as HTMLTextAreaElement, "var", false, (v) => (app.editBody = v)); }}
            onkeydown={handleAutocompleteKeydown}
            onblur={hideAutocompleteSoon}
            onmousemove={handleBodyMouseMove}
            onmouseleave={handleBodyMouseLeave}
          ></textarea>
        </div>
      {:else if app.editBodyType === "form-data"}
        <div class="params-table">
          {#each app.editFormDataItems as item, i (i)}
            <div class="params-row">
              <input type="checkbox" bind:checked={item.enabled} title={t("params.enabled")} onchange={scheduleAutoSave} />
              <input placeholder={t("params.key")} bind:value={item.key} oninput={() => { growFormDataItems(); scheduleAutoSave(); }} />
              {#if item.is_file}
                <input placeholder={t("body.filePath")} bind:value={item.file_path} oninput={() => { growFormDataItems(); scheduleAutoSave(); }} />
              {:else}
                <input placeholder={t("params.value")} bind:value={item.value} oninput={() => { growFormDataItems(); scheduleAutoSave(); }} />
              {/if}
              <label class="checkbox-label" title={t("body.fileHint")}>
                <input type="checkbox" bind:checked={item.is_file} onchange={scheduleAutoSave} /> {t("body.file")}
              </label>
              {#if i < app.editFormDataItems.length - 1 || item.key.trim()}
                <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeFormDataItem(i); scheduleAutoSave(); }}>{@render iconTrash()}</button>
              {/if}
            </div>
          {/each}
        </div>
      {:else if app.editBodyType === "x-www-form-urlencoded"}
        <div class="params-table">
          {#each app.editUrlEncodedItems as item, i (i)}
            <div class="params-row">
              <input type="checkbox" bind:checked={item.enabled} title={t("params.enabled")} onchange={scheduleAutoSave} />
              <input placeholder={t("params.key")} bind:value={item.key} oninput={() => { growUrlEncodedItems(); scheduleAutoSave(); }} />
              <input placeholder={t("params.value")} bind:value={item.value} oninput={() => { growUrlEncodedItems(); scheduleAutoSave(); }} />
              {#if i < app.editUrlEncodedItems.length - 1 || item.key.trim()}
                <button type="button" class="icon-btn" title={t("params.remove")} onclick={() => { removeUrlEncodedItem(i); scheduleAutoSave(); }}>{@render iconTrash()}</button>
              {/if}
            </div>
          {/each}
        </div>
      {:else if app.editBodyType === "binary"}
        <div class="params-table">
          <div class="params-row">
            <input placeholder={t("body.binaryPathPlaceholder")} bind:value={app.editBinaryFilePath} oninput={scheduleAutoSave} />
          </div>
          <p class="hint">{t("body.binaryHint")}</p>
        </div>
      {:else if app.editBodyType === "graphql"}
        <div class="graphql-editor">
          <h4>{t("body.graphqlQuery")}</h4>
          <textarea
            placeholder={t("body.graphqlQueryPlaceholder")}
            bind:value={app.editGraphqlQuery}
            class="body-input"
            rows="6"
            oninput={() => {
              try {
                const vars = app.editGraphqlVariables ? JSON.parse(app.editGraphqlVariables) : {};
                app.editBody = JSON.stringify({ query: app.editGraphqlQuery, variables: vars }, null, 2);
              } catch {
                app.editBody = JSON.stringify({ query: app.editGraphqlQuery }, null, 2);
              }
              scheduleAutoSave();
            }}
          ></textarea>
          <h4>{t("body.graphqlVariables")}</h4>
          <textarea
            placeholder={t("body.graphqlVariablesPlaceholder")}
            bind:value={app.editGraphqlVariables}
            class="body-input"
            rows="3"
            oninput={() => {
              try {
                const vars = app.editGraphqlVariables ? JSON.parse(app.editGraphqlVariables) : {};
                app.editBody = JSON.stringify({ query: app.editGraphqlQuery, variables: vars }, null, 2);
              } catch {
                app.editBody = JSON.stringify({ query: app.editGraphqlQuery }, null, 2);
              }
              scheduleAutoSave();
            }}
          ></textarea>
        </div>
      {/if}
    </div>

  {:else if app.activeEditorTab === "scripts"}
    <div class="scripts-layout">
      <div class="scripts-side">
        <button type="button" class="scripts-side-item" class:active={app.activeScriptTab === "pre"} onclick={() => (app.activeScriptTab = "pre")}>
          {t("scripts.pre")} {#if app.editPreScript}<span class="tab-dot">•</span>{/if}
        </button>
        <button type="button" class="scripts-side-item" class:active={app.activeScriptTab === "post"} onclick={() => (app.activeScriptTab = "post")}>
          {t("scripts.post")} {#if app.editPostScript}<span class="tab-dot">•</span>{/if}
        </button>
      </div>
      <div class="scripts-main">
        {#if app.activeScriptTab === "pre"}
          <p class="hint">{t("scripts.preHint")}</p>
          <textarea
            placeholder={t("scripts.prePlaceholder")}
            bind:value={app.editPreScript}
            class="body-input scripts-textarea"
            oninput={(e) => { scheduleAutoSave(); updateAutocompleteFor(e.currentTarget as HTMLTextAreaElement, "pm", false, (v) => (app.editPreScript = v)); }}
            onkeydown={handleAutocompleteKeydown}
            onblur={hideAutocompleteSoon}
          ></textarea>
        {:else}
          <div class="field-header-row">
            <p class="hint">{t("scripts.postHint")}</p>
            <button
              type="button"
              class="btn-ghost btn-xs"
              disabled={app.generatingTestsDocs}
              onclick={() => generateTestsAndDocsWithAiAction("tests")}
              title={t("scripts.generateTestsTitle")}
            >
              {app.generatingTestsDocs ? t("scripts.generating") : t("scripts.generateTests")}
            </button>
          </div>
          {#if app.testsDocsFeedback}
            <p class="action-feedback-inline">{app.testsDocsFeedback}</p>
          {/if}
          <textarea
            placeholder={t("scripts.postPlaceholder")}
            bind:value={app.editPostScript}
            class="body-input scripts-textarea"
            oninput={(e) => { scheduleAutoSave(); updateAutocompleteFor(e.currentTarget as HTMLTextAreaElement, "pm", true, (v) => (app.editPostScript = v)); }}
            onkeydown={handleAutocompleteKeydown}
            onblur={hideAutocompleteSoon}
          ></textarea>
        {/if}
      </div>
    </div>

  {:else if app.activeEditorTab === "settings"}
    <div class="params-table settings-grid">
      <label class="settings-row">
        <span>{t("reqSettings.timeout")}</span>
        <input
          type="number"
          placeholder={t("reqSettings.timeoutPlaceholder")}
          value={app.editTimeoutMs ?? ""}
          oninput={(e) => {
            const val = (e.target as HTMLInputElement).value;
            app.editTimeoutMs = val ? parseInt(val, 10) : null;
            scheduleAutoSave();
          }}
        />
      </label>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={app.editFollowRedirects} onchange={scheduleAutoSave} />
        {t("reqSettings.followRedirects")}
      </label>
      <label class="settings-row">
        <span>{t("reqSettings.maxRedirects")}</span>
        <input type="number" bind:value={app.editMaxRedirects} min="0" max="50" oninput={scheduleAutoSave} />
      </label>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={app.editVerifySsl} onchange={scheduleAutoSave} />
        {t("reqSettings.verifySsl")}
      </label>
      <label class="settings-row">
        <span>{t("reqSettings.proxyUrl")}</span>
        <input placeholder="http://127.0.0.1:8080" bind:value={app.editProxyUrl} oninput={scheduleAutoSave} />
      </label>
      <label class="settings-row">
        <span>{t("reqSettings.httpVersion")}</span>
        <select bind:value={app.editHttpVersion} onchange={scheduleAutoSave}>
          <option value="">{t("reqSettings.httpVersionDefault")}</option>
          <option value="HTTP/1.1">HTTP/1.1</option>
          <option value="HTTP/2">HTTP/2</option>
        </select>
      </label>
    </div>

  {:else if app.activeEditorTab === "docs"}
    <div class="params-table">
      <div class="field-header-row">
        <h4>{t("docs.title")}</h4>
        <button
          type="button"
          class="btn-ghost btn-xs"
          disabled={app.generatingTestsDocs}
          onclick={() => generateTestsAndDocsWithAiAction("docs")}
          title={t("docs.generateTitle")}
        >
          {app.generatingTestsDocs ? t("scripts.generating") : t("docs.generate")}
        </button>
      </div>
      {#if app.testsDocsFeedback}
        <p class="action-feedback-inline">{app.testsDocsFeedback}</p>
      {/if}
      <textarea
        placeholder={t("docs.placeholder")}
        bind:value={app.editDescription}
        class="body-input"
        rows="8"
        oninput={scheduleAutoSave}
      ></textarea>
    </div>

  {:else if app.activeEditorTab === "mock"}
    <div class="sample-responses-section">
      <div class="field-header-row">
        <p class="hint">{t("sample.hint")}</p>
        <button
          type="button"
          class="btn-ghost btn-xs"
          disabled={app.generatingSample}
          onclick={generateSampleResponseWithAiAction}
          title={t("sample.generateTitle")}
        >
          {app.generatingSample ? t("scripts.generating") : t("sample.generate")}
        </button>
      </div>
      {#if app.sampleFeedback}
        <p class="action-feedback-inline">{app.sampleFeedback}</p>
      {/if}
      {#if app.sampleResponses.length}
        <div class="sample-responses-list">
          {#each app.sampleResponses as sr (sr.id)}
            {#if app.renamingSampleResponseId === sr.id}
              <div class="sample-response-card">
                <form class="inline-form" onsubmit={(e) => { e.preventDefault(); submitRenameSampleResponse(sr.request_id); }}>
                  <input bind:value={app.renameSampleResponseValue} use:focusOnMount onblur={() => submitRenameSampleResponse(sr.request_id)} />
                  <button type="submit" title={t("sidebar.save")}>{@render iconCheck()}</button>
                  <button type="button" title={t("sidebar.cancel")} onclick={() => (app.renamingSampleResponseId = null)}>{@render iconClose()}</button>
                </form>
              </div>
            {:else}
              <details class="sample-response-card">
                <summary class="sample-response-summary">
                  <span class="badge badge-sample">{t("sample.badge")}</span>
                  <strong class:status-ok={sr.status < 400} class:status-err={sr.status >= 400}>
                    {sr.status}
                  </strong>
                  <span class="sample-name">{sr.name}</span>
                  <span class="hint">{new Date(sr.created_at).toLocaleTimeString()}</span>
                  <button
                    type="button"
                    class="btn-delete-icon"
                    onclick={(e) => { e.stopPropagation(); startRenameSampleResponse(sr); }}
                    title={t("sidebar.rename")}
                  >{@render iconEdit()}</button>
                  <button
                    type="button"
                    class="btn-delete-icon"
                    onclick={(e) => { e.stopPropagation(); deleteSampleResponseAction(sr.request_id, sr.id); }}
                    title={t("sample.delete")}
                  >{@render iconClose()}</button>
                </summary>
                <pre class="body-view">{sr.body ?? ""}</pre>
              </details>
            {/if}
          {/each}
        </div>
      {:else}
        <p class="screen-empty-inline">{t("sample.empty")}</p>
      {/if}
    </div>
  {/if}
</div>
</div>
