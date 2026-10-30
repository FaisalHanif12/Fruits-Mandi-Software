// Plan 39, A1 and A3: runs the alignment check (measure.mjs) on every development page and print,
// in both languages and at both window sizes, in a headless Chrome, and takes the pictures.
//
//   node tests/e2e/alignment/run.mjs --phase before            all screens and prints
//   node tests/e2e/alignment/run.mjs --phase after --only report-r2,grid
//   node tests/e2e/alignment/run.mjs --phase after --sizes 1366x768 --langs ur --no-shots
//   (--json name.json  the result file, to run two sizes side by side;  --which screens|prints|all;
//    --closeups N  how many close-ups of tables and cards to take for each screen, default 3;
//    --only a,b  the screens whose id starts with one of these words; `report-r2$` is exactly that id)
//
// It is a developer tool, like the shots-plan*.playwright.txt snippets: the project does not carry
// Playwright (Tech Stack: WebdriverIO is the Windows suite). It needs
//   - the Vite dev server (pnpm dev) on ALIGN_BASE, default http://localhost:5173;
//   - the folder of playwright-core in PLAYWRIGHT_CORE_DIR (default: found in ~/.npm/_npx);
//   - a Chrome in CHROME_PATH (default: the one Playwright cached under ~/Library/Caches/ms-playwright).
// Output: tests/e2e/output/screenshots/plan39/<phase>/<screen>-<lang>-<width>.png (the window), the
// close-ups <screen>-<lang>-<width>-t<N>.png, and alignment.json. The exit code is 1 while any
// rule fails. Folder names can contain spaces, so every path is built with path.join.

import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { PRINTS, SCREENS } from './screens.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT = path.resolve(HERE, '..', 'output', 'screenshots', 'plan39');
// The page-side function (measure.page.txt): sent to the page as an expression and called there.
const MEASURE = readFileSync(path.join(HERE, 'measure.page.txt'), 'utf8');

function argOf(name, fallback) {
  const at = process.argv.indexOf(`--${name}`);
  return at === -1 ? fallback : (process.argv[at + 1] ?? fallback);
}
const flag = (name) => process.argv.includes(`--${name}`);

const PHASE = argOf('phase', 'after');
const BASE = process.env.ALIGN_BASE ?? 'http://localhost:5173';
const SIZES = argOf('sizes', '1366x768,1024x600')
  .split(',')
  .map((s) => s.split('x').map(Number));
const LANGS = argOf('langs', 'ur,en').split(',');
const ONLY = argOf('only', '')
  .split(',')
  .filter((s) => s !== '');
const SHOTS = !flag('no-shots');
const WHICH = argOf('which', 'all'); // screens | prints | all
const CLOSEUPS = Number(argOf('closeups', '3'));

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

const items = [
  ...(WHICH === 'prints'
    ? []
    : SCREENS.map(([id, route, opts]) => ({ id, route, opts: opts ?? {} }))),
  ...(WHICH === 'screens'
    ? []
    : PRINTS.map(([id, route, opts]) => ({ id, route, opts: opts ?? {} }))),
].filter(
  (item) =>
    ONLY.length === 0 ||
    ONLY.some((o) => (o.endsWith('$') ? item.id === o.slice(0, -1) : item.id.startsWith(o))),
);

const outDir = path.join(OUTPUT, PHASE);
mkdirSync(outDir, { recursive: true });

const browser = await chromium.launch({ headless: true, executablePath: findChrome() });
const results = [];

async function setLanguage(page, lang) {
  for (let step = 0; step < 3; step += 1) {
    const now = await page.evaluate(() => document.documentElement.lang);
    if (now === lang) return;
    await page.keyboard.press('Alt+l');
    await page.waitForTimeout(350);
  }
}

