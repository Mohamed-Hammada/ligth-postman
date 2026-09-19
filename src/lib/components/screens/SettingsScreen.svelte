<script lang="ts">
  import type { App } from "$lib/app/controller.svelte";
  let { app }: { app: App } = $props();
  const { ACCENT_PRESETS, SHORTCUT_DEFS, SURFACE_TINTS, THEME_FONT_OPTIONS, THEME_OPTIONS, accentStyleOverride, fontStyleOverride, formatByteSize, isCustomSurfaceTint, setAccentColor, setAutoSyncIntervalMs, setBodyFontOverride, setHeadingFontOverride, setLocale, setShortcutEnabled, setSurfaceTint, setTextColorOverride, setThemeMode, setUiScale, surfaceStyleOverride, t, textColorStyleOverride, toggleSidebarSectionVisibility } = app;
</script>

<section class="screen-page">
  <div class="screen-page-header">
    <span class="screen-kicker">{t("rail.settings")}</span>
    <h1 class="screen-title">{t("settings.title")}</h1>
    <p class="screen-subtitle">{t("settings.subtitle")}</p>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.autoSyncInterval")}</div>
      <div class="screen-empty-inline">{t("settings.autoSyncHint")}</div>
    </div>
    <div class="seg">
      {#each [[30000, "30s"], [60000, "60s"], [120000, "2m"], [300000, "5m"]] as [ms, label]}
        <button type="button" class="seg-opt" class:active={app.autoSyncIntervalMs === ms} onclick={() => setAutoSyncIntervalMs(ms as number)}>{label}</button>
      {/each}
    </div>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.appearance")}</div>
      <div class="screen-empty-inline">{t("settings.appearanceHint")}</div>
    </div>
    <div class="seg">
      {#each THEME_OPTIONS as opt (opt.id)}
        <button type="button" class="seg-opt" class:active={app.themeMode === opt.id} onclick={() => setThemeMode(opt.id)}>{t(opt.label)}</button>
      {/each}
    </div>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">Sidebar Sections</div>
      <div class="screen-empty-inline">Choose which sections appear in the sidebar accordion.</div>
    </div>
    <div class="settings-sections-checkbox-grid">
      <label class="settings-checkbox-pill" class:active={app.sidebarSectionsVisible.collections}>
        <input type="checkbox" checked={app.sidebarSectionsVisible.collections} onchange={() => toggleSidebarSectionVisibility("collections")} />
        <span>Collections</span>
      </label>
      <label class="settings-checkbox-pill" class:active={app.sidebarSectionsVisible.environments}>
        <input type="checkbox" checked={app.sidebarSectionsVisible.environments} onchange={() => toggleSidebarSectionVisibility("environments")} />
        <span>Environments</span>
      </label>
      <label class="settings-checkbox-pill" class:active={app.sidebarSectionsVisible.datasets}>
        <input type="checkbox" checked={app.sidebarSectionsVisible.datasets} onchange={() => toggleSidebarSectionVisibility("datasets")} />
        <span>Datasets <em class="preview-note">(preview)</em></span>
      </label>
      <label class="settings-checkbox-pill" class:active={app.sidebarSectionsVisible.documents}>
        <input type="checkbox" checked={app.sidebarSectionsVisible.documents} onchange={() => toggleSidebarSectionVisibility("documents")} />
        <span>Documents <em class="preview-note">(preview)</em></span>
      </label>
      <label class="settings-checkbox-pill" class:active={app.sidebarSectionsVisible.specs}>
        <input type="checkbox" checked={app.sidebarSectionsVisible.specs} onchange={() => toggleSidebarSectionVisibility("specs")} />
        <span>Specs <em class="preview-note">(preview)</em></span>
      </label>
      <label class="settings-checkbox-pill" class:active={app.sidebarSectionsVisible.mocks}>
        <input type="checkbox" checked={app.sidebarSectionsVisible.mocks} onchange={() => toggleSidebarSectionVisibility("mocks")} />
        <span>Mocks <em class="preview-note">(preview)</em></span>
      </label>
      <label class="settings-checkbox-pill" class:active={app.sidebarSectionsVisible.flows}>
        <input type="checkbox" checked={app.sidebarSectionsVisible.flows} onchange={() => toggleSidebarSectionVisibility("flows")} />
        <span>Flows <em class="preview-note">(preview)</em></span>
      </label>
    </div>
  </div>

  <div class="settings-screen-block">
    <p class="screen-subtitle">{t("theme.subtitle")}</p>
    <div class="theme-compare">
      {#each THEME_OPTIONS as opt (opt.id)}
        <button type="button" class="theme-compare-col" class:active={app.themeMode === opt.id} onclick={() => setThemeMode(opt.id)}>
          <div class="theme-compare-header">
            <span class="screen-kicker">{t(opt.label)}</span>
            {#if app.themeMode === opt.id}<span class="theme-active-badge">{t("theme.active")}</span>{/if}
          </div>
          <div
            class="theme-swatch"
            data-theme={opt.id}
            style="{surfaceStyleOverride(app.surfaceTint, opt.id)} {accentStyleOverride(app.accentColor)} {fontStyleOverride(app.headingFontOverride, app.bodyFontOverride)} {textColorStyleOverride(app.textColorOverride)}"
          >
            <div class="theme-swatch-topbar">
              <span>{t("rail.brand")}</span>
              <span class="theme-swatch-sync">SYNC</span>
            </div>
            <div class="theme-swatch-body">
              <div class="theme-swatch-side">
                <div class="screen-kicker">Explorer</div>
                <div>List charges</div>
                <div>Create charge</div>
                <div class="theme-swatch-active">Refund charge</div>
              </div>
              <div class="theme-swatch-main">
                <div class="theme-swatch-path">POST /v1/charges/:id/refund</div>
                <div class="hr"></div>
                <div class="theme-swatch-status">200 OK</div>
                <div class="screen-empty-inline">214 ms · 1.2 KB</div>
              </div>
            </div>
          </div>
        </button>
      {/each}
    </div>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.accentColor")}</div>
      <div class="screen-empty-inline">{t("settings.accentColorHint")}</div>
    </div>
    <div class="accent-picker">
      {#each ACCENT_PRESETS as preset (preset.id)}
        <button
          type="button"
          class="accent-swatch"
          class:active={app.accentColor === preset.hex}
          style="background: {preset.hex}"
          title={t(`accent.${preset.id}`)}
          onclick={() => setAccentColor(preset.hex)}
        ></button>
      {/each}
      <label class="accent-swatch accent-swatch-custom" style={app.accentColor && !ACCENT_PRESETS.some((p) => p.hex === app.accentColor) ? `background: ${app.accentColor}` : ""} title={t("accent.custom")}>
        <input type="color" value={app.accentColor ?? "#c1603f"} oninput={(e) => setAccentColor((e.currentTarget as HTMLInputElement).value)} />
        {#if !app.accentColor || ACCENT_PRESETS.some((p) => p.hex === app.accentColor)}<span class="accent-swatch-plus">+</span>{/if}
      </label>
      {#if app.accentColor}
        <button type="button" class="btn-ghost accent-reset" onclick={() => setAccentColor(null)}>{t("accent.reset")}</button>
      {/if}
    </div>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.surfaceTint")}</div>
      <div class="screen-empty-inline">{app.surfaceTintAvailable ? t("settings.surfaceTintHint") : t("settings.surfaceTintUnavailable")}</div>
    </div>
    {#if app.surfaceTintAvailable}
      <div class="accent-picker">
        {#each SURFACE_TINTS as tint (tint.id)}
          <button
            type="button"
            class="accent-swatch"
            class:active={app.surfaceTint === tint.id}
            style="background: {app.themeMode === 'dark' ? tint.dark.bg : tint.light.bg}"
            title={t(`surfaceTint.${tint.id}`)}
            onclick={() => setSurfaceTint(tint.id)}
          ></button>
        {/each}
        <label class="accent-swatch accent-swatch-custom" style={isCustomSurfaceTint(app.surfaceTint) ? `background: ${app.surfaceTint}` : ""} title={t("accent.custom")}>
          <input type="color" value={isCustomSurfaceTint(app.surfaceTint) ? app.surfaceTint : "#faf6ef"} oninput={(e) => setSurfaceTint((e.currentTarget as HTMLInputElement).value)} />
          {#if !isCustomSurfaceTint(app.surfaceTint)}<span class="accent-swatch-plus">+</span>{/if}
        </label>
        {#if app.surfaceTint}
          <button type="button" class="btn-ghost accent-reset" onclick={() => setSurfaceTint(null)}>{t("accent.reset")}</button>
        {/if}
      </div>
    {/if}
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.headingFont")}</div>
      <div class="screen-empty-inline">{t("settings.headingFontHint")}</div>
    </div>
    <select
      class="url-input"
      value={app.headingFontOverride ?? ""}
      onchange={(e) => setHeadingFontOverride((e.currentTarget as HTMLSelectElement).value || null)}
    >
      <option value="">{t("settings.themeDefault")}</option>
      {#each THEME_FONT_OPTIONS as font (font.id)}
        <option value={font.id}>{font.label}</option>
      {/each}
    </select>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.bodyFont")}</div>
      <div class="screen-empty-inline">{t("settings.bodyFontHint")}</div>
    </div>
    <select
      class="url-input"
      value={app.bodyFontOverride ?? ""}
      onchange={(e) => setBodyFontOverride((e.currentTarget as HTMLSelectElement).value || null)}
    >
      <option value="">{t("settings.themeDefault")}</option>
      {#each THEME_FONT_OPTIONS as font (font.id)}
        <option value={font.id}>{font.label}</option>
      {/each}
    </select>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.textColor")}</div>
      <div class="screen-empty-inline">{t("settings.textColorHint")}</div>
      {#if app.textColorContrastWarning}<div class="warn-inline">{app.textColorContrastWarning}</div>{/if}
    </div>
    <div class="accent-picker">
      <label class="accent-swatch accent-swatch-custom" style={app.textColorOverride ? `background: ${app.textColorOverride}` : ""} title={t("accent.custom")}>
        <input type="color" value={app.textColorOverride ?? "#2b2620"} oninput={(e) => setTextColorOverride((e.currentTarget as HTMLInputElement).value)} />
        {#if !app.textColorOverride}<span class="accent-swatch-plus">+</span>{/if}
      </label>
      {#if app.textColorOverride}
        <button type="button" class="btn-ghost accent-reset" onclick={() => setTextColorOverride(null)}>{t("accent.reset")}</button>
      {/if}
    </div>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.uiScale")}</div>
      <div class="screen-empty-inline">{t("settings.uiScaleHint")}</div>
    </div>
    <div class="seg">
      {#each [[85, t("settings.scaleSmall")], [100, t("settings.scaleDefault")], [115, t("settings.scaleLarge")], [130, t("settings.scaleExtraLarge")]] as [pct, label]}
        <button type="button" class="seg-opt" class:active={app.uiScale === pct} onclick={() => setUiScale(pct as number)}>{label}</button>
      {/each}
    </div>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.language")}</div>
      <div class="screen-empty-inline">{t("settings.languageHint")}</div>
    </div>
    <div class="seg">
      <button type="button" class="seg-opt" class:active={app.locale === "en"} onclick={() => setLocale("en")}>{t("settings.languageEnglish")}</button>
      <button type="button" class="seg-opt" class:active={app.locale === "ar"} onclick={() => setLocale("ar")}>{t("settings.languageArabic")}</button>
    </div>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.shortcuts")}</div>
      <div class="screen-empty-inline">{t("settings.shortcutsHint")}</div>
    </div>
    <div class="shortcuts-list">
      {#each SHORTCUT_DEFS as def (def.id)}
        <label class="shortcut-toggle-row">
          <input
            type="checkbox"
            checked={app.shortcutsEnabled[def.id]}
            onchange={(e) => setShortcutEnabled(def.id, (e.target as HTMLInputElement).checked)}
          />
          <span class="shortcut-label">{t(def.label)}</span>
          <kbd class="shortcut-keys">{def.keys}</kbd>
        </label>
      {/each}
    </div>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.responseMemory")}</div>
      <div class="screen-empty-inline">{t("settings.responseMemoryHint")}</div>
    </div>
    <div class="settings-screen-row-value">256 KB</div>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.scriptTimeout")}</div>
      <div class="screen-empty-inline">{t("settings.scriptTimeoutHint")}</div>
    </div>
    <div class="settings-screen-row-value">1.5s / 2s</div>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.responseCache")}</div>
      <div class="screen-empty-inline">{t("settings.responseCacheHint")}</div>
    </div>
    <div class="settings-screen-row-value">{t("settings.none")}</div>
  </div>

  <div class="settings-screen-row">
    <div>
      <div class="settings-screen-row-label">{t("settings.historyRetention")}</div>
      <div class="screen-empty-inline">{t("settings.historyRetentionHint")}</div>
    </div>
    <div class="settings-screen-row-value">{t("settings.unbounded")}</div>
  </div>

  {#if app.systemDiagnostics}
    <div class="settings-screen-row">
      <div>
        <div class="settings-screen-row-label">{t("settings.currentProcess")}</div>
        <div class="screen-empty-inline">{t("settings.currentProcessHint")}</div>
      </div>
      <div class="settings-screen-diagnostics-grid">
        <span>{t("settings.memoryRss")}</span><strong>{formatByteSize(app.systemDiagnostics.process_rss_bytes)}</strong>
        <span>{t("settings.database")}</span><strong>{formatByteSize(app.systemDiagnostics.db_size_bytes)}</strong>
        <span>{t("settings.walFile")}</span><strong>{formatByteSize(app.systemDiagnostics.db_wal_size_bytes)}</strong>
        <span>{t("settings.projects")}</span><strong>{app.systemDiagnostics.total_projects}</strong>
        <span>{t("settings.requests")}</span><strong>{app.systemDiagnostics.total_requests}</strong>
        <span>{t("settings.responsesStored")}</span><strong>{app.systemDiagnostics.total_responses}</strong>
        <span>{t("settings.consoleEvents")}</span><strong>{app.systemDiagnostics.console_events_count}</strong>
        <span>{t("settings.uptime")}</span><strong>{Math.floor(app.systemDiagnostics.uptime_seconds / 60)} min</strong>
      </div>
    </div>
  {/if}
</section>
