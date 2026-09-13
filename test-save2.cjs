const fs = require('fs');
let content = fs.readFileSync('src/routes/+page.svelte', 'utf8');

const regex = /async function saveCurrentResponse\(\) \{[\s\S]*?errorMessage = describeError\(err\);\s*\}/;
console.log(content.match(regex)?.[0]);
