const fs = require('fs');
let content = fs.readFileSync('src/routes/+page.svelte', 'utf8');

content = content.replace(
  /async function deleteSampleResponseAction\(requestId: string, id: string\) \{\s*try \{/g,
  `function deleteSampleResponseAction(requestId: string, id: string) {\n    showConfirm(t("common.confirm"), t("error.confirmDeleteSampleResponse"), async () => {\n      try {`
);

const idx = content.indexOf(`function deleteSampleResponseAction(`);
if (idx !== -1) {
  let openBraces = 0;
  let inString = false;
  let stringChar = '';
  let started = false;
  let i = idx;
  for (; i < content.length; i++) {
    const char = content[i];
    if (inString) {
      if (char === stringChar && content[i-1] !== '\\') inString = false;
    } else {
      if (char === '"' || char === "'" || char === '`') {
        inString = true;
        stringChar = char;
      } else if (char === '{') {
        openBraces++;
        started = true;
      } else if (char === '}') {
        openBraces--;
        if (started && openBraces === 1) { // We are at the end of the async () => { block
          content = content.slice(0, i) + "  });\n  }" + content.slice(i + 1);
          break;
        }
      }
    }
  }
}

fs.writeFileSync('src/routes/+page.svelte', content);
