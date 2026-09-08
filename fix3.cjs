const fs = require('fs');
let content = fs.readFileSync('src/routes/+page.svelte', 'utf8');

content = content.replace(/async function deleteSampleResponseAction\(requestId: string, id: string\) \{\s*try \{/g, `function deleteSampleResponseAction(requestId: string, id: string) {\n  showConfirm(t("common.confirm"), t("error.confirmDeleteSampleResponse"), async () => {\n    try {`);

fs.writeFileSync('src/routes/+page.svelte', content);
