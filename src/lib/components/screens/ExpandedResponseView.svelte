<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconInboxEmpty } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { copyResponseBody, downloadResponseBody, formatByteSize, openHistoryResponse, t } = app;
</script>

<section class="screen-page">
  <div class="screen-page-header">
    <span class="screen-kicker">{t("response.title")}</span>
    <div class="screen-title-row">
      {#if app.selectedRequest}
        <h1 class="screen-title">{app.selectedRequest.method} {app.selectedRequest.name}</h1>
      {:else}
        <h1 class="screen-title">{t("response.noRequestOpen")}</h1>
      {/if}
      <button type="button" class="btn-ghost btn-xs" title={t("response.backToWorkspace")} onclick={() => (app.responseExpanded = false)}>{t("response.backToWorkspace")}</button>
    </div>
  </div>

  {#if !app.selectedRequest}
    <div class="screen-empty">
      <div class="empty-icon">{@render iconInboxEmpty()}</div>
      <p>{t("response.openFromWorkspace")}</p>
    </div>
  {:else if !app.activeResponse}
    <div class="screen-empty">
      <div class="empty-icon">{@render iconInboxEmpty()}</div>
      <p>{t("response.sendToSee")}</p>
    </div>
  {:else}
    <div class="response-screen-body">
      <aside class="response-screen-side">
        <div class="response-screen-stat-block">
          <span class="screen-kicker">{t("response.status")}</span>
          <div class="response-screen-status" class:status-ok={app.activeResponse.status < 400} class:status-err={app.activeResponse.status >= 400}>
            {app.activeResponse.status} {app.activeResponse.status_text}
          </div>
          <div class="response-screen-path">{app.selectedRequest.method} {app.selectedRequest.url}</div>
        </div>
        <div class="response-screen-metrics">
          <div class="response-screen-metric">
            <span class="screen-kicker">{t("response.time")}</span>
            <div class="response-screen-metric-value">{app.activeResponse.duration_ms} ms</div>
          </div>
          <div class="response-screen-metric">
            <span class="screen-kicker">{t("response.size")}</span>
            <div class="response-screen-metric-value">{formatByteSize(app.activeResponse.body_size)}</div>
          </div>
          <div class="response-screen-metric">
            <span class="screen-kicker">{t("response.heldInRam")}</span>
            <div class="response-screen-metric-value">{formatByteSize(Math.min(app.activeResponse.body_size, 256 * 1024))}</div>
          </div>
          <div class="response-screen-metric">
            <span class="screen-kicker">{t("response.storage")}</span>
            <div class="response-screen-metric-value">{app.activeResponse.body_size > 256 * 1024 ? t("response.disk") : t("response.inline")}</div>
          </div>
        </div>
        <div class="response-screen-tests">
          <span class="screen-kicker">{t("response.tests")}</span>
          {#if app.activeResponseTests.length}
            {#each app.activeResponseTests as test, i (i)}
              <div class="response-screen-test-row">
                <span class:status-ok={test.passed} class:status-err={!test.passed}>{test.passed ? t("response.pass") : t("response.fail")}</span>
                <span>{test.name}</span>
              </div>
            {/each}
          {:else}
            <p class="screen-empty-inline">{t("response.noTestsRan")}</p>
          {/if}
        </div>
      </aside>

      <div class="response-screen-main">
        <div class="response-subtabs">
          <button type="button" class="response-subtab" class:active={app.responseSubTab === "body"} onclick={() => (app.responseSubTab = "body")}>{t("response.body")}</button>
          <button type="button" class="response-subtab" class:active={app.responseSubTab === "headers"} onclick={() => (app.responseSubTab = "headers")}>
            {t("response.headers")} {#if app.activeResponse.headers?.length}<span class="tab-badge">{app.activeResponse.headers.length}</span>{/if}
          </button>
          <button type="button" class="response-subtab" class:active={app.responseSubTab === "cookies"} onclick={() => (app.responseSubTab = "cookies")}>
            {t("response.cookies")} {#if app.activeResponse.cookies?.length}<span class="tab-badge">{app.activeResponse.cookies.length}</span>{/if}
          </button>
          <button type="button" class="response-subtab" class:active={app.responseSubTab === "history"} onclick={() => (app.responseSubTab = "history")}>
            {t("response.history")} {#if app.responseHistory.length}<span class="tab-badge">{app.responseHistory.length}</span>{/if}
          </button>
          <div class="response-stat-spacer"></div>
          {#if app.responseSubTab === "body"}
            <button type="button" class="btn-ghost btn-xs" class:active={app.responseViewMode === "pretty"} onclick={() => (app.responseViewMode = "pretty")}>{t("response.pretty")}</button>
            <button type="button" class="btn-ghost btn-xs" class:active={app.responseViewMode === "raw"} onclick={() => (app.responseViewMode = "raw")}>{t("response.raw")}</button>
            {#if app.responseBodyIsHtml}
              <button type="button" class="btn-ghost btn-xs" class:active={app.responseViewMode === "preview"} onclick={() => (app.responseViewMode = "preview")}>{t("response.preview")}</button>
            {/if}
          {/if}
          <button type="button" class="btn-ghost btn-xs" onclick={copyResponseBody}>{t("response.copyAction")}</button>
          <button type="button" class="btn-ghost btn-xs" onclick={downloadResponseBody}>{t("response.saveToFile")}</button>
        </div>
        <div class="response-screen-content">
          {#if app.responseSubTab === "body"}
            {#if app.responseViewMode === "preview" && app.responseBodyIsHtml}
              <iframe class="response-preview-frame" title={t("response.preview")} sandbox="" srcdoc={app.activeResponseBody}></iframe>
            {:else if app.responseBodyIsJson}
              <pre class="body-view screen-body-view">{@html app.highlightedResponseBody}</pre>
            {:else}
              <pre class="body-view screen-body-view">{app.prettyResponseBody}</pre>
            {/if}
            {#if app.activeResponseTruncated}<p class="hint">{t("response.truncated")}</p>{/if}
          {:else if app.responseSubTab === "headers"}
            {#if app.activeResponse.headers?.length}
              <div class="headers-list">
                {#each app.activeResponse.headers as h}
                  <div class="header-line"><strong>{h.key}:</strong> {h.value}</div>
                {/each}
              </div>
            {:else}
              <p class="empty">{t("response.noHeaders")}</p>
            {/if}
          {:else if app.responseSubTab === "cookies"}
            {#if app.activeResponse.cookies?.length}
              <div class="headers-list">
                {#each app.activeResponse.cookies as c}
                  <div class="header-line"><strong>{c.name}:</strong> {c.value}</div>
                {/each}
              </div>
            {:else}
              <p class="empty">{t("response.noCookies")}</p>
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
      </div>
    </div>
  {/if}
</section>
