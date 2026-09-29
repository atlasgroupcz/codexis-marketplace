#!/usr/bin/env node
// Drives the dev daemon (https://vm.dev.internal.codexis.ai) for the developer-flow skill:
// log in, point the builtin marketplace at a branch, install a plugin, run a chat with
// screenshot + video, and restore the marketplace. Evidence goes to CDX_EVIDENCE_DIR,
// never into the repository.
//
// Usage: node devel-vm.cjs <command> [args]
//   use-branch <gitRef>              marketplace -> GitLab source at <gitRef>
//   install <plugin>                 install (or reinstall) <plugin> from the marketplace
//   chat <name> "<prompt>"           new chat; writes <name>.png, <name>.webm, <name>.json
//   restore                          marketplace -> public GitHub dist, default branch
//   status                           marketplace source and plugin versions
//
// Env: CDX_DEVEL_URL, CDX_DEVEL_USER, CDX_DEVEL_PASSWORD, CDX_EVIDENCE_DIR, HEADED=1

const { spawnSync } = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');

const BASE = process.env.CDX_DEVEL_URL || 'https://vm.dev.internal.codexis.ai';
const USER = process.env.CDX_DEVEL_USER || 'umtest@atlasgroup.cz';
const PASSWORD = process.env.CDX_DEVEL_PASSWORD || 'test12';
const CACHE = path.join(os.homedir(), '.cache', 'cdx-devel-vm');
const EVIDENCE = process.env.CDX_EVIDENCE_DIR || path.join(os.tmpdir(), 'cdx-evidence');
const GITLAB_URL = 'https://gitlab.agrp.dev/profidata/codexis-marketplace.git';
const GITHUB_URL = 'https://github.com/atlasgroupcz/codexis-marketplace';

function loadPlaywright() {
  try {
    return require(path.join(CACHE, 'node_modules', 'playwright'));
  } catch {
    console.error(
      `Playwright missing. One-time setup:\n` +
        `  npm install --prefix ${CACHE} playwright@1.59.1\n` +
        `  ${CACHE}/node_modules/.bin/playwright install ffmpeg   (Google Chrome must be installed)`,
    );
    process.exit(2);
  }
}

const Q_MARKETPLACES = `{ marketplaces { id name source { source url ref } error
  plugins { id name version installation { installedVersion } } } }`;
const M_ADD = `mutation($url: String!, $ref: String) {
  addMarketplace(input: { sourceType: GIT, gitUrl: $url, gitRef: $ref }) {
    name source { url ref } error plugins { name version installation { installedVersion } } } }`;
const M_INSTALL = `mutation($id: ID!) { installPlugin(input: { id: $id }) {
  name version installation { installedVersion } skills { name } } }`;
const Q_CHAT = `query($id: ID!) { node(id: $id) { ... on Chat { status messages {
  __typename ... on AiChatMessage { parts { __typename
    ... on TextMessagePart { content }
    ... on ToolMessagePart { toolName output { blocks { ... on TextToolOutput { content } } } }
    ... on ShellToolMessagePart { command }
    ... on SkillToolMessagePart { skill } } } } } } }`;

async function open(recordVideoDir) {
  const { chromium } = loadPlaywright();
  const ctx = await chromium.launchPersistentContext(path.join(CACHE, 'profile'), {
    channel: 'chrome',
    headless: process.env.HEADED !== '1',
    viewport: { width: 1440, height: 900 },
    ...(recordVideoDir ? { recordVideo: { dir: recordVideoDir, size: { width: 1440, height: 900 } } } : {}),
  });
  const page = ctx.pages()[0] || (await ctx.newPage());
  await page.goto(`${BASE}/plugins`, { waitUntil: 'networkidle' });
  if (!page.url().startsWith(BASE)) {
    await page.locator('input:visible').first().fill(USER);
    await page.keyboard.press('Enter');
    await page.locator('input[type=password]').waitFor({ timeout: 30000 });
    await page.fill('input[type=password]', PASSWORD);
    await page.keyboard.press('Enter');
    await page.waitForURL((u) => u.toString().startsWith(BASE), { timeout: 60000 });
    await page.waitForLoadState('networkidle');
  }
  return { ctx, page };
}

async function gql(page, query, variables = {}) {
  for (let attempt = 0; attempt < 10; attempt++) {
    const [status, body] = await page.evaluate(
      async ([query, variables]) => {
        const res = await fetch('/graphql', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify({ query, variables }),
        });
        return [res.status, await res.text()];
      },
      [query, variables],
    );
    if (status === 503) {
      await page.waitForTimeout(5000);
      continue;
    }
    const json = JSON.parse(body);
    if (json.errors) throw new Error(JSON.stringify(json.errors));
    return json.data;
  }
  throw new Error('daemon kept answering 503 - is the VM starting?');
}

