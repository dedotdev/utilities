import * as fs from 'fs';
import * as path from 'path';

const filesToCopy = ['package.json', 'README.md', 'LICENSE'];
const targetDir = 'dist';

const main = () => {
  if (!fs.existsSync(targetDir)) {
    return;
  }

  const currentDir = process.cwd();

  filesToCopy.forEach((file) => {
    let filePath = path.resolve(currentDir, file);

    if (!fs.existsSync(filePath)) {
      return;
    }

    let fileContent = fs.readFileSync(filePath, { encoding: 'utf8' });

    if (file === 'package.json') {
      const pkgJson = JSON.parse(fileContent);

      if (!['@dedot/wasm'].includes(pkgJson.name)) {
        pkgJson.main = '';
        pkgJson.module = './index.js';
        pkgJson.types = './index.d.ts';

        // Special handling for @dedot/chain-specs package
        if (pkgJson.name === '@dedot/chain-specs') {
          pkgJson.main = './cjs/index.js';

          const specsDir = path.join(currentDir, 'specs');

          if (fs.existsSync(specsDir)) {
            const specFiles = fs.readdirSync(specsDir)
              .filter(file => file.endsWith('.json'))
              .map(file => file.slice(0, -5)); // Remove .json extension

            // Generate exports field
            const exports: Record<string, any> = {
              '.': {
                types: './index.d.ts',
                import: './index.js',
                require: './cjs/index.js',
                default: './index.js'
              }
            };

            // Add export for each spec file
            specFiles.forEach(specName => {
              exports[`./${specName}`] = {
                types: `./specs/${specName}.d.ts`,
                import: `./specs/${specName}.js`,
                require: `./cjs/specs/${specName}.js`,
                default: `./specs/${specName}.js`
              };
            });

            pkgJson.exports = exports;
          }
        }

        fileContent = JSON.stringify(pkgJson, null, 2);
      }
    }

    fs.writeFileSync(path.join(currentDir, targetDir, file), fileContent);
  });
};

main();
