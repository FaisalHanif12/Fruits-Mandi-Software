// PDFs of printed documents, made through headless Chrome from the SAME print routes the hidden print
// window uses (`#/print?...`), with the backend's own print data (the contract fixtures of
// frontend/tests/fixtures) instead of the Tauri backend. Plan 69 T3 and plan 63 T4, deliverable D8
// (D8-A6, D8-A12): the evidence that the Urdu seller bill (R1), ادھاری تفصیل (R5) and the party day slip
// (R35) hold no Urdu digits (۰ to ۹) and no Arabic-Indic digits (٠ to ٩), and do hold grouped numbers
// like "10,000" and the year "2026".
//
//   node scripts/print/pdf-from-route.mjs [--out docs/reviews/d8-evidence] [--base http://localhost:5173]
//                                         [--only r1,r5,r35] [--chrome <path>] [--port 9444]
//
// How it works, for each document:
//   1. Chrome opens the real route. A stand-in for Tauri's `invoke` is put on the page BEFORE the app
//      loads (CDP `Page.addScriptToEvaluateOnNewDocument`), so the route runs its own code: it asks for
//      its print data and the shared header with the real command names, and the stand-in answers with
//      the backend's fixture. Nothing in the app is changed and no screen calculates anything.
//   2. The route says `print-ready-<job>` with its page height, exactly as it tells the backend. This
//      script waits for that event (and stops with `print-error-<job>`).
//   3. Chrome prints the page to a PDF with the page size and margins of the print style
//      (`preferCSSPageSize`), like the WebView2 print of Windows.
//   4. A PNG of the first page is made (macOS `qlmanage`), and the text of the PDF is read with
//      PDFKit (scripts/print/pdf-text.swift, macOS only) into `<name>.txt`.
//   5. The text is checked: no ۰-۹ or ٠-٩, and the numbers the document must hold. The exit code is
//      1 when any check fails.
//
// The text of a PDF proves the characters, not the glyph shapes. Look at the PNG as well: an Urdu
// font could draw 0-9 in another shape. The Windows WebView2 check stays LOCAL-ONLY (CLAUDE.md).
// If nothing listens on --base, this script starts the Vite dev server of frontend/ itself and
// stops it at the end.

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const FIXTURES = path.join(ROOT, 'frontend', 'tests', 'fixtures');
const SWIFT_SOURCE = path.join(ROOT, 'scripts', 'print', 'pdf-text.swift');

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
const OUT = path.resolve(ROOT, option('out', path.join('docs', 'reviews', 'd8-evidence')));
const CHROME = option(
  'chrome',
  process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
);
const PORT = Number(option('port', '9444'));
const ONLY = option('only', '')
  .split(',')
  .map((item) => item.trim())
  .filter((item) => item !== '');
const READY_TIMEOUT_MS = 90_000;
const POLL_MS = 100;
const START_TRIES = 100;
const SETTLE_MS = 600;
const PNG_SIZE = 1400;

/** Digits that are not 0-9: Urdu (U+06F0-06F9) and Arabic-Indic (U+0660-0669). */
const ODD_DIGITS = /[۰-۹٠-٩]/g;

/** @param {number} ms */
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * The JSON of a fixture file. A `.snap` file is an `insta` snapshot: a header between two `---`
 * lines, then the JSON.
 * @param {string} file path relative to frontend/tests/fixtures
 * @returns {unknown}
 */
function fixture(file) {
  const text = fs.readFileSync(path.join(FIXTURES, file), 'utf8');
  const body = file.endsWith('.snap') ? text.split('---\n').slice(2).join('---\n') : text;
  return JSON.parse(body);
}

/**
 * The address of a report print window, from the request the backend fixture was made with (the
 * same fields `reports_print_report` puts into the address).
 * @param {string} requestFile
 * @param {string} job
 * @param {boolean} slips
 */
function reportAddress(requestFile, job, slips) {
  const request = /** @type {{ id: string, language: string, paper: string, filters: unknown }} */ (
    fixture(requestFile)
  );
  const query = new URLSearchParams({
    report: request.id,
    language: request.language,
    paper: request.paper,
    filters: JSON.stringify(request.filters),
    job,
  });
  if (slips) query.set('slips', '1');
  return `#/print?${query.toString()}`;
}

/**
 * The three documents of D8-A6, in Urdu. `answers` is what the stand-in gives for each command the
 * route calls; `must` is what the text must hold (besides 2026 and no odd digits).
 * @type {{ name: string, label: string, address: string, answers: Record<string, unknown>, must: string[] }[]}
 */
