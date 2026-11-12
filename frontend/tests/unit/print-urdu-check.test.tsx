// What the Urdu check of the prints in headless Chromium found and fixed (CLAUDE.md, Mac phase; plan 71
// follow-up): four things that made Urdu print badly although every test of the text passed.
//
// 1. The header facts. A party's name with his code ("0 اکرم ٹول ٹیکس") was drawn inside `Code`, which is Inter,
//    and Inter has no Urdu letters, so the name was drawn in whatever Urdu font the computer had. A fact with
//    Urdu letters is now `PrintText` (the text font; its number is a number); every other fact is still one code.
// 2. The side word of a balance (بنام, جمع) stood in a number cell, so it was Inter too. It is a word now.
// 3. A code, a phone or a truck number broke at its hyphen ("0300-" and "0000000" on two lines).
// 4. The heading of a day of پچھلی بکری ("09-10-2026: بکری 2") sat in one narrow column and broke in pieces:
//    a sentence in a heading row now spreads over the empty cells after it.
// The fonts themselves were read from Chromium (CDP) for each page; here are the rules that keep them right.

import { render } from '@testing-library/react';
import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { PastPrintDocument } from '../../src/features/bikri/print/PastPrintDocument';
import { ReportPrintDocument } from '../../src/features/reports/print/ReportPrintDocument';
import { printFrameFor } from '../helpers/print-frame';
import { pastPrintDataFor } from '../helpers/past-print';
import { visible } from '../helpers/render';
import { PRINT_R13_AKRAM_UR } from '../helpers/reports-real';

const SRC = path.resolve(import.meta.dirname, '../../src');
const PRINT_CSS = fs.readFileSync(path.join(SRC, 'shared/print/print.css'), 'utf8');

const PAST_INPUT = {
  from: '03-10-2026',
  to: '09-10-2026',
  seller_id: null,
  item_id: null,
  status: null,
  show_finished: false,
  search: null,
  all: false,
  with_today: true,
  language: 'ur' as const,
  paper: 'a4' as const,
};

function showReport() {
  const frame = {
    ...printFrameFor({
      language: PRINT_R13_AKRAM_UR.language,
      title_id: PRINT_R13_AKRAM_UR.title_id,
      facts: PRINT_R13_AKRAM_UR.filters,
    }),
    dir: PRINT_R13_AKRAM_UR.dir,
  };
  const { container } = render(<ReportPrintDocument data={PRINT_R13_AKRAM_UR} frame={frame} />);
  return container;
}

function showPast() {
  const data = pastPrintDataFor(PAST_INPUT);
  const frame = {
    ...printFrameFor({ language: 'ur', title_id: data.title_id, facts: data.filters }),
    dir: data.dir,
  };
  const { container } = render(<PastPrintDocument data={data} frame={frame} />);
  return { container, data };
}

describe('the facts of the print header in Urdu', () => {
  it('draw a party name in the text font, with its code a number, and not inside a code (Inter)', () => {
    const facts = showReport().querySelector('.print-facts');
    expect(facts).not.toBeNull();
    const party = Array.from(facts?.querySelectorAll('.print-fact') ?? []).find((fact) =>
      visible(fact.textContent).includes('اکرم ٹول ٹیکس'),
    );
    expect(party).toBeDefined();
    const value = party?.querySelector('dd');
    // The name is not in a code element: that is the font of digits and Latin letters only.
    const inCode = Array.from(value?.querySelectorAll('[data-kind="code"]') ?? []).some((code) =>
      /\p{Script=Arabic}/u.test(code.textContent),
    );
    expect(inCode).toBe(false);
    expect(visible(value?.textContent)).toBe('0 اکرم ٹول ٹیکس');
    // The code of the party is a number of its own, written left to right.
    const numbers = Array.from(value?.querySelectorAll('bdi[dir="ltr"]') ?? []).map((bdi) =>
      visible(bdi.textContent),
    );
    expect(numbers).toContain('0');
  });

  it('keep a date as one code, written left to right', () => {
    const facts = showReport().querySelector('.print-facts');
    const dates = Array.from(facts?.querySelectorAll('dd [data-kind="code"]') ?? []).map((code) =>
      visible(code.textContent),
    );
    expect(dates.every((text) => /^[0-9-]+$/.test(text))).toBe(true);
  });
});

describe('the print of پچھلی بکری in Urdu', () => {
  it('writes the heading of a day in one cell spread over the empty cells after it, not in a narrow column', () => {
    const { container, data } = showPast();
    const columns = data.columns.length;
    const headings = Array.from(container.querySelectorAll('tr[data-row="group"]'));
    // Today, the day with بکری, and the days with none.
    expect(headings.length).toBeGreaterThanOrEqual(3);
    for (const row of headings) {
      const cells = Array.from(row.querySelectorAll('td'));
      const spanning = cells.find((cell) => cell.colSpan > 1);
      expect(spanning, 'a sentence that spreads').toBeDefined();
      // Every column of the row is a cell or is spread over: nothing is lost, nothing is added.
      const total = cells.reduce((sum, cell) => sum + cell.colSpan, 0);
      expect(total).toBe(columns);
    }
    // The heading of a day with بکری keeps its خام بکری in its own column after the sentence.
    const dayWithRows = headings.find((row) => visible(row.textContent).includes('09-10-2026'));
    expect(dayWithRows).toBeDefined();
    expect(visible(dayWithRows?.textContent)).toContain('58,800');
  });

  it('keeps one cell for each column in the rows of بکری', () => {
    const { container, data } = showPast();
    const rows = Array.from(container.querySelectorAll('tr[data-row="normal"]'));
    expect(rows.length).toBeGreaterThan(0);
    for (const row of rows) {
      expect(row.querySelectorAll('td')).toHaveLength(data.columns.length);
      expect(Array.from(row.querySelectorAll('td')).every((cell) => cell.colSpan === 1)).toBe(true);
    }
  });

  it('never breaks a truck number at its hyphen, and leaves a text with spaces free to wrap', () => {
    const { container } = showPast();
    const truck = Array.from(container.querySelectorAll('bdi')).find(
      (bdi) => visible(bdi.textContent) === 'LES-1234' && bdi.className.includes('print-nobreak'),
    );
    expect(truck, 'the truck number in a bdi that does not break').toBeDefined();
    expect(PRINT_CSS).toMatch(/\.print-nobreak\s*{\s*white-space:\s*nowrap;/);
  });
});

describe('the print stylesheet', () => {
  it('keeps a code, a phone or a truck number on one line', () => {
    expect(PRINT_CSS).toMatch(
      /\[data-kind='code'\],\s*\.print-doc \.print-nobreak\s*{\s*white-space:\s*nowrap;/,
    );
  });

  it('sets the side word of a balance in the text font, not in the font of its number cell', () => {
    const rule = /\.print-doc \.print-balance-side\s*{([^}]*)}/.exec(PRINT_CSS)?.[1] ?? '';
    expect(rule).toMatch(/font-family:\s*var\(--font-text\);/);
    expect(rule).toMatch(/font-size:\s*var\(--print-text-3\);/);
  });
});
