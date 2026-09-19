<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconChevronDown, iconChevronRight, iconClose } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { asCookieRows, asHeaderRows, clearConsole, copyConsoleLog, copyEventDetails, detailStr, exportConsoleJson, formatByteSize, formatConsoleTime, refreshConsoleEvents, startConsoleResize, t, toggleEventExpanded, toggleRawDetails } = app;
</script>

{#if app.showConsole}
  <div
    class="console-resize-handle"
    class:resizing={app.consoleResizing}
    onmousedown={startConsoleResize}
    onkeydown={(e) => {
      if (e.key === "ArrowUp") app.consoleHeight = Math.min(window.innerHeight - 160, app.consoleHeight + 16);
      else if (e.key === "ArrowDown") app.consoleHeight = Math.max(120, app.consoleHeight - 16);
    }}
    role="slider"
    aria-orientation="horizontal"
    aria-label={t("console.resizeHandle")}
    aria-valuenow={app.consoleHeight}
    aria-valuemin={120}
    aria-valuemax={900}
    tabindex="0"
  ></div>
  <div class="console-drawer" style="height: {app.consoleHeight}px">
    <div class="console-header">
      <div class="console-title-group">
        <span class="console-title">{t("console.title")}</span>
        <span class="console-count-badge">{t("console.eventsCount", { count: app.filteredConsoleEvents.length })}</span>
      </div>

      <div class="console-toolbar">
        <select bind:value={app.consoleLevelFilter} class="console-select">
          <option value="all">{t("console.allLevels")}</option>
          <option value="info">{t("console.info")}</option>
          <option value="warn">{t("console.warn")}</option>
          <option value="error">{t("console.error")}</option>
          <option value="debug">{t("console.debug")}</option>
        </select>

        <label class="console-check-label" title={t("console.activeRequestOnlyTitle")}>
          <input type="checkbox" bind:checked={app.consoleActiveRequestOnly} />
          {t("console.activeRequestOnly")}
        </label>

        <input
          type="search"
          placeholder={t("console.filterPlaceholder")}
          bind:value={app.consoleSearchFilter}
          class="console-search"
        />

        <button type="button" class="console-btn" onclick={refreshConsoleEvents} title={t("console.refreshTitle")}>{t("console.refresh")}</button>
        <button type="button" class="console-btn" onclick={clearConsole} title={t("console.clearTitle")}>{t("console.clear")}</button>
        <button type="button" class="console-btn" onclick={copyConsoleLog} title={t("console.copyTitle")}>{t("console.copy")}</button>
        <button type="button" class="console-btn" onclick={exportConsoleJson} title={t("console.exportJsonTitle")}>{t("console.exportJson")}</button>
        <button type="button" class="console-close-btn" onclick={() => (app.showConsole = false)} title={t("console.closeTitle")}>{@render iconClose()}</button>
      </div>
    </div>

    <div class="console-body">
      {#if app.filteredConsoleEvents.length === 0}
        <div class="console-empty">{t("console.empty")}</div>
      {:else}
        <div class="console-events-list">
          {#each app.filteredConsoleEvents as evt (evt.id)}
            <div class="console-row" class:error-row={evt.level === "error"} class:warn-row={evt.level === "warn"}>
              <div
                class="console-row-summary"
                onclick={() => toggleEventExpanded(evt.id)}
                role="button"
                tabindex="0"
                onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") toggleEventExpanded(evt.id); }}
              >
                <span class="evt-expander">{#if app.expandedEventIds.has(evt.id)}{@render iconChevronDown()}{:else}{@render iconChevronRight()}{/if}</span>
                <span class="evt-time">{formatConsoleTime(evt.timestamp)}</span>
                <span class="evt-level level-{evt.level}">{evt.level.toUpperCase()}</span>
                <span class="evt-type">{evt.event_type}</span>
                <span class="evt-cid" title={t("console.correlationIdTitle", { id: evt.correlation_id ?? "" })}>#{evt.correlation_id ? evt.correlation_id.slice(0, 8) : "—"}</span>
                <span class="evt-msg">{evt.message}</span>
              </div>

              {#if app.expandedEventIds.has(evt.id) && evt.details}
                {@const d = evt.details as Record<string, unknown>}
                {@const known = ["request_start", "response_received", "cookie_injected", "request_error", "test_assertion"].includes(evt.event_type)}
                <div class="console-row-details">
                  <div class="details-actions">
                    {#if known}
                      <button type="button" class="console-mini-btn" onclick={() => toggleRawDetails(evt.id)}>
                        {app.rawDetailsVisible.has(evt.id) ? t("console.hideRawJson") : t("console.viewRawJson")}
                      </button>
                    {/if}
                    <button type="button" class="console-mini-btn" onclick={() => copyEventDetails(evt)}>{t("console.copyDetailsJson")}</button>
                  </div>

                  {#if evt.event_type === "request_start"}
                    <div class="detail-kv-grid">
                      <span class="detail-k">{t("console.method")}</span><span class="detail-v">{detailStr(d, "method")}</span>
                      <span class="detail-k">{t("console.url")}</span><span class="detail-v detail-v-wrap">{detailStr(d, "url")}</span>
                      <span class="detail-k">{t("console.authType")}</span><span class="detail-v">{detailStr(d, "auth_type")}</span>
                      <span class="detail-k">{t("console.bodyBytes")}</span><span class="detail-v">{formatByteSize(Number(d.body_bytes ?? 0))}</span>
                      <span class="detail-k">{t("console.timeout")}</span><span class="detail-v">{detailStr(d, "timeout_ms")} ms</span>
                      <span class="detail-k">{t("console.followRedirects")}</span><span class="detail-v">{String(d.follow_redirects)}</span>
                      <span class="detail-k">{t("console.verifySsl")}</span><span class="detail-v">{String(d.verify_ssl)}</span>
                      {#if detailStr(d, "proxy")}
                        <span class="detail-k">{t("console.proxy")}</span><span class="detail-v">{detailStr(d, "proxy")}</span>
                      {/if}
                      {#if detailStr(d, "http_version")}
                        <span class="detail-k">{t("console.httpVersion")}</span><span class="detail-v">{detailStr(d, "http_version")}</span>
                      {/if}
                    </div>
                    {#if asHeaderRows(d.headers).length}
                      <table class="detail-header-table">
                        <thead><tr><th>{t("params.key")}</th><th>{t("params.value")}</th></tr></thead>
                        <tbody>
                          {#each asHeaderRows(d.headers) as h, i (i)}
                            <tr class:disabled-row={h.enabled === false}><td>{h.key}</td><td>{h.value}</td></tr>
                          {/each}
                        </tbody>
                      </table>
                    {/if}
                  {:else if evt.event_type === "response_received"}
                    <div class="detail-kv-grid">
                      <span class="detail-k">{t("console.status")}</span><span class="detail-v">{detailStr(d, "status")} {detailStr(d, "status_text")}</span>
                      <span class="detail-k">{t("console.duration")}</span><span class="detail-v">{detailStr(d, "duration_ms")} ms</span>
                      <span class="detail-k">{t("console.bodySize")}</span><span class="detail-v">{formatByteSize(Number(d.body_size ?? 0))}</span>
                      {#if detailStr(d, "content_type")}
                        <span class="detail-k">{t("console.contentType")}</span><span class="detail-v">{detailStr(d, "content_type")}</span>
                      {/if}
                    </div>
                    {#if asHeaderRows(d.headers).length}
                      <table class="detail-header-table">
                        <thead><tr><th>{t("params.key")}</th><th>{t("params.value")}</th></tr></thead>
                        <tbody>
                          {#each asHeaderRows(d.headers) as h, i (i)}
                            <tr><td>{h.key}</td><td>{h.value}</td></tr>
                          {/each}
                        </tbody>
                      </table>
                    {/if}
                    {#if asCookieRows(d.cookies).length}
                      <table class="detail-header-table">
                        <thead><tr><th>{t("console.cookieName")}</th><th>{t("params.value")}</th><th>{t("console.cookieDomain")}</th></tr></thead>
                        <tbody>
                          {#each asCookieRows(d.cookies) as c, i (i)}
                            <tr><td>{c.name}</td><td>{c.value || "—"}</td><td>{c.domain}{c.path}</td></tr>
                          {/each}
                        </tbody>
                      </table>
                    {/if}
                  {:else if evt.event_type === "cookie_injected"}
                    <p class="detail-list-label">{t("console.injectedCookies")}</p>
                    <ul class="detail-plain-list">
                      {#each (Array.isArray(d.cookies) ? d.cookies : []) as c, i (i)}
                        <li>{String(c)}</li>
                      {/each}
                    </ul>
                  {:else if evt.event_type === "request_error"}
                    <div class="detail-kv-grid">
                      <span class="detail-k">{t("console.error")}</span><span class="detail-v detail-v-wrap">{detailStr(d, "error")}</span>
                    </div>
                  {:else if evt.event_type === "test_assertion"}
                    <div class="detail-kv-grid">
                      <span class="detail-k">{t("console.testName")}</span><span class="detail-v">{detailStr(d, "name")}</span>
                      <span class="detail-k">{t("console.testPassed")}</span><span class="detail-v">{d.passed ? t("console.pass") : t("console.fail")}</span>
                      {#if detailStr(d, "error")}
                        <span class="detail-k">{t("console.testError")}</span><span class="detail-v detail-v-wrap">{detailStr(d, "error")}</span>
                      {/if}
                    </div>
                  {/if}

                  {#if !known || app.rawDetailsVisible.has(evt.id)}
                    <pre class="console-json-view">{JSON.stringify(evt.details, null, 2)}</pre>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}
