// Screenshots of the printed documents with the print style on (plan 14, NFR-SPD-14), in headless
// Chrome, with no browser add-on: it starts Chrome with a debugging port and asks it, over the
// DevTools protocol, for the PDF of the development page /dev/print (the same pages the hidden
// print window makes, with the canned numbers of the PRD worked day). Each PDF is turned into a PNG
// of its first page with the macOS tool `qlmanage`, when it exists.
//
//   pnpm --filter frontend dev                      (in another terminal; the page is dev only)
//   node frontend/tests/e2e/print-shots.mjs [--base http://localhost:5173] [--out <folder>]
//
// It writes `<name>.pdf` (and `<name>.png`) for the Urdu and English version of every document on
// A4, A5 and the 80 mm roll, tells how many pages each has, and times 15 sheets. It is a developer
// tool: the Windows check of the real WebView2 print is the E2E spec 15-bikri-print.
// Nothing here calculates money: the numbers are the canned PRD numbers of tests/helpers.

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const args = process.argv.slice(2);

/**
 * The value after `--name` on the command line.
 * @param {string} name
 * @param {string} fallback
 * @returns {string}
 */
function option(name, fallback) {
  const at = args.indexOf(`--${name}`);
  return at >= 0 ? (args[at + 1] ?? fallback) : fallback;
}

const BASE = option('base', 'http://localhost:5173');
const OUT = path.resolve(
  option('out', path.join(import.meta.dirname, 'output', 'screenshots', 'plan14')),
);
const CHROME = option(
  'chrome',
  process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
);
const PORT = Number(option('port', '9333'));
const SETTLE_MS = 600;
const READY_TRIES = 100;
const START_TRIES = 50;
const POLL_MS = 100;

/** @type {[name: string, query: string, pageRanges?: string][]} */
const CASES = [];
for (const language of ['ur', 'en']) {
  CASES.push([`seller-bill-${language}-a4`, `doc=seller_bill&language=${language}`]);
  CASES.push([`buyer-slips-${language}-a5`, `doc=buyer_slips&language=${language}`]);
  CASES.push([`pukhta-slip-${language}-a5`, `doc=pukhta_slip&language=${language}`]);
  CASES.push([`buyer-slips-${language}-mm80`, `doc=buyer_slips&language=${language}&paper=mm80`]);
  // A table that runs over several sheets, and the second sheet alone: the header row must repeat
  // and no row may be cut.
  CASES.push([
    `seller-bill-${language}-a4-60-lines`,
    `doc=seller_bill&language=${language}&lines=60`,
  ]);
  CASES.push([
    `seller-bill-${language}-a4-60-lines-page2`,
    `doc=seller_bill&language=${language}&lines=60`,
    '2',
  ]);
}
CASES.push(['seller-bill-ur-a4-urdu-digits', 'doc=seller_bill&language=ur&digits']);
CASES.push(['seller-bill-en-a4-cancelled', 'doc=seller_bill&language=en&cancelled']);
CASES.push(['fifteen-sheets-ur-a5', 'doc=buyer_slips&language=ur&long=15']);

/** @param {number} ms */
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * JSON text as a value of a type nobody has checked yet.
 * @param {string} text
 * @returns {unknown}
 */
function parseJson(text) {
  return JSON.parse(text);
}

/**
 * A tiny DevTools protocol client over the WebSocket that Node has built in.
 * @param {string} url
 */
async function connect(url) {
  const socket = new WebSocket(url);
  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });
  let next = 1;
  /** @type {Map<number, { resolve: (value: unknown) => void, reject: (reason: Error) => void }>} */
  const waiting = new Map();
  socket.addEventListener('message', (event) => {
    const text = typeof event.data === 'string' ? event.data : '';
    const message = /** @type {{ id?: number, result?: unknown, error?: { message: string } }} */ (
      parseJson(text)
    );
    const callback = message.id === undefined ? undefined : waiting.get(message.id);
    if (message.id === undefined || callback === undefined) return;
    waiting.delete(message.id);
    if (message.error === undefined) callback.resolve(message.result);
    else callback.reject(new Error(message.error.message));
  });
  return {
    /**
     * @param {string} method
     * @param {Record<string, unknown>} [params]
     * @returns {Promise<unknown>}
     */
    send: (method, params = {}) =>
      new Promise((resolve, reject) => {
        const id = next;
        next += 1;
        waiting.set(id, { resolve, reject });
        socket.send(JSON.stringify({ id, method, params }));
      }),
    close: () => {
      socket.close();
    },
  };
}

