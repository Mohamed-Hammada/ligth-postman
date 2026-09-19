<script module lang="ts">
  export {
    iconSearch,
    iconBell,
    iconHome,
    iconUser,
    iconArrowLeft,
    iconArrowRight,
    iconCube,
    iconClose,
    iconCheck,
    iconCheckCircle,
    iconXCircle,
    iconTrash,
    iconEdit,
    iconWarning,
    iconInfo,
    iconMoreVertical,
    iconMenu,
    iconFolder,
    iconFolderOpen,
    iconFolderPlus,
    iconGlobe,
    iconImport,
    iconEye,
    iconLock,
    iconGitBranch,
    iconMonitor,
    iconGrid,
    iconLayout,
    iconSidebar,
    iconClock,
    iconSettings,
    iconSparkle,
    iconInboxEmpty,
    iconFileText,
    iconSave,
    iconStar,
    iconChevronRight,
    iconChevronLeft,
    iconChevronDown,
    iconChevronUp,
    iconExpandAll,
    iconCollapseAll,
    iconCopy,
    iconExpandDiagonal,
    iconCompressDiagonal,
    iconMinus,
    iconArrowUp,
    iconArrowDown,
    railScreenIcon,
  };
</script>

<!-- Icon library — one consistent stroke system (1.6px, square caps/joins, 16x16) replacing the
     platform-emoji glyphs the app used to lean on. Filled shapes are noted per-icon. Sized via
     `.icon { width/height: 1em }` so every call site scales with its own font-size (and with
     `uiScale`, since that's how the rest of the app's rem-based sizing already scales). -->