const CASES = [
  {
    name: 'r1-seller-bill-ur',
    label: 'R1 seller bill',
    address: '#/print?bikri=1&doc=seller_bill&language=ur&job=pdfr1',
    answers: {
      reports_bikri_print_data: fixture('print/print_data__snapshot_seller_bill_event1_urdu.snap'),
    },
    must: ['29,400', '19,316', '10-10-2026'],
  },
  {
    name: 'r5-adhari-tafseel-ur',
    label: 'R5 ادھاری تفصیل (all the buyers, one slip each)',
    address: reportAddress('reports_buyer_bills_print_data_ur_request.json', 'pdfr5', true),
    answers: {
      reports_buyer_bills_print_data: fixture('reports_buyer_bills_print_data_ur.json'),
    },
    must: ['12,000', '17,400', '10-10-2026'],
  },
  {
    name: 'r35-party-slip-ur',
    label: 'R35 party day slip (Akram)',
    address: reportAddress('report_r35_slip_buyer_akram_print_ur_request.json', 'pdfr35', false),
    answers: {
      reports_print_data: fixture('report_r35_slip_buyer_akram_print_ur.json'),
    },
    must: ['10,000', '12,880', '10-10-2026'],
  },
];

/** The shared header and footer of every report print (`reports_print_header`). */
const HEADER_BASE = fixture('reports_print_header_ur.json');

/**
 * The page script that stands in for Tauri's `invoke`. It records every command the route calls and
 * every event it emits, answers the commands of `answers`, echoes the title and facts into the
 * shared header (as the backend does) and refuses anything else with ERR_NOT_IMPLEMENTED.
 * @param {Record<string, unknown>} answers
 */
function bridgeSource(answers) {
  return `(() => {
    const answers = ${JSON.stringify(answers)};
    const headerBase = ${JSON.stringify(HEADER_BASE)};
    window.__calls = [];
    window.__events = [];
    let callbacks = 0;
    const refusal = (code) =>
      Object.assign(new Error(code), { ok: false, errors: [{ code, field: null, values: {} }], ref: 'E-0' });
    window.__TAURI_INTERNALS__ = {
      transformCallback: (callback) => {
        callbacks += 1;
        window['_' + callbacks] = callback;
        return callbacks;
      },
      unregisterCallback: () => undefined,
      convertFileSrc: (file) => file,
      invoke: (command, input) =>
        new Promise((resolve, reject) => {
          window.__calls.push(command);
          setTimeout(() => {
            if (command === 'plugin:event|emit') {
              window.__events.push({ event: input.event, payload: input.payload });
              resolve(null);
            } else if (command === 'reports_print_header') {
              const asked = input.input;
              resolve({ ...headerBase, header: { ...headerBase.header, title_id: asked.title_id, facts: asked.facts } });
            } else if (command in answers) {
              resolve(answers[command]);
            } else {
              reject(refusal('ERR_NOT_IMPLEMENTED'));
            }
          }, 10);
        }),
    };
  })();`;
}

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

/** True when something answers at the address. @param {string} url */
async function reachable(url) {
  try {
    const response = await fetch(url);
    return response.ok;
  } catch {
    return false;
  }
}

/** Starts the Vite dev server of frontend/ on the port of BASE. Returns the child to stop. */
async function startDevServer() {
  const port = new URL(BASE).port || '5173';
  const vite = path.join(ROOT, 'frontend', 'node_modules', 'vite', 'bin', 'vite.js');
  const child = spawn(process.execPath, [vite, '--port', port, '--strictPort'], {
    cwd: path.join(ROOT, 'frontend'),
    stdio: 'ignore',
  });
  for (let attempt = 0; attempt < START_TRIES; attempt += 1) {
    await sleep(5 * POLL_MS);
    if (await reachable(BASE)) return child;
  }
  child.kill();
  throw new Error(`the dev server did not start on ${BASE}`);
}

/** Compiles scripts/print/pdf-text.swift once and returns the path of the tool. */
function buildTextTool() {
  const tool = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'mandi-pdf-text-')), 'pdf-text');
  const built = spawnSync('swiftc', ['-O', SWIFT_SOURCE, '-o', tool], { encoding: 'utf8' });
  if (built.status !== 0) throw new Error(`swiftc failed: ${built.stderr}`);
  return tool;
}

/**
 * One document: open the route, wait for its ready event, print to PDF, make the PNG and the text.
 * @param {{ send: (method: string, params?: Record<string, unknown>) => Promise<unknown> }} client
 * @param {(typeof CASES)[number]} item
 * @param {string} textTool
 */
