const fs = require('fs');
let content = fs.readFileSync('src/routes/+page.svelte', 'utf8');

// 1. Add `isTabDirty(tab.id)` circle to tabs
// We have two places for tabs: `request-tabs-bar` and `tab-overflow-list`.
content = content.replace(
  /<span class="tab-name">\{tab\.name\}<\/span>/g,
  `<span class="tab-name">{tab.name}</span>\n                        {#if isTabDirty(tab.id)}\n                          <span class="unsaved-indicator"></span>\n                        {/if}`
);

// 2. Change `closeTab(tab.id)` to `closeTabAction(tab.id)` inside the tab bar
// The close buttons have: onclick={(e) => { e.stopPropagation(); closeTab(tab.id); }}
content = content.replace(
  /closeTab\(tab\.id\)/g,
  `closeTabAction(tab.id)`
);

// 3. Change `closeTab(id)` in deleteRequest to `closeTabAction(id)`? No, `deleteRequest` shouldn't warn about unsaved changes. It's already deleting the request!
content = content.replace(
  /function deleteRequest\(id: string\) \{[\s\S]*?closeTabAction\(id\);/g, // if it matched
  (match) => match.replace("closeTabAction(id)", "closeTab(id)")
);

// 4. Implement closeTabAction
content = content.replace(
  /async function closeTab\(id: string\) \{/,
  `function closeTabAction(id: string) {
    if (isTabDirty(id)) {
      showConfirm(t("common.confirm"), t("error.confirmCloseDirtyTab") || "You have unsaved changes. Are you sure you want to close without saving?", () => {
        // discard changes
        if (id === selectedRequest?.id && autoSaveTimer) {
          clearTimeout(autoSaveTimer);
          autoSaveTimer = null;
          autoSaveStatus = "saved";
        }
        tabDrafts.delete(id);
        closeTab(id, true);
      });
    } else {
      closeTab(id, false);
    }
  }

  async function closeTab(id: string, skipSave: boolean = false) {`
);

// 5. In closeTab, skip saving if skipSave is true
content = content.replace(
  /await saveRequest\(\);\s*selectedRequest = null;/g,
  `if (!skipSave) await saveRequest();\n          selectedRequest = null;`
);

fs.writeFileSync('src/routes/+page.svelte', content);
