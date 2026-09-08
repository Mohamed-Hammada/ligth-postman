const fs = require('fs');
let content = fs.readFileSync('src/routes/+page.svelte', 'utf8');

const funcs = [
  "deleteProject",
  "deleteRequest",
  "deleteFolderAction",
  "deleteEnvironmentAction",
  "deleteVariable",
  "deleteSampleResponseAction"
];

for (const func of funcs) {
  const funcRegex = new RegExp(`function ${func}\\(.*?\\) \\{\\s*showConfirm\\(.*?async \\(\\) => \\{`);
  let match = content.match(funcRegex);
  if (!match) continue;

  const startIndex = match.index + match[0].length;
  
  // See if there's already `});` before the function's closing brace.
  // We can just parse braces
  let openBraces = 2; // function { and async () => {
  let i = startIndex;
  let inString = false;
  let stringChar = '';
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
      } else if (char === '}') {
        openBraces--;
        if (openBraces === 1) {
          // Found the end of `async () => {`
          // Insert `});` here.
          // But wait! If the `}` we just hit is the very last one of the function (because we are missing `}`), then openBraces will go to 1 at the END of the function.
          // Let's check if `});` is already there.
          const before = content.slice(i - 4, i);
          if (!before.includes("});")) {
            content = content.slice(0, i) + "  });\n  }" + content.slice(i + 1);
          }
          break;
        }
      }
    }
  }
}
fs.writeFileSync('src/routes/+page.svelte', content);