async function codexisMarketplace(page) {
  const { marketplaces } = await gql(page, Q_MARKETPLACES);
  const found = marketplaces.find((m) => m.name === 'codexis-plugins');
  if (!found) throw new Error('codexis-plugins marketplace not found');
  return found;
}

async function setSource(page, url, ref) {
  const { addMarketplace: m } = await gql(page, M_ADD, { url, ref });
  if (m.error) throw new Error(`marketplace error: ${m.error}`);
  console.log(JSON.stringify({ name: m.name, source: m.source }, null, 2));
}

async function waitForAnswer(page, chatId) {
  const deadline = Date.now() + 10 * 60 * 1000;
  while (Date.now() < deadline) {
    const { node } = await gql(page, Q_CHAT, { id: chatId });
    const last = node.messages[node.messages.length - 1];
    if (node.status === 'READY' && last && last.__typename === 'AiChatMessage') return node;
    await page.waitForTimeout(3000);
  }
  throw new Error('chat did not finish within 10 minutes');
}

function summarize(node) {
  const parts = node.messages.flatMap((m) => m.parts || []);
  return {
    tools: parts
      .filter((p) => p.toolName)
      .map((p) => ({
        tool: p.toolName,
        input: p.command || p.skill || null,
        output: (p.output?.blocks || []).map((b) => b.content || '').join('').slice(0, 2000),
      })),
    answer: parts
      .filter((p) => p.__typename === 'TextMessagePart')
      .map((p) => p.content)
      .join(''),
  };
}

function assertOutsideRepo(dir) {
  const inside = spawnSync('git', ['-C', dir, 'rev-parse', '--is-inside-work-tree'], { encoding: 'utf8' });
  if (inside.stdout.trim() === 'true') {
    throw new Error(`CDX_EVIDENCE_DIR ${dir} is inside a git checkout - evidence must stay out of the repo`);
  }
}

async function chat(name, prompt) {
  if (!name || !prompt) throw new Error('usage: chat <name> "<prompt>"');
  fs.mkdirSync(EVIDENCE, { recursive: true });
  assertOutsideRepo(EVIDENCE);
  const videoDir = fs.mkdtempSync(path.join(os.tmpdir(), 'cdx-video-'));
  const { ctx, page } = await open(videoDir);
  await page.goto(`${BASE}/`, { waitUntil: 'networkidle' });
  const box = page.locator('textarea, [contenteditable=true]').first();
  await box.waitFor({ timeout: 60000 });
  await box.click();
  await page.keyboard.type(prompt, { delay: 15 });
  await page.keyboard.press('Enter');
  await page.waitForURL(/\/chat\//, { timeout: 60000 });
  const chatId = page.url().split('/chat/')[1].split(/[?#/]/)[0];
  const node = await waitForAnswer(page, chatId);
  await page.waitForTimeout(3000);
  await page.screenshot({ path: path.join(EVIDENCE, `${name}.png`), animations: 'disabled' });
  const result = { chatUrl: page.url(), prompt, ...summarize(node) };
  fs.writeFileSync(path.join(EVIDENCE, `${name}.json`), JSON.stringify(result, null, 2));
  const video = page.video();
  await ctx.close();
  fs.renameSync(await video.path(), path.join(EVIDENCE, `${name}.webm`));
  fs.rmSync(videoDir, { recursive: true, force: true });
  console.log(JSON.stringify(result, null, 2));
  console.log(`evidence: ${path.join(EVIDENCE, name)}.{png,webm,json}`);
}

async function main() {
  const [command, ...args] = process.argv.slice(2);
  if (command === 'chat') return chat(args[0], args[1]);
  const { ctx, page } = await open();
  try {
    if (command === 'use-branch') await setSource(page, GITLAB_URL, args[0]);
    else if (command === 'restore') await setSource(page, GITHUB_URL, null);
    else if (command === 'install') {
      const plugin = (await codexisMarketplace(page)).plugins.find((p) => p.name === args[0]);
      if (!plugin) throw new Error(`plugin ${args[0]} not in the marketplace`);
      console.log(JSON.stringify((await gql(page, M_INSTALL, { id: plugin.id })).installPlugin, null, 2));
    } else if (command === 'status') {
      const m = await codexisMarketplace(page);
      console.log(JSON.stringify({ source: m.source, error: m.error }, null, 2));
      for (const p of m.plugins) {
        console.log(`${p.name} ${p.version}${p.installation ? ` (installed ${p.installation.installedVersion})` : ''}`);
      }
    } else {
      console.error('commands: use-branch <ref> | install <plugin> | chat <name> "<prompt>" | restore | status');
      process.exitCode = 2;
    }
  } finally {
    await ctx.close();
  }
}

main().catch((e) => {
  console.error(e.message || e);
  process.exit(1);
});