{#snippet iconSearch()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="7" cy="7" r="4.5"/><path d="M10.5 10.5L14 14"/></svg>{/snippet}
{#snippet iconBell()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M8 2.5a3 3 0 0 0-3 3v2.8l-1.2 2.2h8.4L11 8.3V5.5a3 3 0 0 0-3-3Z"/><path d="M6.8 12.5a1.2 1.2 0 0 0 2.4 0"/></svg>{/snippet}
{#snippet iconHome()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M2.5 7.5L8 3l5.5 4.5V13a1 1 0 0 1-1 1h-9a1 1 0 0 1-1-1V7.5Z"/><path d="M6 14V9h4v5"/></svg>{/snippet}
{#snippet iconUser()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="5.5" r="2.5"/><path d="M3.5 13.5c0-2.5 2-4 4.5-4s4.5 1.5 4.5 4"/></svg>{/snippet}
{#snippet iconArrowLeft()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M10 3.5L5.5 8l4.5 4.5"/></svg>{/snippet}
{#snippet iconArrowRight()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M6 3.5L10.5 8l-4.5 4.5"/></svg>{/snippet}
{#snippet iconCube()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M8 2.2l5 2.8v6l-5 2.8-5-2.8v-6l5-2.8z"/><path d="M8 2.2v11.6M3 5l5 2.8 5-2.8"/></svg>{/snippet}

{#snippet iconClose()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M4 4l8 8M12 4l-8 8"/></svg>{/snippet}
{#snippet iconCheck()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square" stroke-linejoin="miter"><path d="M3.5 8.5l3 3 6-7"/></svg>{/snippet}
{#snippet iconCheckCircle()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><path d="M5.2 8.2l2 2 3.6-4.4"/></svg>{/snippet}
{#snippet iconXCircle()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><path d="M6 6l4 4M10 6l-4 4"/></svg>{/snippet}
{#snippet iconTrash()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M3 5h10M6 5V3.6a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1V5"/><path d="M5.1 5l.7 8a1 1 0 0 0 1 .9h2.4a1 1 0 0 0 1-.9l.7-8"/><path d="M6.6 7.3v5M9.4 7.3v5"/></svg>{/snippet}
{#snippet iconEdit()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><path d="M10.4 3l2.6 2.6-7.4 7.4H3v-2.6z"/><path d="M9 4.4L11.6 7"/></svg>{/snippet}
{#snippet iconWarning()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><path d="M8 2.3L14.2 13H1.8Z"/><path d="M8 6.3v3.2"/><circle cx="8" cy="11.3" r="0.55" fill="currentColor" stroke="none"/></svg>{/snippet}
{#snippet iconInfo()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><path d="M8 7.3v4"/><circle cx="8" cy="4.9" r="0.55" fill="currentColor" stroke="none"/></svg>{/snippet}
{#snippet iconMoreVertical()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16"><circle cx="8" cy="3.7" r="1.15" fill="currentColor"/><circle cx="8" cy="8" r="1.15" fill="currentColor"/><circle cx="8" cy="12.3" r="1.15" fill="currentColor"/></svg>{/snippet}
{#snippet iconMenu()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M2.5 4.5h11M2.5 8h11M2.5 11.5h11"/></svg>{/snippet}
{#snippet iconFolder()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><path d="M2 4.5h4l1.2 1.5H14v6.5H2Z"/></svg>{/snippet}
{#snippet iconFolderOpen()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><path d="M2 4.8h4l1.2 1.5H14L12.7 12.5H3.3Z"/></svg>{/snippet}
{#snippet iconFolderPlus()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter" stroke-linecap="square"><path d="M2 4.5h4l1.2 1.5H14v6.5H2Z"/><path d="M8 7.3v3.4M6.3 9h3.4"/></svg>{/snippet}
{#snippet iconGlobe()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><path d="M2 8h12M8 2c2.1 2.1 2.1 9.9 0 12M8 2c-2.1 2.1-2.1 9.9 0 12"/></svg>{/snippet}
{#snippet iconImport()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square" stroke-linejoin="miter"><path d="M8 2.2v7.3M5 6.8L8 9.8l3-3"/><path d="M2.5 10.3v2.2a1 1 0 0 0 1 1h9a1 1 0 0 0 1-1v-2.2"/></svg>{/snippet}
{#snippet iconEye()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M1.4 8S4 3.6 8 3.6 14.6 8 14.6 8 12 12.4 8 12.4 1.4 8 1.4 8Z"/><circle cx="8" cy="8" r="2"/></svg>{/snippet}
{#snippet iconLock()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><rect x="3.3" y="7" width="9.4" height="6.3"/><path d="M5.5 7V5a2.5 2.5 0 0 1 5 0v2"/></svg>{/snippet}
{#snippet iconGitBranch()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="4.5" cy="3.6" r="1.3"/><circle cx="4.5" cy="12.4" r="1.3"/><circle cx="11.5" cy="7.6" r="1.3"/><path d="M4.5 4.9v6.2M4.5 8.2C4.5 5.9 6.6 5 10.2 4.7"/></svg>{/snippet}
{#snippet iconMonitor()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><rect x="2" y="3" width="12" height="8"/><path d="M6 13.3h4M8 11v2.3"/></svg>{/snippet}
{#snippet iconGrid()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><rect x="2.3" y="2.3" width="4.7" height="4.7"/><rect x="9" y="2.3" width="4.7" height="4.7"/><rect x="2.3" y="9" width="4.7" height="4.7"/><rect x="9" y="9" width="4.7" height="4.7"/></svg>{/snippet}
{#snippet iconLayout()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><rect x="2" y="2.3" width="12" height="3.2"/><rect x="2" y="7" width="12" height="6.7"/></svg>{/snippet}
{#snippet iconSidebar()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><rect x="2" y="2" width="12" height="12" rx="1"/><path d="M6 2v12"/></svg>{/snippet}
{#snippet iconClock()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><path d="M8 4.6V8l2.8 1.8"/></svg>{/snippet}
{#snippet iconSettings()}<svg class="icon" aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>{/snippet}
{#snippet iconSparkle()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="currentColor" stroke="none"><path d="M8 1.4c.45 2.85 1.85 4.25 4.6 4.6-2.75.45-4.15 1.85-4.6 4.6-.45-2.75-1.85-4.15-4.6-4.6C6.15 5.65 7.55 4.25 8 1.4Z"/></svg>{/snippet}
{#snippet iconInboxEmpty()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><path d="M2 9.3 4.4 3h7.2L14 9.3"/><path d="M2 9.3v3.4h12V9.3h-3.1a2.2 2.2 0 0 1-4.4 0H2Z"/></svg>{/snippet}
{#snippet iconFileText()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><path d="M4 2h5.4L12 4.6V14H4Z"/><path d="M9.4 2v2.6H12"/><path d="M6 8.2h4M6 10.6h4"/></svg>{/snippet}
{#snippet iconSave()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"><path d="M13.5 13.5H2.5v-11h8l3 3v8z"/><path d="M4 2.5v4h5v-4"/><path d="M11.5 13.5v-4h-7v4"/></svg>{/snippet}
{#snippet iconStar(filled: boolean)}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill={filled ? "currentColor" : "none"} stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"><path d="M8 2.1l1.8 3.7 4 .6-2.9 2.8.7 4-3.6-1.9-3.6 1.9.7-4-2.9-2.8 4-.6Z"/></svg>{/snippet}
{#snippet iconChevronRight()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M6 3.3 11 8l-5 4.7"/></svg>{/snippet}
{#snippet iconChevronLeft()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M10 3.3 5 8l5 4.7"/></svg>{/snippet}
{#snippet iconChevronDown()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M3.3 6 8 11l4.7-5"/></svg>{/snippet}
{#snippet iconChevronUp()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M3.3 10 8 5l4.7 5"/></svg>{/snippet}
{#snippet iconExpandAll()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><rect x="2.4" y="2.4" width="11.2" height="11.2"/><path d="M8 5v6M5 8h6"/></svg>{/snippet}
{#snippet iconCollapseAll()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><rect x="2.4" y="2.4" width="11.2" height="11.2"/><path d="M5 8h6"/></svg>{/snippet}
{#snippet iconCopy()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><rect x="5.5" y="5.5" width="8" height="8"/><path d="M10.5 5.5V3H3v8h2.5"/></svg>{/snippet}
{#snippet iconExpandDiagonal()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M9.5 2.5h4v4M13.5 2.5 8.8 7.2"/><path d="M6.5 13.5h-4v-4M2.5 13.5 7.2 8.8"/></svg>{/snippet}
{#snippet iconCompressDiagonal()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M14 6.5h-4v-4M10 6.5l4.5-4.5"/><path d="M2 9.5h4v4M6 9.5 1.5 14"/></svg>{/snippet}
{#snippet iconMinus()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M3 8h10"/></svg>{/snippet}
{#snippet iconArrowUp()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M8 12.5V4M4.3 7.7 8 4l3.7 3.7"/></svg>{/snippet}
{#snippet iconArrowDown()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"><path d="M8 3.5V12M4.3 8.3 8 12l3.7-3.7"/></svg>{/snippet}
{#snippet railScreenIcon(id: string)}
  {#if id === "workspace"}{@render iconLayout()}
  {:else if id === "environments"}{@render iconGlobe()}
  {:else if id === "git"}{@render iconGitBranch()}
  {:else if id === "launcher"}{@render iconGrid()}
  {:else if id === "history"}{@render iconClock()}
  {:else if id === "settings"}{@render iconSettings()}
  {/if}
{/snippet}
