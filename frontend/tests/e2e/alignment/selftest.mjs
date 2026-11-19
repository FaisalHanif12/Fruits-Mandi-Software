// Plan 39: the alignment check proves itself. It is given small pages that are lined up wrongly on
// purpose, and must name each fault (a check that passes everything proves nothing); and a page that is
// lined up right, which it must pass.
//
//   node tests/e2e/alignment/selftest.mjs
//
// Same needs as run.mjs: playwright-core (PLAYWRIGHT_CORE_DIR) and a Chrome (CHROME_PATH).

import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const MEASURE = readFileSync(path.join(HERE, 'measure.page.txt'), 'utf8');

function findPlaywright() {
  if (process.env.PLAYWRIGHT_CORE_DIR) return process.env.PLAYWRIGHT_CORE_DIR;
  const npx = path.join(os.homedir(), '.npm', '_npx');
  for (const dir of existsSync(npx) ? readdirSync(npx) : []) {
    const candidate = path.join(npx, dir, 'node_modules', 'playwright-core');
    if (existsSync(candidate)) return candidate;
  }
  throw new Error('playwright-core not found: set PLAYWRIGHT_CORE_DIR');
}

function findChrome() {
  if (process.env.CHROME_PATH) return process.env.CHROME_PATH;
  const cache = path.join(os.homedir(), 'Library', 'Caches', 'ms-playwright');
  for (const dir of existsSync(cache) ? readdirSync(cache).sort().reverse() : []) {
    if (!dir.startsWith('chromium_headless_shell')) continue;
    const base = path.join(cache, dir);
    for (const sub of readdirSync(base)) {
      const exe = path.join(base, sub, 'chrome-headless-shell');
      if (existsSync(exe)) return exe;
    }
  }
  throw new Error('Chrome not found: set CHROME_PATH');
}

const require = createRequire(path.join(findPlaywright(), 'index.js'));
const { chromium } = require(findPlaywright());

const STYLE = `<style>
  body { margin: 0; font: 16px Arial; }
  table { border-collapse: collapse; width: 600px; }
  th, td { padding: 8px 12px; }
  .ok-num { text-align: center; }
</style>`;

/** Pages with a fault on purpose: the rule the check must report, and the page. */
const FAULTS = [
  [
    'R1: a figure column whose heading is on the right while its figures are in the middle',
    'R1',
    `<table><thead><tr><th style="text-align:start">Name</th><th style="text-align:right">Quantity</th></tr></thead>
      <tbody><tr><td>Akram</td><td class="ok-num">17</td></tr><tr><td>Arshad</td><td class="ok-num">250</td></tr></tbody></table>`,
  ],
  [
    'R1: figures of one column that are not level (some centred, some on the right)',
    'R1',
    `<table><thead><tr><th>Name</th><th class="ok-num">Amount</th></tr></thead>
      <tbody><tr><td>A</td><td class="ok-num">12,000</td></tr><tr><td>B</td><td style="text-align:right">5</td></tr></tbody></table>`,
  ],
  [
    'R2: a text column whose heading starts at the other edge',
    'R2',
    `<table><thead><tr><th style="text-align:right">Buyer</th><th class="ok-num">Qty</th></tr></thead>
      <tbody><tr><td style="text-align:left">Akram Toll Tax</td><td class="ok-num">17</td></tr></tbody></table>`,
  ],
  [
    'R4: a heading cut off by its cell',
    'R4',
    `<table style="table-layout:fixed;width:300px"><thead><tr><th style="overflow:hidden;white-space:nowrap;width:60px">A very long heading</th><th>Qty</th></tr></thead>
      <tbody><tr><td>x</td><td>1</td></tr></tbody></table>`,
  ],
  [
    'R4: a heading broken in the middle of a word',
    'R4',
    `<table style="table-layout:fixed;width:300px"><thead><tr><th style="width:40px;word-break:break-all">Commission</th><th>Qty</th></tr></thead>
      <tbody><tr><td>x</td><td>1</td></tr></tbody></table>`,
  ],
  [
    'R6: a total that is not under its figures',
    'R6',
    `<table><thead><tr><th>Name</th><th class="ok-num">Amount</th></tr></thead>
      <tbody><tr><td>A</td><td class="ok-num">12,000</td></tr></tbody>
      <tfoot><tr><td>Total</td><td style="text-align:right">12,000</td></tr></tfoot></table>`,
  ],
  [
    'R7: words at the top of a tall row',
    'R7',
    `<table><thead><tr><th>Name</th><th class="ok-num">Qty</th></tr></thead>
      <tbody><tr style="height:90px"><td style="vertical-align:top">Akram</td><td class="ok-num" style="vertical-align:top">17</td></tr></tbody></table>`,
  ],
  [
    'R8: the figures of a card of lines that end at different edges',
    'R8',
    `<dl style="width:300px"><div style="display:flex;justify-content:space-between"><dt>Gross</dt><dd style="margin:0;direction:ltr"><bdi>29,400</bdi></dd></div>
      <div style="display:flex;justify-content:space-between"><dt>Freight</dt><dd style="margin:0">3</dd></div>
      <div style="display:flex;justify-content:space-between"><dt>Net</dt><dd style="margin:0;margin-inline-end:30px">19,316</dd></div></dl>`,
  ],
];

