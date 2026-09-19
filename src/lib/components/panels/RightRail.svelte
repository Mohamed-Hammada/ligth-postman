<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  import { iconEye, iconInfo } from "$lib/components/icons.svelte";
  let { app }: { app: App } = $props();
  const { loadVariables, setRightSidebarVisible } = app;
</script>

<aside class="right-utility-rail" aria-label="Utility rail">
  <button
    type="button"
    class="rail-action-btn"
    class:active={app.rightSidebarVisible && app.rightPanel === "ai"}
    title="Postman AI Assistant"
    onclick={() => {
      if (app.rightSidebarVisible && app.rightPanel === "ai") {
        setRightSidebarVisible(false);
        app.rightPanel = null;
      } else {
        app.rightPanel = "ai";
        setRightSidebarVisible(true);
      }
    }}
  >
    <span class="rail-action-icon">🪄</span>
    <span class="rail-action-label">AI</span>
  </button>

  <button
    type="button"
    class="rail-action-btn"
    class:active={app.rightSidebarVisible && app.rightPanel === "variables"}
    title="All variables (Environment quick look)"
    onclick={() => {
      if (app.rightSidebarVisible && app.rightPanel === "variables") {
        setRightSidebarVisible(false);
        app.rightPanel = null;
      } else {
        loadVariables();
        app.rightPanel = "variables";
        setRightSidebarVisible(true);
      }
    }}
  >
    <span class="rail-action-icon">{@render iconEye()}</span>
  </button>

  <button
    type="button"
    class="rail-action-btn"
    class:active={app.rightSidebarVisible && (app.rightPanel === "code" || !app.rightPanel)}
    title="Code snippet"
    onclick={() => {
      if (app.rightSidebarVisible && app.rightPanel === "code") {
        setRightSidebarVisible(false);
        app.rightPanel = null;
      } else {
        app.rightPanel = "code";
        setRightSidebarVisible(true);
      }
    }}
  >
    <span class="rail-action-code">&lt;/&gt;</span>
  </button>

  <button
    type="button"
    class="rail-action-btn"
    class:active={app.rightSidebarVisible && app.rightPanel === "comments"}
    title="Comments"
    onclick={() => {
      if (app.rightSidebarVisible && app.rightPanel === "comments") {
        setRightSidebarVisible(false);
        app.rightPanel = null;
      } else {
        app.rightPanel = "comments";
        setRightSidebarVisible(true);
      }
    }}
  >
    <span class="rail-action-icon">💬</span>
  </button>

  <button
    type="button"
    class="rail-action-btn"
    class:active={app.rightSidebarVisible && app.rightPanel === "info"}
    title="Request info"
    onclick={() => {
      if (app.rightSidebarVisible && app.rightPanel === "info") {
        setRightSidebarVisible(false);
        app.rightPanel = null;
      } else {
        app.rightPanel = "info";
        setRightSidebarVisible(true);
      }
    }}
  >
    <span class="rail-action-icon">{@render iconInfo()}</span>
  </button>
</aside>
