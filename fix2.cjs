const fs = require('fs');
let content = fs.readFileSync('src/routes/+page.svelte', 'utf8');

// Replace standard confirm with custom modal function showConfirm(...)
content = content.replace(
  /async function deleteProject\(id: string\) \{\s*if \(\!confirm\(t\("error\.confirmDeleteProject"\) \|\| "Are you sure\?"\)\) return;/g,
  `function deleteProject(id: string) {\n    showConfirm(t("common.confirm"), t("error.confirmDeleteProject"), async () => {`
);

content = content.replace(
  /async function deleteRequest\(id: string\) \{\s*if \(\!confirm\(t\("error\.confirmDeleteRequest"\) \|\| "Are you sure\?"\)\) return;/g,
  `function deleteRequest(id: string) {\n    showConfirm(t("common.confirm"), t("error.confirmDeleteRequest"), async () => {`
);

content = content.replace(
  /async function deleteFolderAction\(id: string\) \{\s*if \(\!confirm\(t\("error\.confirmDeleteFolder"\) \|\| "Are you sure\?"\)\) return;/g,
  `function deleteFolderAction(id: string) {\n    showConfirm(t("common.confirm"), t("error.confirmDeleteFolder"), async () => {`
);

content = content.replace(
  /async function deleteEnvironmentAction\(id: string\) \{\s*if \(\!confirm\(t\("error\.confirmDeleteEnvironment"\) \|\| "Are you sure\?"\)\) return;/g,
  `function deleteEnvironmentAction(id: string) {\n    showConfirm(t("common.confirm"), t("error.confirmDeleteEnvironment"), async () => {`
);

content = content.replace(
  /async function deleteVariable\(id: string\) \{\s*if \(\!confirm\(t\("error\.confirmDeleteVariable"\) \|\| "Are you sure\?"\)\) return;/g,
  `function deleteVariable(id: string) {\n    showConfirm(t("common.confirm"), t("error.confirmDeleteVariable"), async () => {`
);

content = content.replace(
  /async function deleteSampleResponseAction\(id: string, requestId: string\) \{\s*if \(\!confirm\(t\("error\.confirmDeleteSampleResponse"\) \|\| "Are you sure\?"\)\) return;/g,
  `function deleteSampleResponseAction(id: string, requestId: string) {\n    showConfirm(t("common.confirm"), t("error.confirmDeleteSampleResponse"), async () => {`
);

// We need to close the `});` for each function. We can find the end of the function.
const funcs = [
  "deleteProject",
  "deleteRequest",
  "deleteFolderAction",
  "deleteEnvironmentAction",
  "deleteVariable",
  "deleteSampleResponseAction"
];
for (const func of funcs) {
  // Find where the function ends
  const idx = content.indexOf(`function ${func}(`);
  if (idx !== -1) {
    let openBraces = 0;
    let foundBrace = false;
    let i = idx;
    for (; i < content.length; i++) {
      if (content[i] === '{') {
        openBraces++;
        foundBrace = true;
      } else if (content[i] === '}') {
        openBraces--;
      }
      if (foundBrace && openBraces === 0) {
        break;
      }
    }
    // Now insert `});` right before the closing brace
    content = content.slice(0, i) + "  });\n" + content.slice(i);
  }
}

// Add state and modal UI
content = content.replace(/<\/script>/, `
  let confirmDialog = $state<{
    show: boolean;
    title: string;
    message: string;
    onConfirm: () => void;
  }>({
    show: false,
    title: "",
    message: "",
    onConfirm: () => {}
  });

  function showConfirm(title: string, message: string, onConfirm: () => void) {
    confirmDialog = { show: true, title, message, onConfirm };
  }
</script>

{#if confirmDialog.show}
  <div
    class="modal-backdrop"
    onclick={(e) => { if (e.target === e.currentTarget) confirmDialog.show = false; }}
    onkeydown={(e) => { if (e.key === "Escape") confirmDialog.show = false; }}
    role="dialog"
    aria-modal="true"
    tabindex="0"
  >
    <div class="modal-container">
      <div class="modal-header">
        <h3>{confirmDialog.title}</h3>
        <button type="button" class="modal-close-btn" title={t("common.close")} onclick={() => (confirmDialog.show = false)}>{@render iconClose()}</button>
      </div>
      <div class="modal-body">
        <p>{confirmDialog.message}</p>
      </div>
      <div class="modal-footer" style="display:flex; justify-content: flex-end; gap: 0.5rem;">
        <button type="button" onclick={() => (confirmDialog.show = false)}>{t("request.cancel")}</button>
        <button type="button" class="btn-primary" onclick={() => { confirmDialog.show = false; confirmDialog.onConfirm(); }}>{t("common.confirm")}</button>
      </div>
    </div>
  </div>
{/if}
`);

fs.writeFileSync('src/routes/+page.svelte', content);