const RIGHT = `<table><thead><tr><th data-align="text" style="text-align:start">Name</th><th data-align="num" class="ok-num">Quantity</th><th data-align="num" class="ok-num">Amount</th></tr></thead>
  <tbody><tr><td data-align="text">Akram</td><td data-align="num" class="ok-num">17</td><td data-align="num" class="ok-num">12,880</td></tr>
  <tr><td data-align="text">Arshad Multani</td><td data-align="num" class="ok-num">250</td><td data-align="num" class="ok-num">18,684</td></tr></tbody>
  <tfoot><tr><td data-align="text">Total</td><td data-align="num" class="ok-num">267</td><td data-align="num" class="ok-num">31,564</td></tr></tfoot></table>
  <dl style="width:300px"><div style="display:flex;justify-content:space-between"><dt>Gross</dt><dd style="margin:0;width:90px;text-align:end;direction:ltr">29,400</dd></div>
  <div style="display:flex;justify-content:space-between"><dt>Freight</dt><dd style="margin:0;width:90px;text-align:end;direction:ltr">3,000</dd></div>
  <div style="display:flex;justify-content:space-between"><dt>Net</dt><dd style="margin:0;width:90px;text-align:end;direction:ltr">19,316</dd></div></dl>`;

const browser = await chromium.launch({ headless: true, executablePath: findChrome() });
const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
let failed = 0;

async function measure(html, dir) {
  await page.setContent(
    `<!doctype html><html dir="${dir}" lang="${dir === 'rtl' ? 'ur' : 'en'}"><head>${STYLE}</head><body>${html}</body></html>`,
  );
  return page.evaluate(`(${MEASURE})(${JSON.stringify({ tolerance: 2 })})`);
}

for (const dir of ['ltr', 'rtl']) {
  for (const [name, rule, html] of FAULTS) {
    const result = await measure(html, dir);
    const found = result.violations.some((v) => v.rule === rule);
    process.stdout.write(
      `${found ? 'ok  ' : 'FAIL'} ${dir} ${found ? 'finds' : 'MISSED'} ${name}\n`,
    );
    if (!found) failed += 1;
  }
  const right = await measure(RIGHT, dir);
  const clean = right.violations.length === 0;
  process.stdout.write(
    `${clean ? 'ok  ' : 'FAIL'} ${dir} passes a table and a card that are lined up\n`,
  );
  if (!clean) {
    failed += 1;
    for (const v of right.violations)
      process.stdout.write(`       ${v.rule} ${v.where}: ${v.what}\n`);
  }
}
await browser.close();
process.stdout.write(
  failed === 0
    ? '\nThe check finds every fault and passes the right page.\n'
    : `\n${String(failed)} self-tests failed.\n`,
);
process.exit(failed === 0 ? 0 : 1);
