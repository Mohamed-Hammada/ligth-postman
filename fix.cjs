const fs = require('fs');
let content = fs.readFileSync('src/routes/+page.svelte', 'utf8');

// 1. Delete Confirmations
content = content.replace(/async function deleteProject\(id: string\) \{/g, `async function deleteProject(id: string) {\n    if (!confirm(t("error.confirmDeleteProject") || "Are you sure?")) return;`);
content = content.replace(/async function deleteRequest\(id: string\) \{/g, `async function deleteRequest(id: string) {\n    if (!confirm(t("error.confirmDeleteRequest") || "Are you sure?")) return;`);
content = content.replace(/async function deleteFolderAction\(id: string\) \{/g, `async function deleteFolderAction(id: string) {\n    if (!confirm(t("error.confirmDeleteFolder") || "Are you sure?")) return;`);
content = content.replace(/async function deleteEnvironmentAction\(id: string\) \{/g, `async function deleteEnvironmentAction(id: string) {\n    if (!confirm(t("error.confirmDeleteEnvironment") || "Are you sure?")) return;`);
content = content.replace(/async function deleteVariable\(id: string\) \{/g, `async function deleteVariable(id: string) {\n    if (!confirm(t("error.confirmDeleteVariable") || "Are you sure?")) return;`);
content = content.replace(/async function deleteSampleResponseAction\(id: string, requestId: string\) \{/g, `async function deleteSampleResponseAction(id: string, requestId: string) {\n    if (!confirm(t("error.confirmDeleteSampleResponse") || "Are you sure?")) return;`);

// 2. Workspace Export Action
const exportAction = `
  async function exportWorkspaceAction(format: 'light-postman' | 'postman') {
    try {
      const dirHandle = await (window as any).showDirectoryPicker();
      for (const proj of projects) {
        const data = await api.exportProject(proj.id, format);
        const ext = format === 'postman' ? 'postman_collection.json' : 'json';
        const fileHandle = await dirHandle.getFileHandle(\`\${proj.name.replace(/[^a-z0-9]/gi, '_')}.\${ext}\`, { create: true });
        const writable = await fileHandle.createWritable();
        await writable.write(data);
        await writable.close();
      }
      exportFeedback = "Export successful!";
      setTimeout(() => (exportFeedback = ""), 3000);
    } catch (e) {
      errorMessage = describeError(e);
    }
  }
`;
content = content.replace(/async function exportProjectAction\(format: "postman" \| "light-postman"\) \{/, exportAction + '\n  async function exportProjectAction(format: "postman" | "light-postman") {');

// Add workspace export buttons to markup
content = content.replace(
  /<div class="projectfile-options">\s*<button[^>]*>Light-Postman Format<\/button>\s*<button[^>]*>Postman Format<\/button>\s*<\/div>/,
  `<div class="projectfile-options">
      <button type="button" class="btn-ghost" onclick={() => exportWorkspaceAction('light-postman')} disabled={projects.length === 0}>Export All (Light-Postman)</button>
      <button type="button" class="btn-ghost" onclick={() => exportWorkspaceAction('postman')} disabled={projects.length === 0}>Export All (Postman)</button>
    </div>
    <div class="projectfile-options">
      <button type="button" class="btn-ghost" onclick={() => exportProjectAction("light-postman")} disabled={!selectedProjectId}>{t("git.exportLightPostman")}</button>
      <button type="button" class="btn-ghost" onclick={() => exportProjectAction("postman")} disabled={!selectedProjectId}>{t("git.exportPostman")}</button>
    </div>`
);

// 3. downloadFile with native dialog
content = content.replace(
  /function downloadFile\(filename: string, content: string\) \{[\s\S]*?\n  \}/,
  `async function downloadFile(filename: string, content: string) {
    try {
      const handle = await (window as any).showSaveFilePicker({ suggestedName: filename });
      const writable = await handle.createWritable();
      await writable.write(content);
      await writable.close();
    } catch (e) {
      // fallback
      const blob = new Blob([content], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }
  }`
);

// 4. saveCurrentResponse payload and prompt
content = content.replace(
  /async function saveCurrentResponse\(\) \{\s*if \(\!selectedRequest \|\| \!activeResponse \|\| \!activeResponseBody\) return;\s*try \{/,
  `async function saveCurrentResponse() {
    if (!selectedRequest || !activeResponse || !activeResponseBody) return;
    const defaultName = \`Example - \${activeResponse.status}\`;
    const name = window.prompt("Enter a name for this example:", defaultName);
    if (name === null) return;
    try {`
);
content = content.replace(
  /name: \`Example - \$\{activeResponse\.status\}\`,/,
  `name: name || defaultName,`
);
content = content.replace(
  /status: activeResponse\.status,\s*headers: activeResponse\.headers \?\? \[\],\s*body: activeResponseBody/,
  `status: activeResponse.status,\n        status_text: activeResponse.status_text || "",\n        headers: activeResponse.headers ?? [],\n        body: activeResponseBody,\n        content_type: activeResponse.content_type || null`
);

// 5. Swap Send and Save buttons
content = content.replace(
  /\{#if activeResponse\}\s*<button type="button" class="btn-ghost" onclick=\{saveCurrentResponse\} title="Save current response as an example" style="padding: 0 0.5rem; border: 1px solid var\(--color-border\); border-radius: var\(--radius-sm\); font-size: var\(--text-sm\);">Save Response<\/button>\s*\{\/if\}\s*<button type="button" class="btn-send" onclick=\{sendCurrentRequest\}>\{t\("request.send"\)\}<\/button>/,
  `<button type="button" class="btn-send" onclick={sendCurrentRequest}>{t("request.send")}</button>
                  {#if activeResponse}
                    <button type="button" class="btn-ghost" onclick={saveCurrentResponse} title="Save current response as an example" style="padding: 0 0.5rem; border: 1px solid var(--color-border); border-radius: var(--radius-sm); font-size: var(--text-sm);">{@render iconSave()} Save</button>
                  {/if}`
);

// 6. body-mode-bar Beautify
content = content.replace(
  /<div class="body-mode-bar">\s*<label class="radio-label">\s*<input type="radio" bind:group=\{editBodyType\} value="raw"[^>]*> \{t\("body.raw"\)\}\s*<\/label>\s*<label class="radio-label">\s*<input type="radio" bind:group=\{editBodyType\} value="form-data"[^>]*> \{t\("body.formData"\)\}\s*<\/label>\s*<label class="radio-label">\s*<input type="radio" bind:group=\{editBodyType\} value="x-www-form-urlencoded"[^>]*> \{t\("body.urlEncoded"\)\}\s*<\/label>\s*<label class="radio-label">\s*<input type="radio" bind:group=\{editBodyType\} value="binary"[^>]*> \{t\("body.binary"\)\}\s*<\/label>\s*<label class="radio-label">\s*<input type="radio" bind:group=\{editBodyType\} value="graphql"[^>]*> \{t\("body.graphql"\)\}\s*<\/label>\s*\{#if editBodyType === "raw"\}\s*<select class="raw-type-select"[^>]*>\s*\{#each RAW_CONTENT_TYPES as type \(type.id\)\}\s*<option value=\{type.id\}>\{t\(type.label\)\}<\/option>\s*\{\/each\}\s*<\/select>\s*\{#if rawContentType === "json"\}\s*<button type="button" onclick=\{[^}]*\}>\{t\("body.jsonTemplate"\)\}<\/button>\s*\{\/if\}\s*<button type="button" onclick=\{[^}]*\}>\{t\("body.clearBody"\)\}<\/button>\s*<button type="button" onclick=\{prettifyBody\}>\{t\("body.prettify"\)\}<\/button>\s*\{#if bodyPrettifyFeedback\}<span class="warn-inline">\{bodyPrettifyFeedback\}<\/span>\{\/if\}\s*\{\/if\}\s*<\/div>/,
  `<div class="body-mode-bar">
                    <label class="radio-label">
                      <input type="radio" bind:group={editBodyType} value="raw" onchange={scheduleAutoSave} /> {t("body.raw")}
                    </label>
                    <label class="radio-label">
                      <input type="radio" bind:group={editBodyType} value="form-data" onchange={scheduleAutoSave} /> {t("body.formData")}
                    </label>
                    <label class="radio-label">
                      <input type="radio" bind:group={editBodyType} value="x-www-form-urlencoded" onchange={scheduleAutoSave} /> {t("body.urlEncoded")}
                    </label>
                    <label class="radio-label">
                      <input type="radio" bind:group={editBodyType} value="binary" onchange={scheduleAutoSave} /> {t("body.binary")}
                    </label>
                    <label class="radio-label">
                      <input type="radio" bind:group={editBodyType} value="graphql" onchange={scheduleAutoSave} /> {t("body.graphql")}
                    </label>

                    <div style="flex:1"></div>

                    {#if editBodyType === "raw"}
                      <select class="raw-type-select" value={rawContentType} onchange={(e) => setRawContentType((e.target as HTMLSelectElement).value)}>
                        {#each RAW_CONTENT_TYPES as type (type.id)}
                          <option value={type.id}>{t(type.label)}</option>
                        {/each}
                      </select>
                      {#if rawContentType === "json"}
                        <button type="button" class="btn-ghost btn-xs" onclick={() => { if (!editBody) editBody = "{\\n  \\n}"; scheduleAutoSave(); }}>{t("body.jsonTemplate")}</button>
                      {/if}
                    {/if}
                    
                    {#if editBodyType === "raw" || editBodyType === "graphql"}
                      <button type="button" class="btn-ghost btn-xs" onclick={() => { editBody = ""; editGraphqlVariables = ""; scheduleAutoSave(); }}>{t("body.clearBody")}</button>
                      <button type="button" class="btn-ghost btn-xs" onclick={prettifyBody}>{t("body.prettify")}</button>
                      {#if bodyPrettifyFeedback}<span class="warn-inline">{bodyPrettifyFeedback}</span>{/if}
                    {/if}
                  </div>`
);

// 7. prettifyBody logic
content = content.replace(
  /function prettifyBody\(\) \{\s*if \(rawContentType === "json"\)/,
  `function prettifyBody() {
    if (editBodyType === "graphql") {
      try {
        if (editGraphqlVariables && editGraphqlVariables.trim() !== "") {
          editGraphqlVariables = JSON.stringify(JSON.parse(editGraphqlVariables), null, 2);
        }
        bodyPrettifyFeedback = "";
        scheduleAutoSave();
      } catch {
        bodyPrettifyFeedback = "Invalid JSON in Variables";
        setTimeout(() => (bodyPrettifyFeedback = ""), 2500);
      }
      return;
    }
    if (rawContentType === "json")`
);

// 8. iconSave
content = content.replace(
  /\{#snippet iconFileText\(\)\}<svg class="icon"[^>]*>[\s\S]*?<\/svg>\{\/snippet\}/,
  `{#snippet iconFileText()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="miter"><path d="M4 2h5.4L12 4.6V14H4Z"/><path d="M9.4 2v2.6H12"/><path d="M6 8.2h4M6 10.6h4"/></svg>{/snippet}\n{#snippet iconSave()}<svg class="icon" aria-hidden="true" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"><path d="M13.5 13.5H2.5v-11h8l3 3v8z"/><path d="M4 2.5v4h5v-4"/><path d="M11.5 13.5v-4h-7v4"/></svg>{/snippet}`
);

// 9. openTabs $effect
content = content.replace(
  /let openTabs = \$state<RequestTab\[\]>\(\[\]\);\s*const tabDrafts = new Map<string, RequestDraft>\(\);/,
  `let openTabs = $state<RequestTab[]>([]);
  const tabDrafts = new Map<string, RequestDraft>();

  $effect(() => {
    if (selectedProjectId) {
      try {
        localStorage.setItem(\`lp-open-tabs-\${selectedProjectId}\`, JSON.stringify(openTabs));
        if (selectedRequest) {
          localStorage.setItem(\`lp-selected-request-\${selectedProjectId}\`, selectedRequest.id);
        } else {
          localStorage.removeItem(\`lp-selected-request-\${selectedProjectId}\`);
        }
      } catch {}
    }
  });`
);

// 10. selectProject openTabs restore
content = content.replace(
  /async function selectProject\(id: string\) \{\s*await saveRequest\(\);\s*selectedProjectId = id;\s*selectedRequest = null;\s*selectedEnvironmentId = null;\s*openTabs = \[\];\s*tabDrafts\.clear\(\);/,
  `async function selectProject(id: string) {
    await saveRequest();
    selectedProjectId = id;
    selectedRequest = null;
    selectedEnvironmentId = null;
    openTabs = [];
    tabDrafts.clear();
    try {
      const savedTabs = localStorage.getItem(\`lp-open-tabs-\${id}\`);
      if (savedTabs) openTabs = JSON.parse(savedTabs);
    } catch {}`
);

content = content.replace(
  /requests = await api\.listRequests\(id\);\s*folders = await api\.listFolders\(id\);/,
  `requests = await api.listRequests(id);
      folders = await api.listFolders(id);
      try {
        const savedReqId = localStorage.getItem(\`lp-selected-request-\${id}\`);
        if (savedReqId && requests.some(r => r.id === savedReqId)) {
          openRequest(savedReqId).catch(console.error);
        }
      } catch {}`
);

fs.writeFileSync('src/routes/+page.svelte', content);
