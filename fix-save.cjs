const fs = require('fs');
let content = fs.readFileSync('src/routes/+page.svelte', 'utf8');

content = content.replace(
  /async function saveCurrentResponse\(\) \{[\s\S]*?const defaultName = \`Example - \$\{activeResponse\.status\}\`;[\s\S]*?const name = window\.prompt\("Enter a name for this example:", defaultName\);[\s\S]*?if \(name === null\) return;[\s\S]*?try \{[\s\S]*?const created = await api\.createSampleResponse\(\{[\s\S]*?request_id: selectedRequest\.id,[\s\S]*?name: name \|\| defaultName,[\s\S]*?status: activeResponse\.status,[\s\S]*?status_text: activeResponse\.status_text \|\| "",[\s\S]*?headers: activeResponse\.headers \?\? \[\],[\s\S]*?body: activeResponseBody,[\s\S]*?content_type: activeResponse\.content_type \|\| null[\s\S]*?\}\);[\s\S]*?await loadSampleResponses\(selectedRequest\.id\);/m,
  `async function saveCurrentResponse() {
    if (!selectedRequest || !activeResponse || !activeResponseBody) return;
    const defaultName = \`Example - \${activeResponse.status}\`;
    try {
      const created = await api.createSampleResponse({
        request_id: selectedRequest.id,
        name: defaultName,
        status: activeResponse.status,
        status_text: activeResponse.status_text || "",
        headers: activeResponse.headers ?? [],
        body: activeResponseBody,
        content_type: activeResponse.content_type || null
      });
      await loadSampleResponses(selectedRequest.id);
      
      // Auto-trigger rename and open docs pane
      startRenameSampleResponse(created);
      setRightSidebarVisible(true);
      rightPanel = "docs";`
);

fs.writeFileSync('src/routes/+page.svelte', content);
