const fs = require('fs');
const content = fs.readFileSync('src/routes/+page.svelte', 'utf8');

const lines = content.split('\n');
lines.forEach((line, i) => {
  if (line.includes('tabDrafts.has')) {
    console.log(`Line ${i + 1}: ${line}`);
  }
});