/**
 * The two heights of the 80 mm roll: the height of the document as the hidden window measures it
 * on screen (what the print route reports as `height_px`), and its height with the print style on.
 * They must be the same, or the roll is cut too short or too long.
 * @param {{ send: (method: string, params?: Record<string, unknown>) => Promise<unknown> }} client
 * @param {string} name
 */
async function rollHeights(client, name) {
  const measure = async () => {
    const answer = /** @type {{ result: { value: number } }} */ (
      await client.send('Runtime.evaluate', {
        expression: "document.querySelector('.print-doc').getBoundingClientRect().height",
        returnByValue: true,
      })
    );
    return Math.ceil(answer.result.value);
  };
  // The text is 74 mm wide on the 80 mm paper: 280 CSS pixels.
  await client.send('Emulation.setDeviceMetricsOverride', {
    width: 302,
    height: 1000,
    deviceScaleFactor: 1,
    mobile: false,
  });
  await client.send('Emulation.setEmulatedMedia', { media: 'screen' });
  const screen = await measure();
  await client.send('Emulation.setEmulatedMedia', { media: 'print' });
  const print = await measure();
  // An empty media type is "no emulation": the PDF is made with the print style, not the screen's.
  await client.send('Emulation.setEmulatedMedia', { media: '' });
  await client.send('Emulation.clearDeviceMetricsOverride');
  return `${name}: roll height on screen ${String(screen)} px, in print ${String(print)} px`;
}

/**
 * The number of pages a PDF file has, from its page objects.
 * @param {string} file
 */
function pageCount(file) {
  const pdf = fs.readFileSync(file).toString('latin1');
  return (pdf.match(/\/Type\s*\/Page(?![s\w])/g) ?? []).length;
}

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), 'mandi-print-shots-'));
  const chrome = spawn(
    CHROME,
    [
      '--headless=new',
      '--disable-gpu',
      `--remote-debugging-port=${String(PORT)}`,
      `--user-data-dir=${profile}`,
      'about:blank',
    ],
    { stdio: 'ignore' },
  );
  try {
    /** @type {{ type: string, webSocketDebuggerUrl: string } | undefined} */
    let target;
    for (let attempt = 0; attempt < START_TRIES && target === undefined; attempt += 1) {
      await sleep(2 * POLL_MS);
      try {
        const response = await fetch(`http://127.0.0.1:${String(PORT)}/json/list`);
        const list = /** @type {{ type: string, webSocketDebuggerUrl: string }[]} */ (
          parseJson(await response.text())
        );
        target = list.find((item) => item.type === 'page');
      } catch {
        // Chrome is still starting.
      }
    }
    if (target === undefined) throw new Error('Chrome did not start');
    const client = await connect(target.webSocketDebuggerUrl);
    await client.send('Page.enable');
    await client.send('Runtime.enable');
    for (const [name, query, pageRanges] of CASES) {
      const started = Date.now();
      await client.send('Page.navigate', { url: `${BASE}/#/dev/print?${query}&window` });
      // The page is ready when a print document is on it and the fonts are loaded.
      for (let attempt = 0; attempt < READY_TRIES; attempt += 1) {
        await sleep(POLL_MS);
        const answer = /** @type {{ result: { value: unknown } }} */ (
          await client.send('Runtime.evaluate', {
            expression:
              "document.querySelector('.print-doc') !== null && document.fonts.status === 'loaded'",
            returnByValue: true,
          })
        );
        if (answer.result.value === true) break;
      }
      await sleep(SETTLE_MS);
      if (name.includes('mm80')) console.log(await rollHeights(client, name));
      const printed = /** @type {{ data: string }} */ (
        await client.send('Page.printToPDF', {
          preferCSSPageSize: true,
          printBackground: true,
          displayHeaderFooter: false,
          ...(pageRanges === undefined ? {} : { pageRanges }),
        })
      );
      const pdf = path.join(OUT, `${name}.pdf`);
      fs.writeFileSync(pdf, Buffer.from(printed.data, 'base64'));
      const picture = spawnSync('qlmanage', ['-t', '-s', '2400', pdf, '-o', OUT], {
        stdio: 'ignore',
      });
      if (picture.status === 0 && fs.existsSync(`${pdf}.png`)) {
        fs.renameSync(`${pdf}.png`, path.join(OUT, `${name}.png`));
      }
      console.log(`${name}: ${String(pageCount(pdf))} page(s), ${String(Date.now() - started)} ms`);
    }
    client.close();
  } finally {
    const closed = new Promise((resolve) => chrome.once('exit', resolve));
    chrome.kill();
    await closed;
    // Chrome may still be writing its profile for a moment.
    fs.rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
  }
}

await main();
