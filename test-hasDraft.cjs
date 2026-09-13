const fs = require('fs');
const content = fs.readFileSync('src/routes/+page.svelte', 'utf8');

const regex = /function\s+([a-zA-Z0-9_]+)\s*\([^)]*\)\s*\{[^}]*tabDrafts\.has[^}]*\}/g;
let match;
while ((match = regex.exec(content)) !== null) {
  console.log("Found function:", match[1]);
}
