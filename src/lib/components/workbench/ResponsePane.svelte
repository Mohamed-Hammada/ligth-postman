<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconCheck, iconChevronUp, iconClose, iconCompressDiagonal, iconCopy, iconExpandDiagonal, iconImport, iconInboxEmpty, iconMinus, iconSave } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { copyResponseBody, downloadResponseBody, formatByteSize, openHistoryResponse, saveCurrentResponse, setResponsePaneCollapsed, startResponsePaneResize, t } = app;
</script>

{#if !app.responsePaneCollapsed && !app.responsePaneMaximized}
    <div
      class="response-pane-resize-handle"
      class:resizing={app.responsePaneResizing}
      onmousedown={startResponsePaneResize}
      ondblclick={() => { app.responsePaneHeight = 360; }}
      onkeydown={(e) => {
        app.responsePaneManuallyResized = true;
        if (e.key === "ArrowUp") app.responsePaneHeight = Math.min(window.innerHeight - 220, app.responsePaneHeight + 16);
        else if (e.key === "ArrowDown") app.responsePaneHeight = Math.max(160, app.responsePaneHeight - 16);
      }}
      role="slider"
      aria-orientation="horizontal"
      aria-label={t("response.resizeHandle")}
      aria-valuenow={app.responsePaneHeight}
      aria-valuemin={100}
      aria-valuemax={1200}
      tabindex="0"
    ></div>
  {/if}

  <div
    class="response-pane"
    class:collapsed={app.responsePaneCollapsed}
    class:maximized={app.responsePaneMaximized}
    class:is-resizing={app.responsePaneResizing}
    style="height: {app.responsePaneMaximized ? '100%' : app.responsePaneCollapsed ? '37px' : app.responsePaneHeight + 'px'}"
  >
  {#if app.sending}
    <div class="response-loading">
      <span class="spinner" aria-hidden="true"></span>
      {t("response.sending")}
    </div>
  {:else if app.activeResponse}
    <div class="response">
      <div class="response-stat-row">
        <span class="response-stat-status" class:status-ok={app.activeResponse.status < 400} class:status-err={app.activeResponse.status >= 400}>
          {app.activeResponse.status} {app.activeResponse.status_text}
        </span>
        <span class="response-stat-item"><span class="response-stat-label">{t("response.time")}</span> {app.activeResponse.duration_ms} ms</span>
        <span class="response-stat-item"><span class="response-stat-label">{t("response.size")}</span> {formatByteSize(app.activeResponse.body_size)}</span>
        <div class="response-stat-spacer"></div>
        {#if app.copyFeedback}
          <span class="hint">{app.copyFeedback}</span>
        {/if}
        <button type="button" class="icon-btn" title={t("response.copyTitle")} onclick={copyResponseBody}>{@render iconCopy()}</button>
        <button type="button" class="icon-btn" title={t("response.downloadTitle")} onclick={downloadResponseBody}>{@render iconImport()}</button>
        <button type="button" class="icon-btn" title={t("response.saveAsSample")} onclick={saveCurrentResponse}>{@render iconSave()}</button>
        <button
          type="button"
          class="icon-btn"
          title={app.responsePaneCollapsed ? "Expand response panel" : "Collapse / Minimize response panel"}
          onclick={() => setResponsePaneCollapsed(!app.responsePaneCollapsed)}
        >{#if app.responsePaneCollapsed}{@render iconChevronUp()}{:else}{@render iconMinus()}{/if}</button>
        <button
          type="button"
          class="icon-btn"
          title={app.responsePaneMaximized ? "Restore response size" : "Maximize response panel"}
          onclick={() => { app.responsePaneMaximized = !app.responsePaneMaximized; if (app.responsePaneMaximized) app.responsePaneCollapsed = false; }}
        >{#if app.responsePaneMaximized}{@render iconCompressDiagonal()}{:else}{@render iconExpandDiagonal()}{/if}</button>
      </div>
      {#if !app.responsePaneCollapsed}

      <div class="response-subtabs">
        <button type="button" class="response-subtab" class:active={app.responseSubTab === "body"} onclick={() => (app.responseSubTab = "body")}>{t("response.body")}</button>
        <button type="button" class="response-subtab" class:active={app.responseSubTab === "headers"} onclick={() => (app.responseSubTab = "headers")}>
          {t("response.headers")}
          {#if app.activeResponse.headers?.length}<span class="tab-badge">{app.activeResponse.headers.length}</span>{/if}
        </button>
        <button type="button" class="response-subtab" class:active={app.responseSubTab === "cookies"} onclick={() => (app.responseSubTab = "cookies")}>
          {t("response.cookies")}
          {#if app.activeResponse.cookies?.length}<span class="tab-badge">{app.activeResponse.cookies.length}</span>{/if}
        </button>
        <button type="button" class="response-subtab" class:active={app.responseSubTab === "tests"} onclick={() => (app.responseSubTab = "tests")}>
          {t("response.tests")}
          {#if app.activeResponseTests.length}
            <span class="tab-badge" class:tab-badge-warn={app.activeResponseTests.some((test) => !test.passed)}>
              {app.activeResponseTests.filter((test) => test.passed).length}/{app.activeResponseTests.length}
            </span>
          {/if}
        </button>
        <button type="button" class="response-subtab" class:active={app.responseSubTab === "history"} onclick={() => (app.responseSubTab = "history")}>
          {t("response.history")}
          {#if app.responseHistory.length}<span class="tab-badge">{app.responseHistory.length}</span>{/if}
        </button>

        {#if app.responseSubTab === "body"}
          <div class="response-format-toggle">
            <button type="button" class="btn-toggle" class:active={app.responseViewMode === "pretty"} onclick={() => (app.responseViewMode = "pretty")}>{t("response.pretty")}</button>
            <button type="button" class="btn-toggle" class:active={app.responseViewMode === "raw"} onclick={() => (app.responseViewMode = "raw")}>{t("response.raw")}</button>
            {#if app.responseBodyIsHtml}
              <button type="button" class="btn-toggle" class:active={app.responseViewMode === "preview"} onclick={() => (app.responseViewMode = "preview")}>{t("response.preview")}</button>
            {/if}
          </div>
        {/if}
      </div>

      <div class="response-subtab-content">
        {#if app.responseSubTab === "body"}
          {#if app.responseViewMode === "preview" && app.responseBodyIsHtml}
            <iframe class="response-preview-frame" title={t("response.preview")} sandbox="" srcdoc={app.activeResponseBody}></iframe>
          {:else if app.responseBodyIsJson}
            <div class="code-editor-shell response-code-shell">
              <div class="code-gutter" aria-hidden="true">
                {#each (app.prettyResponseBody || "").split("\n") as _, lineIdx (lineIdx)}
                  <span class="gutter-num">{lineIdx + 1}</span>
                {/each}
              </div>
              <pre class="body-view code-editor-pre">{@html app.highlightedResponseBody}</pre>
            </div>
          {:else}
            <div class="code-editor-shell response-code-shell">
              <div class="code-gutter" aria-hidden="true">
                {#each (app.prettyResponseBody || "").split("\n") as _, lineIdx (lineIdx)}
                  <span class="gutter-num">{lineIdx + 1}</span>
                {/each}
              </div>
              <pre class="body-view code-editor-pre">{app.prettyResponseBody}</pre>
            </div>
          {/if}
          {#if app.activeResponseTruncated}
            <p class="hint">{t("response.truncated")}</p>
          {/if}
        {:else if app.responseSubTab === "headers"}
          {#if app.activeResponse.headers?.length}
            <div class="headers-list">
              {#each app.activeResponse.headers as h}
                <div class="header-line">
                  <strong>{h.key}:</strong> {h.value}
                </div>
              {/each}
            </div>
          {:else}
            <p class="empty">{t("response.noHeaders")}</p>
          {/if}
        {:else if app.responseSubTab === "cookies"}
          {#if app.activeResponse.cookies?.length}
            <div class="headers-list">
              {#each app.activeResponse.cookies as c}
                <div class="header-line">
                  <strong>{c.name}:</strong> {c.value}
                  {#if c.domain}<span class="hint">{t("cookie.domain", { value: c.domain })}</span>{/if}
                  {#if c.path}<span class="hint">{t("cookie.path", { value: c.path })}</span>{/if}
                  {#if c.http_only}<span class="badge">{t("cookie.httpOnly")}</span>{/if}
                  {#if c.secure}<span class="badge">{t("cookie.secure")}</span>{/if}
                </div>
              {/each}
            </div>
          {:else}
            <p class="empty">{t("response.noCookies")}</p>
          {/if}
        {:else if app.responseSubTab === "tests"}
          {#if app.activeResponseTests.length}
            <ul class="test-results-list">
              {#each app.activeResponseTests as test, i (i)}
                <li class="test-result-row" class:test-pass={test.passed} class:test-fail={!test.passed}>
                  <span class="test-result-icon">{#if test.passed}{@render iconCheck()}{:else}{@render iconClose()}{/if}</span>
                  <span class="test-result-name">{test.name}</span>
                  {#if !test.passed && test.error}<span class="test-result-error">{test.error}</span>{/if}
                </li>
              {/each}
            </ul>
          {:else}
            <p class="empty">{t("response.testsHintFull")}</p>
          {/if}
        {:else if app.responseSubTab === "history"}
          {#if app.responseHistory.length}
            <ul class="response-history-list">
              {#each app.responseHistory as r (r.id)}
                <li>
                  <button type="button" class="response-history-row" onclick={() => openHistoryResponse(r.id)}>
                    <span class="status-chip" class:status-ok={r.status < 400} class:status-err={r.status >= 400}>{r.status}</span>
                    <span class="response-history-duration">{r.duration_ms} ms</span>
                    <span class="response-history-time">{new Date(r.created_at).toLocaleString()}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {:else}
            <p class="empty">{t("history.empty")}</p>
          {/if}
        {/if}
      </div>
    {/if}
  </div>
{:else}
  <div class="response-pane-bar">
    <button
      type="button"
      class="icon-btn"
      title={app.responsePaneCollapsed ? t("response.expandPane") : t("response.collapsePane")}
      onclick={() => setResponsePaneCollapsed(!app.responsePaneCollapsed)}
    >{#if app.responsePaneCollapsed}{@render iconChevronUp()}{:else}{@render iconMinus()}{/if}</button>
    <span class="response-pane-bar-label">
      {t("response.title")}
    </span>
    <div class="response-stat-spacer"></div>
    <button
      type="button"
      class="icon-btn"
      title={app.responsePaneMaximized ? "Restore response size" : "Maximize response panel"}
      onclick={() => { app.responsePaneMaximized = !app.responsePaneMaximized; if (app.responsePaneMaximized) app.responsePaneCollapsed = false; }}
    >{#if app.responsePaneMaximized}{@render iconCompressDiagonal()}{:else}{@render iconExpandDiagonal()}{/if}</button>
  </div>
  {#if !app.responsePaneCollapsed}
    <div class="response-empty-state">
      <div class="empty-icon">{@render iconInboxEmpty()}</div>
      <p>{t("response.sendEmpty")}</p>
    </div>
  {/if}
{/if}
  </div>
