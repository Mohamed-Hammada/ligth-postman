<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconChevronDown, iconSave } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { sendAndDownload, cancelCurrentSend, handleAutocompleteKeydown, handleDebugRequest, handleDownloadResponse, handleGenericInputMouseLeave, handleGenericInputMouseMove, handleUrlPaste, handleVisualizeResponse, handleWriteTests, hideAutocompleteSoon, saveRequest, scheduleAutoSave, scheduleHideMissingVarPopover, sendCurrentRequest, setRightSidebarVisible, showVarPopover, syncUrlOverlayScroll, t, updateAutocompleteFor } = app;
</script>

<form class="request-bar" onsubmit={(e) => { e.preventDefault(); saveRequest(); }}>
  <div class="request-bar-row">
    <div class="url-pill">
      <select bind:value={app.editMethod} class="method-select method-{app.editMethod.toLowerCase()}" onchange={scheduleAutoSave}>
        <option>GET</option>
        <option>POST</option>
        <option>PUT</option>
        <option>PATCH</option>
        <option>DELETE</option>
        <option>HEAD</option>
        <option>OPTIONS</option>
        <option>TRACE</option>
      </select>
      <span class="url-pill-divider"></span>
      <div class="url-input-shell">
        <div class="url-token-overlay" aria-hidden="true">
          {#each app.urlTokens as tok, i (i)}
            {#if tok.type === "text"}
              <span class="url-token-text">{tok.text}</span>
            {:else}
              {@const missing = app.requestDiagnostics?.all_missing?.includes(tok.name) ?? false}
              <span
                class="url-token-var"
                class:missing
                role="presentation"
                onmouseenter={(e) => showVarPopover(tok.name, e.currentTarget as HTMLElement)}
                onmouseleave={() => scheduleHideMissingVarPopover()}
                onclick={(e) => {
                  const shell = (e.currentTarget as HTMLElement).closest(".url-input-shell");
                  const input = shell?.querySelector("input.url-input") as HTMLInputElement | null;
                  if (input) input.focus();
                }}
              >
                {tok.raw}
              </span>
            {/if}
          {/each}
        </div>
        <input
          placeholder={t("request.urlPlaceholder")}
          bind:value={app.editUrl}
          class="url-input url-input-ghost"
          oninput={(e) => { scheduleAutoSave(); updateAutocompleteFor(e.currentTarget as HTMLInputElement, "var", false, (v) => (app.editUrl = v)); }}
          onkeydown={handleAutocompleteKeydown}
          onblur={hideAutocompleteSoon}
          onpaste={handleUrlPaste}
          onscroll={syncUrlOverlayScroll}
          onmousemove={handleGenericInputMouseMove}
          onmouseleave={handleGenericInputMouseLeave}
        />
      </div>
    </div>
    <div class="send-action">
      {#if app.sending}
        <button type="button" class="btn-cancel" onclick={cancelCurrentSend}>{t("request.cancel")}</button>
      {:else}
        <div class="btn-send-group">
          <button type="button" class="btn-send" onclick={sendCurrentRequest}>{t("request.send")}</button>
          <div class="menu-wrap">
            <button
              type="button"
              class="btn-send-caret"
              title={t("request.sendOptions")}
              onclick={() => (app.sendMenuOpen = !app.sendMenuOpen)}
            >{@render iconChevronDown()}</button>
            {#if app.sendMenuOpen}
              <button type="button" class="dropdown-backdrop" style="background: transparent !important; border: none !important;" aria-label={t("common.close")} onclick={() => (app.sendMenuOpen = false)}></button>
              <div class="send-dropdown-menu">
                <button type="button" class="send-menu-item" onclick={handleVisualizeResponse}>
                  <svg class="send-menu-svg" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M1 8s3-5 7-5 7 5 7 5-3 5-7 5-7-5-7-5z"/><circle cx="8" cy="8" r="2.5"/></svg>
                  <span>Send and show test results</span>
                </button>
                <button type="button" class="send-menu-item" onclick={handleWriteTests}>
                  <svg class="send-menu-svg" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 2h10v12H3z"/><path d="M6 6h4M6 9h4M6 12h2"/></svg>
                  <span>Write tests</span>
                </button>
                <button type="button" class="send-menu-item" onclick={handleDebugRequest}>
                  <svg class="send-menu-svg" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M8 2a3 3 0 0 0-3 3v2h6V5a3 3 0 0 0-3-3zM4 9a4 4 0 0 0 8 0v2a4 4 0 0 1-8 0V9zM2 8h2M12 8h2M3 13l2-1M13 13l-2-1M3 5l2 1M13 5l-2 1"/></svg>
                  <span>Debug request</span>
                </button>
                <button type="button" class="send-menu-item" onclick={() => { app.sendMenuOpen = false; sendAndDownload(); }}>
                  <svg class="send-menu-svg" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><polygon points="2,2 9,6 2,10"/><path d="M12 3v8M9 8l3 3 3-3"/></svg>
                  <span>{t("request.sendAndDownload")}</span>
                </button>
                <div class="send-menu-divider"></div>
                <button type="button" class="send-menu-item" onclick={handleDownloadResponse}>
                  <svg class="send-menu-svg" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M8 2v9M4 8l4 4 4-4M2 14h12"/></svg>
                  <span>Download response</span>
                </button>
              </div>
            {/if}
          </div>
        </div>
      {/if}
      <button
        type="button"
        class="btn-save"
        class:is-error={app.autoSaveStatus === "error"}
        class:is-unsaved={app.autoSaveStatus === "unsaved"}
        title={app.autoSaveStatus === "saving" ? t("request.saving") : app.autoSaveStatus === "unsaved" ? t("request.unsaved") : app.autoSaveStatus === "error" ? t("request.saveFailed") : t("request.saved")}
        onclick={() => saveRequest()}
      >
        {@render iconSave()}
        <span>{t("request.saveLabel")}</span>
        {#if app.autoSaveStatus === "unsaved" || app.autoSaveStatus === "error"}<span class="dirty-dot" aria-hidden="true">•</span>{/if}
      </button>
      <button
        type="button"
        class="icon-btn btn-code-toggle"
        class:active={app.rightSidebarVisible && app.rightPanel === "code"}
        title={t("bottom.codeSnippet")}
        onclick={() => {
          if (app.rightSidebarVisible && app.rightPanel === "code") {
            setRightSidebarVisible(false);
          } else {
            app.rightPanel = "code";
            setRightSidebarVisible(true);
          }
        }}
      >&lt;/&gt;</button>
    </div>
  </div>
  {#if app.curlDetectedFeedback || app.sendCancelledNotice}
    <div class="request-bar-row secondary">
      {#if app.curlDetectedFeedback}
        <span class="hint">{app.curlDetectedFeedback}</span>
      {/if}
      {#if app.sendCancelledNotice}
        <span class="hint">{app.sendCancelledNotice}</span>
      {/if}
    </div>
  {/if}
</form>
