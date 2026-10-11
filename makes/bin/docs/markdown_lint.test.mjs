import assert from 'node:assert/strict';
import { mkdir, mkdtemp, rm, writeFile, symlink } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { markdownFiles, lintMarkdown } from './markdown_lint.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const artifacts = path.join(root, 'artifacts/docs/markdownlint-tests');
const nodeRoot = process.env.BIJUX_DOCS_NODE_ROOT || path.join(root, 'configs/sources/repository/docs');
async function fixture(t) {
  await mkdir(artifacts, { recursive: true });
  const repo = await mkdtemp(path.join(artifacts, 'repository-'));
  t.after(() => rm(repo, { recursive: true, force: true }));
  await mkdir(path.join(repo, 'docs'), { recursive: true });
  await mkdir(path.join(repo, 'ops'), { recursive: true });
  await mkdir(path.join(repo, 'configs/sources/repository/docs'), { recursive: true });
  await writeFile(path.join(repo, 'configs/sources/repository/docs/markdownlint.json'), '{}\n');
  return repo;
}

test('discovers nested documentation and operations without including retired generated surfaces', async t => {
  const repo = await fixture(t);
  for (const name of ['docs/index.md', 'docs/handbook/start.md', 'ops/runbooks/recovery.md', 'docs/_drafts/draft.md', 'docs/_generated/report.md', 'docs/_nav/index.md', 'ops/_generated.example/sample.md']) {
    await mkdir(path.dirname(path.join(repo, name)), { recursive: true });
    await writeFile(path.join(repo, name), '# Document\n');
  }
  await writeFile(path.join(repo, 'docs/data.json'), '{}\n');
  assert.deepEqual(await markdownFiles(repo), ['docs/handbook/start.md', 'docs/index.md', 'ops/runbooks/recovery.md']);
});

test('rejects symlinked Markdown sources crossing repository ownership', async t => {
  const repo = await fixture(t);
  await writeFile(path.join(repo, 'outside.md'), '# External\n');
  await symlink(path.join(repo, 'outside.md'), path.join(repo, 'docs/external.md'));
  await assert.rejects(markdownFiles(repo), /must not be symlinked/);
});

test('applies the declared lint rules to both documentation and operations', async t => {
  const repo = await fixture(t);
  await writeFile(path.join(repo, 'docs/index.md'), '# Valid\n\nText.\n');
  await writeFile(path.join(repo, 'ops/invalid.md'), '#Missing space\n');
  const { results } = await lintMarkdown(repo, nodeRoot);
  assert.deepEqual(results[path.join(repo, 'docs/index.md')], []);
  assert(results[path.join(repo, 'ops/invalid.md')].some(d => d.ruleNames.includes('MD018')));
  await writeFile(path.join(repo, 'configs/sources/repository/docs/markdownlint.json'), '{"MD018":false,"MD041":false}\n');
  const configured = await lintMarkdown(repo, nodeRoot);
  assert.deepEqual(configured.results[path.join(repo, 'ops/invalid.md')], []);
});

test('treats brace syntax in file names as literal paths', async t => {
  const repo = await fixture(t);
  await writeFile(path.join(repo, 'docs/{guide,reference}.md'), '# Literal\n\nText.\n');
  const { files, results } = await lintMarkdown(repo, nodeRoot);
  assert.deepEqual(files, ['docs/{guide,reference}.md']);
  assert.deepEqual(Object.values(results), [[]]);
});
