<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { closePalette, t, togglePaletteField } = app;
</script>

{#if app.paletteOpen}
  <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) closePalette(); }} onkeydown={(e) => { if (e.key === "Escape") closePalette(); }} role="dialog" aria-modal="true" tabindex="0">
    <div class="palette">
      <div class="palette-header">
        <span class="screen-kicker">{t("palette.goTo")}</span>
        <input class="palette-input" placeholder={t("palette.placeholder")} bind:value={app.paletteQuery} bind:this={app.paletteInputEl} />
      </div>
      <div class="palette-scope-row">
        <button type="button" class="palette-scope-btn" class:active={app.paletteScope === "all"} onclick={() => (app.paletteScope = "all")}>{t("palette.scopeAll")}</button>
        <button type="button" class="palette-scope-btn" class:active={app.paletteScope === "projects"} onclick={() => (app.paletteScope = "projects")}>{t("palette.scopeProjects")}</button>
        <button type="button" class="palette-scope-btn" class:active={app.paletteScope === "apis"} onclick={() => (app.paletteScope = "apis")}>{t("palette.scopeApis")}</button>
      </div>
      {#if app.paletteScope !== "projects"}
        <div class="palette-scope-row palette-field-row">
          <span class="palette-field-label">{t("palette.fieldsLabel")}</span>
          <button type="button" class="palette-scope-btn" class:active={app.paletteFields.name} aria-pressed={app.paletteFields.name} onclick={() => togglePaletteField("name")}>{t("palette.fieldName")}</button>
          <button type="button" class="palette-scope-btn" class:active={app.paletteFields.url} aria-pressed={app.paletteFields.url} onclick={() => togglePaletteField("url")}>{t("palette.fieldUrl")}</button>
          <button type="button" class="palette-scope-btn" class:active={app.paletteFields.body} aria-pressed={app.paletteFields.body} onclick={() => togglePaletteField("body")}>{t("palette.fieldBody")}</button>
        </div>
      {/if}
      <div class="palette-results">
        {#each app.paletteItems as p, i (i)}
          <button type="button" class="palette-item" onclick={p.onSelect}>
            <span class="palette-item-method">{p.method ?? ""}</span>
            <span class="palette-item-label">
              {#if p.project}<span class="palette-item-breadcrumb">{p.project} ›</span>{/if}
              {p.label}
            </span>
            <div class="response-stat-spacer"></div>
            <span class="palette-item-hint">{p.hint}</span>
          </button>
        {:else}
          <p class="screen-empty-inline">{t("palette.noMatches")}</p>
        {/each}
      </div>
      <div class="palette-footer">
        <span>{t("palette.openHint")}</span>
        <span>{t("palette.dismissHint")}</span>
      </div>
    </div>
  </div>
{/if}