async function makeOne(client, item, textTool) {
  const started = Date.now();
  // A new address that only differs after the `#` would not reload the page, and the stand-in of the
  // last document would stay. A blank page first makes every document load from nothing.
  await client.send('Page.navigate', { url: 'about:blank' });
  await sleep(POLL_MS);
  /** @type {{ identifier: string }} */
  const registered = /** @type {{ identifier: string }} */ (
    await client.send('Page.addScriptToEvaluateOnNewDocument', {
      source: bridgeSource(item.answers),
    })
  );
  try {
    await client.send('Page.navigate', { url: `${BASE}/${item.address}` });
    const job = /job=([A-Za-z0-9_-]+)/.exec(item.address)?.[1] ?? '';
    /** @type {{ height_px?: number } | null} */
    let ready = null;
    for (let waited = 0; waited < READY_TIMEOUT_MS && ready === null; waited += POLL_MS) {
      await sleep(POLL_MS);
      const answer = /** @type {{ result: { value?: unknown } }} */ (
        await client.send('Runtime.evaluate', {
          expression: `(window.__events ?? []).find((e) => e.event === 'print-ready-${job}' || e.event === 'print-error-${job}') ?? null`,
          returnByValue: true,
        })
      );
      const found = /** @type {{ event: string, payload: { height_px?: number } } | null} */ (
        answer.result.value ?? null
      );
      if (found?.event === `print-error-${job}`) {
        const calls = /** @type {{ result: { value: string[] } }} */ (
          await client.send('Runtime.evaluate', {
            expression: 'window.__calls',
            returnByValue: true,
          })
        ).result.value;
        throw new Error(
          `${item.name}: the route said print-error ${JSON.stringify(found.payload)} after calling ${calls.join(', ')}`,
        );
      }
      if (found !== null) ready = found.payload;
    }
    if (ready === null) throw new Error(`${item.name}: no print-ready event in time`);
    await sleep(SETTLE_MS);
    const calls = /** @type {{ result: { value: string[] } }} */ (
      await client.send('Runtime.evaluate', { expression: 'window.__calls', returnByValue: true })
    ).result.value;
    const printed = /** @type {{ data: string }} */ (
      await client.send('Page.printToPDF', {
        preferCSSPageSize: true,
        printBackground: true,
        displayHeaderFooter: false,
      })
    );
    const pdf = path.join(OUT, `${item.name}.pdf`);
    fs.writeFileSync(pdf, Buffer.from(printed.data, 'base64'));
    const picture = spawnSync('qlmanage', ['-t', '-s', String(PNG_SIZE), pdf, '-o', OUT], {
      stdio: 'ignore',
    });
    if (picture.status === 0 && fs.existsSync(`${pdf}.png`)) {
      fs.renameSync(`${pdf}.png`, path.join(OUT, `${item.name}.png`));
    }
    const extracted = spawnSync(textTool, [pdf], { encoding: 'utf8', maxBuffer: 1 << 26 });
    if (extracted.status !== 0) throw new Error(`${item.name}: text extraction failed`);
    const text = extracted.stdout;
    fs.writeFileSync(path.join(OUT, `${item.name}.txt`), text);
    const pages = Math.max(
      1,
      (
        fs
          .readFileSync(pdf)
          .toString('latin1')
          .match(/\/Type\s*\/Page(?![s\w])/g) ?? []
      ).length,
    );
    return { text, calls, pages, ms: Date.now() - started };
  } finally {
    await client.send('Page.removeScriptToEvaluateOnNewDocument', {
      identifier: registered.identifier,
    });
  }
}

/**
 * The checks of one document's text. Returns one line for each, with ok or FAIL.
 * @param {string} text
 * @param {string[]} must
 * @returns {{ ok: boolean, line: string }[]}
 */
function check(text, must) {
  const odd = text.match(ODD_DIGITS) ?? [];
  const results = [
    { ok: odd.length === 0, line: `no ۰-۹ or ٠-٩ in the text (found ${String(odd.length)})` },
    { ok: /\b\d{2}-\d{2}-2026\b/.test(text), line: 'a date with the year 2026 (DD-MM-2026)' },
    { ok: /2026/.test(text), line: 'holds "2026"' },
    { ok: /\d{1,3}(?:,\d{3})+/.test(text), line: 'has grouped numbers like 10,000' },
  ];
  for (const figure of must) {
    results.push({ ok: text.includes(figure), line: `holds "${figure}"` });
  }
  return results;
}

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  const chosen = CASES.filter(
    (item) => ONLY.length === 0 || ONLY.some((short) => item.name.startsWith(short)),
  );
  if (chosen.length === 0) throw new Error(`nothing matches --only ${ONLY.join(',')}`);
  const server = (await reachable(BASE)) ? null : await startDevServer();
  const textTool = buildTextTool();
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), 'mandi-pdf-route-'));
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
  let failed = false;
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
    /** @type {string[]} */
    const texts = [];
    for (const item of chosen) {
      const made = await makeOne(client, item, textTool);
      texts.push(made.text);
      console.log(
        `\n${item.label}: ${item.name}.pdf, ${String(made.pages)} page(s), ${String(made.ms)} ms`,
      );
      console.log(`  commands the route called: ${made.calls.join(', ')}`);
      for (const result of check(made.text, item.must)) {
        if (!result.ok) failed = true;
        console.log(`  ${result.ok ? 'ok  ' : 'FAIL'} ${result.line}`);
      }
    }
    // The amount of the acceptance (D8-A6) is on the party slip: the worked day has no 10,000 on the
    // seller bill or on R5, so it is checked over the documents together.
    if (chosen.length === CASES.length) {
      const together = texts.some((text) => text.includes('10,000'));
      if (!together) failed = true;
      console.log(`\n${together ? 'ok  ' : 'FAIL'} the documents together hold "10,000"`);
    }
    client.close();
  } finally {
    const closed = new Promise((resolve) => chrome.once('exit', resolve));
    chrome.kill();
    await closed;
    fs.rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
    if (server !== null) server.kill();
  }
  console.log(`\nfiles in ${path.relative(ROOT, OUT)}`);
  if (failed) process.exitCode = 1;
}

await main();
