<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { cancelHideMissingVarPopover, findResolvedVariable, saveVariableFromPopover, scheduleHideMissingVarPopover, t } = app;
</script>

{#if app.missingVarHover && app.hoveredVarRect}
  <div
    class="hovered-var-token-highlight"
    style="top: {app.hoveredVarRect.top}px; left: {app.hoveredVarRect.left}px; width: {app.hoveredVarRect.right - app.hoveredVarRect.left}px; height: {app.hoveredVarRect.bottom - app.hoveredVarRect.top}px;"
    aria-hidden="true"
  ></div>
{/if}

{#if app.missingVarHover}
  {@const resolved = findResolvedVariable(app.missingVarHover.name)}
  {@const activeEnv = app.allEnvironments.find((e) => e.id === app.selectedEnvironmentId)}
  <div
    class="postman-var-popover"
    class:place-above={app.missingVarHover.placeAbove}
    role="dialog"
    aria-label={t(resolved ? "var.editValueFor" : "missingvar.addValueFor", { name: app.missingVarHover.name })}
    style="top: {app.missingVarHover.top}px; left: {app.missingVarHover.left}px;"
    onmouseenter={cancelHideMissingVarPopover}
    onmouseleave={() => scheduleHideMissingVarPopover(250)}
  >
    <div class="var-popover-header">
      <div class="var-popover-title-group">
        <span class="var-popover-var-symbol" aria-hidden="true">&#123;&#123; &#125;&#125;</span>
        <span class="var-popover-var-name">{app.missingVarHover.name}</span>
      </div>
      <div class="var-popover-badge-group">
        {#if resolved}
          {#if resolved.scope === "environment"}
            <span class="var-scope-badge badge-env" title="Environment Variable">ENVIRONMENT</span>
          {:else}
            <span class="var-scope-badge badge-global" title="Global Variable">GLOBAL</span>
          {/if}
        {:else}
          <span class="var-scope-badge badge-unresolved" title="Unresolved Variable">UNRESOLVED</span>
        {/if}
        <button
          type="button"
          class="var-popover-close-btn"
          title="Close"
          onclick={() => { app.missingVarHover = null; app.hoveredVarRect = null; }}
        >&times;</button>
      </div>
    </div>

    {#if resolved && resolved.scope === "environment"}
      <div class="var-popover-env-context">
        <span class="env-context-label">Environment:</span>
        <span class="env-context-name">{activeEnv?.name ?? "Active Environment"}</span>
      </div>
    {:else if !resolved}
      <div class="var-popover-unresolved-hint">
        Not resolved in current environment or globals.
      </div>
    {/if}

    <div class="var-popover-body">
      <label class="var-popover-label" for="popover-var-val">CURRENT VALUE</label>
      <div class="var-popover-input-wrap">
        <input
          id="popover-var-val"
          class="var-popover-input"
          placeholder={resolved ? "Value..." : "Set value to define variable..."}
          value={app.missingVarDrafts[app.missingVarHover.name] ?? ""}
          onfocus={() => { app.popoverInputFocused = true; cancelHideMissingVarPopover(); }}
          onblur={() => { app.popoverInputFocused = false; scheduleHideMissingVarPopover(300); }}
          oninput={(e) => (app.missingVarDrafts[app.missingVarHover!.name] = (e.target as HTMLInputElement).value)}
          onkeydown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              saveVariableFromPopover(app.missingVarHover!.name);
            } else if (e.key === "Escape") {
              e.preventDefault();
              app.missingVarHover = null;
              app.hoveredVarRect = null;
            }
          }}
        />
      </div>
    </div>

    <div class="var-popover-footer">
      <div class="var-popover-status">
        {#if app.popoverSaveSuccess}
          <span class="save-success-msg">&#10003; Saved</span>
        {/if}
      </div>
      <div class="var-popover-actions">
        <button
          type="button"
          class="var-popover-btn var-popover-btn-save"
          onclick={() => saveVariableFromPopover(app.missingVarHover!.name)}
        >
          {#if resolved}
            Save
          {:else if app.selectedEnvironmentId}
            Add to Environment
          {:else}
            Add to Global
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
