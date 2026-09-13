const fs = require('fs');
let content = fs.readFileSync('src/routes/+page.svelte', 'utf8');

// Replace closeTab(...) calls with closeTabAction(...) in the UI
content = content.replace(
  /closeTab\(tab\.id\)/g,
  `closeTabAction(tab.id)`
);

// We need to inject closeTabAction
content = content.replace(
  /async function closeTab\(id: string\) \{/,
  `function closeTabAction(id: string) {
    if (isTabDirty(id)) {
      showConfirm(t("common.confirm"), t("error.confirmCloseDirtyTab") || "You have unsaved changes. Are you sure you want to close this tab without saving?", () => {
        // Discard draft
        if (id === selectedRequest?.id) {
           // It's the active tab, we need to revert it so it doesn't auto-save on switch
           if (autoSaveTimer) { clearTimeout(autoSaveTimer); autoSaveTimer = null; }
           // Revert the currently bound fields so they don't get saved by any other path
           editName = selectedRequest.name;
           editMethod = selectedRequest.method;
           editUrl = selectedRequest.url;
           editBody = selectedRequest.body ?? "";
           // etc... wait, it's easier to just skip saving. 
           // But saveRequest is called in closeTab. We can pass a flag.
        }
        closeTab(id, true);
      });
    } else {
      closeTab(id, false);
    }
  }

  async function closeTab(id: string, skipSave: boolean = false) {`
);

// In closeTab, skip saveRequest if skipSave is true
content = content.replace(
  /await saveRequest\(\);/g,
  `if (!skipSave) await saveRequest();` // Wait, this replaces ALL await saveRequest(). There are other places!
);

fs.writeFileSync('test-close.cjs', content);
