import { fileURLToPath } from 'url';
import * as path from 'path';
import * as fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const specsDir = path.join(__dirname, '../specs');
const jsDir = path.join(__dirname, '../src/specs/');

try {
  const files = fs.readFileSync(specsDir);
  const jsonFiles = files.filter((file) => file.endsWith('.json'));

  if (!fs.existsSync(jsDir)) {
    fs.mkdirSync(jsDir);
  }

  jsonFiles.map((file) => {
    const jsonContent = fs.readFileSync(path.join(specsDir, file), {
      encoding: 'utf8',
    });

    const jsContent = `export const chainSpec: string = \`${jsonContent}\``;
    fs.writeFileSync(path.join(jsDir, file.slice(0, -4) + 'ts'), jsContent);
  });
} catch (e) {
  console.log('There was an error creating the js specs');
  console.error(e);
  process.exit(1);
}
