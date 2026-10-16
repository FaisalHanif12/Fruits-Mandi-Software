// The foundation of every printed page (plan 14, reference.md of the print skill): the page footer
// text, the print stylesheet (one literal @page rule per paper class, repeating table headers,
// rows that never split, numbers on the physical right edge, Nastaliq line height), the Fluent ids
// the backend sends (plan 14 contract 6) in both languages, and the five print commands.

import { invoke } from '@tauri-apps/api/core';
import fs from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  reportsBikriPrintData,
  reportsListPrinters,
  reportsPrint,
} from '../../src/features/reports/api';
import { settingsGetPrinting, settingsSetPrinting } from '../../src/features/settings/api';
import { bundles, LANGUAGES } from '../../src/i18n';
import { cssString, footerContent, waitForPrintReady } from '../../src/shared/print';
import { printingSettings } from '../helpers/print';
import { printDataFor } from '../helpers/print-data';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

const SRC = path.resolve(import.meta.dirname, '../../src');
const PRINT_CSS = fs.readFileSync(path.join(SRC, 'shared/print/print.css'), 'utf8');
const TOKENS_CSS = fs.readFileSync(path.join(SRC, 'styles/tokens.css'), 'utf8');

describe('the page footer text', () => {
  const words = { printed: 'Printed', page: 'Page', of: 'of' };
  const isolate = String.fromCodePoint(0x2066);
  const pop = String.fromCodePoint(0x2069);

  it('is the print time, then "Page" and the two counters', () => {
    expect(footerContent(words, '10-10-2026 17:40', false)).toBe(
      `"Printed ${isolate}10-10-2026 17:40${pop} · Page " counter(page) " of " counter(pages)`,
    );
  });

  it('keeps the print time left to right inside an Urdu line, where a margin box has no <bdi>', () => {
    const urdu = { printed: 'پرنٹ کا وقت', page: 'صفحہ', of: 'از' };
    const text = footerContent(urdu, '10-10-2026 17:40', false);
    expect(text).toContain(`${isolate}10-10-2026 17:40${pop}`);
    expect(text).toContain('"پرنٹ کا وقت');
    expect(text).toContain('صفحہ ');
  });

  it('uses the Urdu digit style for the page counters when the Urdu digits setting is on', () => {
    expect(footerContent(words, '۱۰-۱۰-۲۰۲۶', true)).toContain(
      'counter(page, persian) " of " counter(pages, persian)',
    );
  });

  it('is only the print time on the 80 mm roll, which is one tall page', () => {
    expect(footerContent(words, '10-10-2026 17:40', false, false)).toBe(
      `"Printed ${isolate}10-10-2026 17:40${pop}"`,
    );
  });

  it('drops the isolation marks Fluent puts round words, and writes a string safe for CSS', () => {
    const marked = `${isolate}Page${pop}`;
    expect(footerContent({ ...words, page: marked }, '10-10-2026 17:40', false)).not.toContain(
      `${isolate}Page`,
    );
    expect(cssString('say "hi"\\now')).toBe('"say \\"hi\\"\\\\now"');
    expect(cssString('one\ntwo')).toBe('"one two"');
  });
});

