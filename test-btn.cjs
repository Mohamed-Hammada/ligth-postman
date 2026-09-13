const fs = require('fs');
const css = fs.readFileSync('src/app.css', 'utf8');
const lines = css.split('\n');
lines.forEach(l => {
  if (l.includes('.btn')) console.log(l.trim());
});
