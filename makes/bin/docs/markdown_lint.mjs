import { readdir, readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const excluded = new Set(['docs/_generated', 'docs/_drafts', 'docs/_nav', 'ops/_generated.example']);

// The repository owns a finite Markdown surface; patterns never reach a glob parser.
export async function markdownFiles(root) {
  const files = [];
  async function visit(relative) {
    const directory = path.join(root, relative);
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const name = `${relative}/${entry.name}`;
      if (excluded.has(name)) continue;
      if (entry.isSymbolicLink()) throw new Error(`Markdown source must not be symlinked: ${name}`);
      if (entry.isDirectory()) await visit(name);
      else if (entry.isFile() && name.endsWith('.md')) files.push(name);
    }
  }
  await visit('docs');
  await visit('ops');
  return files.sort();
}

export async function lintMarkdown(root, nodeRoot) {
  const files = await markdownFiles(root);
  const require = createRequire(path.join(nodeRoot, 'package.json'));
  const { lint } = await import(pathToFileURL(require.resolve('markdownlint/promise')).href);
  const config = JSON.parse(await readFile(path.join(root, 'configs/sources/repository/docs/markdownlint.json'), 'utf8'));
  const results = await lint({ files: files.map(name => path.join(root, name)), config });
  return { files, results };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const nodeRoot = process.env.BIJUX_DOCS_NODE_ROOT || path.join(repositoryRoot, 'configs/sources/repository/docs');
  const { files, results } = await lintMarkdown(repositoryRoot, nodeRoot);
  let errors = 0;
  for (const [file, diagnostics] of Object.entries(results)) {
    for (const diagnostic of diagnostics) {
      errors++;
      console.error(`${path.relative(repositoryRoot, file)}:${diagnostic.lineNumber}: ${diagnostic.ruleNames[0]} ${diagnostic.ruleDescription}`);
    }
  }
  console.log(`Markdown lint: ${files.length} files, ${errors} diagnostics`);
  if (errors) process.exitCode = 1;
}