describe('the print stylesheet', () => {
  it('has ONE literal @page rule for each paper class, with the paper size in it', () => {
    for (const [name, size] of [
      ['a4', 'a4'],
      ['a5', 'a5'],
      ['r80', '80mm 297mm'],
    ] as const) {
      const rules = PRINT_CSS.match(new RegExp(`@page ${name} \\{`, 'g')) ?? [];
      expect(rules, `@page ${name}`).toHaveLength(1);
      const body = PRINT_CSS.slice(PRINT_CSS.indexOf(`@page ${name} {`));
      expect(body.slice(0, 200)).toContain(`size: ${size};`);
    }
    // The paper of the whole page follows the class on the document.
    for (const [paper, page] of [
      ['a4', 'a4'],
      ['a5', 'a5'],
      ['mm80', 'r80'],
    ] as const) {
      expect(PRINT_CSS).toContain(`html:has(.print-doc[data-paper='${paper}'])`);
      expect(PRINT_CSS).toMatch(
        new RegExp(`html:has\\(\\.print-doc\\[data-paper='${paper}'\\]\\) \\{\\s*page: ${page};`),
      );
    }
  });

  it('draws the page footer from the --print-footer property in the bottom margin of every paper', () => {
    expect(PRINT_CSS.match(/@bottom-center/g)).toHaveLength(3);
    expect(PRINT_CSS.match(/content: var\(--print-footer, (""|'')\);/g)).toHaveLength(3);
  });

  it('repeats the table header on every page and never splits a row', () => {
    expect(PRINT_CSS).toMatch(/\.print-table thead \{\s*display: table-header-group;/);
    expect(PRINT_CSS).toMatch(/\.print-table tr \{\s*break-inside: avoid;/);
    expect(PRINT_CSS).toMatch(/\.print-compact-line \{[^}]*break-inside: avoid;/);
    expect(PRINT_CSS).toMatch(/\.print-summary \{[^}]*break-inside: avoid;/);
  });

  it('puts every number on the physical right edge with equal-width digits', () => {
    expect(PRINT_CSS).toMatch(
      /\.print-table \.print-num,\s*\.print-amounts \.print-num \{\s*direction: ltr;\s*text-align: end;/,
    );
    expect(PRINT_CSS).toContain('font-variant-numeric: tabular-nums');
  });

  it('starts each party on a new sheet on A4 and A5, and cuts the 80 mm roll with a line', () => {
    expect(PRINT_CSS).toMatch(
      /\.print-doc:not\(\[data-paper='mm80'\]\) \.print-sheet \+ \.print-sheet \{\s*break-before: page;/,
    );
    expect(PRINT_CSS).toMatch(
      /\.print-doc\[data-paper='mm80'\] \.print-sheet \+ \.print-sheet \{[^}]*border-block-start: var\(--print-rule\) dashed/,
    );
  });

  it('reads fonts and sizes from tokens only, and has no physical direction', () => {
    expect(PRINT_CSS).not.toMatch(/#[0-9a-f]{3,8}\b/i);
    expect(PRINT_CSS).not.toMatch(/\d(px|pt|rem|em)\b/);
    expect(PRINT_CSS).not.toMatch(/\b(left|right)\b\s*:/);
    expect(PRINT_CSS).not.toMatch(
      /margin-(left|right)|padding-(left|right)|text-align:\s*(left|right)/,
    );
    expect(PRINT_CSS).toContain('font-family: var(--font-text)');
    expect(PRINT_CSS).toContain('var(--font-urdu-digits)');
  });

  it('lays the hidden window out at the width of the text of each paper, with nothing added on screen', () => {
    // The text width is the paper less its side margins: 210 - 2 x 10, 148 - 2 x 8, 80 - 2 x 3.
    for (const [token, mm] of [
      ['a4', 190],
      ['a5', 132],
      ['mm80', 74],
    ] as const) {
      expect(TOKENS_CSS).toContain(`--print-content-w-${token}: ${String(mm)}mm;`);
      expect(PRINT_CSS).toMatch(
        new RegExp(
          `\\.print-doc\\[data-window\\]\\[data-paper='${token}'\\] \\.print-sheet \\{\\s*inline-size: var\\(--print-content-w-${token}\\);`,
        ),
      );
    }
    expect(PRINT_CSS).toMatch(
      /\.print-doc\[data-window\] \.print-sheet \{\s*padding: 0;\s*border: 0;\s*box-shadow: none;/,
    );
    expect(PRINT_CSS).toMatch(/\.print-doc\[data-window\] \{\s*gap: 0;\s*padding: 0;/);
  });

  it('gives Nastaliq a line height of at least 1.8 (NFR-URD-11)', () => {
    const leading = /--leading-urdu:\s*([\d.]+)/.exec(TOKENS_CSS);
    expect(Number(leading?.[1])).toBeGreaterThanOrEqual(1.8);
    expect(PRINT_CSS).toContain('line-height: var(--leading-text)');
  });

  it('sets the print type scale: Latin 8, 9, 10, 12, 16 pt and Urdu 10, 11, 12, 15, 20 pt', () => {
    const sizes = (script: string) =>
      [1, 2, 3, 4, 5].map((step) =>
        Number(new RegExp(`--print-${script}-${String(step)}:\\s*(\\d+)pt`).exec(TOKENS_CSS)?.[1]),
      );
    expect(sizes('latin')).toEqual([8, 9, 10, 12, 16]);
    expect(sizes('urdu')).toEqual([10, 11, 12, 15, 20]);
  });
});

describe.each(LANGUAGES)('the words the backend sends (%s)', (language) => {
  /** Every Fluent id of plan 14 contract 6, and the ids the layout and the print box use. */
  const IDS = [
    'print-seller-bill-title',
    'print-buyer-slip-title',
    'print-pukhta-slip-title',
    'print-fact-number',
    'print-fact-date',
    'print-fact-bill-date',
    'print-fact-truck',
    'print-party-seller',
    'print-party-buyer',
    'print-party-onetime-buyer',
    'print-gross-sale',
    'print-freight',
    'print-commission',
    'print-labour',
    'print-munshiana',
    'print-cash-paid',
    'print-store-rent',
    'print-phone-charge',
    'print-total-deductions',
    'print-net-sale',
    'print-seller-owes',
    'print-items-value',
    'print-buyer-commission',
    'print-laga',
    'print-buyer-pays',
    'print-in-words',
    'print-payment-cash',
    'print-payment-credit',
    'print-col-date',
    'print-col-buyer',
    'print-col-item',
    'print-col-packing',
    'print-col-qty',
    'print-col-weight',
    'print-col-rate',
    'print-col-amount',
    'print-page-label',
    'print-page-of',
    'print-printed-at',
    'print-cancelled-stamp',
    'print-paper-a4',
    'print-paper-a5',
    'print-paper-mm80',
    'settings-tab-printing',
    'err-print-windows-only',
    'err-printer-not-found',
    'err-print-failed',
  ];

  it.each(IDS)('has %s, with words', (id) => {
    const pattern = bundles[language].getMessage(id)?.value;
    expect(pattern, id).toBeTruthy();
    const errors: Error[] = [];
    const text = bundles[language].formatPattern(
      pattern ?? '',
      { number: 1, key: 'Ctrl+Enter', docs: 'x', printer: 'P' },
      errors,
    );
    expect(errors).toEqual([]);
    expect(text.trim()).not.toBe('');
  });

  it('every id the canned print data uses exists', () => {
    for (const doc of ['seller_bill', 'buyer_slips', 'pukhta_slip'] as const) {
      const data = printDataFor(doc, { language });
      const ids = [
        data.title_id,
        ...data.facts.map((fact) => fact.label_id),
        ...data.pages.flatMap((page) => [
          page.party.role_id,
          ...page.amounts.map((amount) => amount.label_id),
          ...(page.payment_id === null ? [] : [page.payment_id]),
          ...(page.words === null ? [] : [page.words.label_id]),
        ]),
      ];
      for (const id of ids) expect(bundles[language].hasMessage(id), id).toBe(true);
    }
  });

  it('uses the glossary words: the labels of the bill are the locked words of PRD 3', () => {
    const say = (id: string) =>
      bundles[language].formatPattern(bundles[language].getMessage(id)?.value ?? '');
    const expected: Record<string, [string, string]> = {
      'print-gross-sale': ['خام بکری', 'Gross Sale'],
      'print-net-sale': ['صافی بکری', 'Net Sale'],
      'print-freight': ['کرایہ', 'Freight'],
      'print-labour': ['مزدوری', 'Labour'],
      'print-munshiana': ['منشیانہ', 'Munshiana'],
      'print-cash-paid': ['نقد', 'Cash Paid'],
      'print-store-rent': ['کرایہ سٹور', 'Store Rent'],
      'print-phone-charge': ['فون', 'Phone Charge'],
      'print-laga': ['لاگا', 'Laga'],
    };
    for (const [id, [ur, en]] of Object.entries(expected)) {
      expect(say(id).replace(/[⁦-⁩]/g, '')).toBe(language === 'ur' ? ur : en);
    }
  });
});

describe('waiting for the print page', () => {
  it('does not wait for an animation frame, which a hidden window can pause for good', async () => {
    const frame = vi.fn(() => 0);
    vi.stubGlobal('requestAnimationFrame', frame);
    try {
      const root = document.createElement('div');
      document.body.append(root);
      const ready = await waitForPrintReady(['Inter'], 'x', root);
      expect(frame).not.toHaveBeenCalled();
      // jsdom has no layout: the document is as tall as the rectangle tests/setup.ts gives it.
      expect(ready).toEqual({ height_px: Math.ceil(root.getBoundingClientRect().height) });
      root.remove();
    } finally {
      vi.unstubAllGlobals();
    }
  });

  it('asks for every font by name, and goes on when one cannot load', async () => {
    const load = vi.fn((font: string) =>
      font.includes('Gulzar') ? Promise.reject(new Error('no file')) : Promise.resolve([]),
    );
    Object.defineProperty(document, 'fonts', {
      configurable: true,
      value: { load, ready: Promise.resolve() },
    });
    try {
      await waitForPrintReady(['Inter', 'Gulzar'], 'ا');
      expect(load.mock.calls.map((call) => call[0])).toEqual(['large "Inter"', 'large "Gulzar"']);
    } finally {
      Reflect.deleteProperty(document, 'fonts');
    }
  });
});

describe('the print commands', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    vi.mocked(invoke).mockResolvedValue(null);
  });

  it('send one { input } object each, as the backend contract says', async () => {
    const dataInput = {
      bikri_id: 1,
      doc: 'seller_bill',
      language: 'ur',
      only_buyer_id: null,
    } as const;
    await reportsBikriPrintData(dataInput);
    const printInput = {
      bikri_id: 1,
      docs: ['seller_bill'],
      language: 'en',
      copies: '2',
      to: 'pdf',
      only_buyer_id: 19,
    } as const;
    await reportsPrint({ ...printInput, docs: ['seller_bill'] });
    await reportsListPrinters();
    await settingsGetPrinting();
    const set = { ...printingSettings(), copies: '3' };
    await settingsSetPrinting(set);
    expect(vi.mocked(invoke).mock.calls).toEqual([
      ['reports_bikri_print_data', { input: dataInput }],
      ['reports_print', { input: { ...printInput, docs: ['seller_bill'] } }],
      ['reports_list_printers', undefined],
      ['settings_get_printing', undefined],
      ['settings_set_printing', { input: set }],
    ]);
  });
});