for (const [width, height] of SIZES) {
  const context = await browser.newContext({ viewport: { width, height } });
  const page = await context.newPage();
  for (const item of items) {
    for (const lang of LANGS) {
      const key = `${item.id}-${lang}-${width}`;
      const record = { id: item.id, lang, width, height, route: item.route };
      try {
        // A fresh load each time: the stand-in backend reads the flags of the address when it starts.
        await page.goto('about:blank');
        await page.goto(`${BASE}/#${item.route}`, { waitUntil: 'load', timeout: 60000 });
        await page.waitForSelector(item.opts.noShell ? item.opts.wait : 'h1, h2, [role="alert"]', {
          timeout: 45000,
        });
        await setLanguage(page, lang);
        await page.waitForTimeout(350);
        if (await page.locator('.ag-root-wrapper').count()) {
          await page.waitForSelector('.ag-row', { timeout: 2500 }).catch(() => undefined);
        }
        if (item.opts.open === 'preview') {
          await page
            .getByRole('button', { name: /^(Print preview|پرنٹ پر نظر)/ })
            .first()
            .click({ timeout: 15000 });
          await page.waitForSelector(item.opts.wait, { timeout: 30000 });
          await page.waitForTimeout(900);
        } else if (item.opts.wait) {
          await page.waitForSelector(item.opts.wait, { timeout: 30000 });
        }
        await page.waitForTimeout(400);
        const measured = await page.evaluate(
          `(${MEASURE})(${JSON.stringify({ root: item.opts.root ?? undefined, tolerance: 2 })})`,
        );
        Object.assign(record, {
          dir: measured.dir,
          violations: measured.violations,
          tables: measured.tables,
          cards: measured.cards,
          pageSideways: await page.evaluate(
            () => document.documentElement.scrollWidth - window.innerWidth,
          ),
        });
        if (SHOTS) {
          await page.screenshot({ path: path.join(outDir, `${key}.png`) });
          const targets = item.opts.root ? [page.locator(item.opts.root).first()] : [];
          if (!item.opts.root) {
            // Tables and grids first, then the cards of labels and values.
            for (const selector of ['.ag-root-wrapper, main table', 'main dl']) {
              const handles = await page.locator(selector).elementHandles();
              for (const handle of handles) {
                const box = await handle.boundingBox();
                if (box && box.height > 40 && box.width > 120) targets.push(handle);
                if (targets.length >= CLOSEUPS) break;
              }
              if (targets.length >= CLOSEUPS) break;
            }
          }
          let n = 0;
          for (const target of targets) {
            n += 1;
            const file = path.join(outDir, `${key}-t${n}.png`);
            if ('screenshot' in target && typeof target.screenshot === 'function') {
              await target.screenshot({ path: file, timeout: 5000 }).catch(() => undefined);
            }
          }
        }
      } catch (error) {
        record.error = String(error).slice(0, 200);
        record.violations = record.violations ?? [];
      }
      results.push(record);
      // The result file is kept up to date after every page, so a long run can be read while it goes.
      writeFileSync(
        path.join(outDir, argOf('json', 'alignment.json')),
        JSON.stringify(results, null, 1),
      );
      const count = record.violations.length;
      process.stdout.write(
        `${record.error ? 'ERR ' : count === 0 ? 'ok  ' : 'FAIL'} ${key} ${count > 0 ? `(${count})` : ''}${record.error ? ' ' + record.error : ''}\n`,
      );
    }
  }
  await context.close();
}
await browser.close();

writeFileSync(path.join(outDir, argOf('json', 'alignment.json')), JSON.stringify(results, null, 1));
const failing = results.filter((r) => r.violations.length > 0 || r.error);
const byRule = {};
for (const r of results) for (const v of r.violations) byRule[v.rule] = (byRule[v.rule] ?? 0) + 1;
process.stdout.write(
  `\n${results.length} runs, ${failing.length} with a problem. By rule: ${JSON.stringify(byRule)}\n`,
);
process.exit(failing.length > 0 ? 1 : 0);
