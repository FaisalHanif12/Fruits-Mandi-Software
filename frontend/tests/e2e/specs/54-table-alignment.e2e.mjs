// Plan 39 (PRD FR-19.13, NFR-USE-19, NFR-USE-28, Code Standards F17 to F19): every table, grid and
// label-and-value card of the real app is lined up, in Urdu and then in English, on the real app after
// the worked day (the earlier specs have entered the بکری, the vouchers and the expenses).
//   1. For each screen below, the alignment check (tests/e2e/alignment/measure.page.txt) measures the layout
//      the browser made and finds nothing:
//      - R1 a number column (money, qty, rate, date, code, days): its heading, its figures and its
//        total are centred in the column;
//      - R2 a text column: the words and the heading start at the same edge;
//      - R3 a tick, a chip or an icon: centred;
//      - R4 a heading is never cut and never broken in a word, at most two lines, one height;
//      - R6 a total sits under its column; the word "Total" is in the first text column;
//      - R7 the words are in the middle of the row and keep off the cell border;
//      - R8 the figures of a card of lines stand in one column on the right edge.
//      The allowed difference between two edges is 2 px.
// The same function runs on the development pages in headless Chrome (node tests/e2e/alignment/run.mjs),
// with both sizes and the pictures. This spec proves it on the real app. It runs on Windows only
// (LOCAL-ONLY in the Mac phase: checked with `node --check`).

import { browser, expect } from '@wdio/globals';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { E2E_DIR } from '../paths.mjs';
import { OWNER } from '../support/credentials.mjs';
import { openRoute, startSignedIn, switchLanguage } from '../support/page.mjs';

const SETTLE_MS = 1800;

/** The page-side check, as the text of one function (tests/e2e/alignment/measure.page.txt). */
const MEASURE = readFileSync(path.join(E2E_DIR, 'alignment', 'measure.page.txt'), 'utf8');

/**
 * What the check reports for the page it ran on.
 * @typedef {{ rule: string, where: string, what: string, text?: string }} Finding
 * @typedef {{ lang: string, violations: Finding[] }} Measured
 */

/** The screens that hold a table, a grid or a card of figures. */
const ROUTES = [
  '#/',
  '#/bikri/book',
  '#/bikri/today',
  '#/bikri/stock',
  '#/bikri/pukhta',
  '#/bikri/receipt',
  '#/bikri/collection',
  '#/rokar/book',
  '#/rokar/general',
  '#/index/accounts',
  '#/index/chart',
  '#/partners',
  '#/partners/statement?partner=2',
  '#/day',
  '#/month',
  '#/reports/R2',
  '#/reports/R5',
  '#/reports/R10',
  '#/reports/R12',
  '#/reports/R18',
  '#/reports/R20',
  '#/reports/R24',
  '#/reports/R26',
  '#/reports/R34',
  '#/reports/R35',
  '#/reports/R37',
  '#/analytics/A',
  '#/analytics/B',
  '#/analytics/F',
  '#/analytics/J',
  '#/analytics/K',
];

/**
 * Opens a screen, waits for it to settle and measures it.
 * @param {string} route
 */
async function measureRoute(route) {
  await openRoute(route);
  await browser.pause(SETTLE_MS);
  /** @type {Measured} */
  const measured = await browser.execute(`return (${MEASURE})(arguments[0]);`, { tolerance: 2 });
  return measured;
}

/** @param {'ur' | 'en'} language */
function checkAll(language) {
  for (const route of ROUTES) {
    it(`${route} is lined up (${language})`, async () => {
      const result = await measureRoute(route);
      expect(result.lang).toBe(language);
      // The words of every finding are in the failure message, so a run on the CI shows where.
      const lines = result.violations.map(
        (v) => `${v.rule} ${v.where}: ${v.what}${v.text ? ` [${v.text}]` : ''}`,
      );
      expect(lines).toEqual([]);
    });
  }
}

describe('every table is lined up (Urdu)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });
  checkAll('ur');
});

describe('every table is lined up (English)', () => {
  before(async () => {
    await switchLanguage('en');
  });
  checkAll('en');
});
