// The quality of a sale line (درجہ, PRD FR-12.42, plan 58) against the backend's own answers
// (tests/fixtures/sale_grade_*.json): the seller bill writes the word after the item, the buyer slips
// show the words the backend wrote into each `detail` once, and the registers carry a grade column
// that holds the word of each line. Nothing is worked out here: the words are Fluent's.

import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { reportColumns } from '../../src/features/reports/columns';
import { BikriPrintDocument } from '../../src/features/bikri/print/BikriPrintDocument';
import { message, visible } from '../helpers/render';
import {
  GRADE_BUYER_SLIPS_UR,
  GRADE_R2,
  GRADE_R35_AKRAM,
  GRADE_SELLER_BILL_EN,
} from '../helpers/sale-grade-real';

const WORD = {
  first: 'glossary-darja-awwal',
  second: 'glossary-darja-doyam',
  third: 'glossary-darja-soyam',
} as const;

describe('the seller bill with a quality on each line (English, the backend answer)', () => {
  it('writes each item with its quality word after it, in the order of the lines', () => {
    const { container } = render(<BikriPrintDocument data={GRADE_SELLER_BILL_EN} />);
    const text = visible(container.textContent);
    for (const grade of ['first', 'second', 'third'] as const) {
      const word = message('en', WORD[grade]);
      expect(text).toContain(message('en', 'common-item-grade', { item: 'Apple', grade: word }));
    }
  });
});

describe('the buyer slips with a quality on each line (Urdu, the backend answer)', () => {
  it('shows the words the backend wrote in each detail, and the item is not named twice', () => {
    const lines = GRADE_BUYER_SLIPS_UR.pages.flatMap((page) => page.lines);
    expect(lines.length).toBeGreaterThan(0);
    for (const line of lines) {
      const word = message('ur', WORD[line.grade ?? 'first']);
      expect(line.detail).toContain(`${line.item} ${word}`);
      expect(line.detail.split(word)).toHaveLength(2);
    }
  });
});

describe.each([
  ['R2', GRADE_R2, 'item'],
  ['R35 slip', GRADE_R35_AKRAM, 'amount'],
] as const)('the %s register', (_name, page, before) => {
  const keys = page.columns.map((column) => column.key);

  it('has a grade column, a label column, in the place the backend put it', () => {
    const at = keys.indexOf('grade');
    expect(at).toBeGreaterThan(-1);
    expect(at).toBeGreaterThan(keys.indexOf(before));
    expect(page.columns[at]?.title_id).toBe('report-col-grade');
    expect(page.columns[at]?.kind).toBe('label');
  });

  it('holds a word of the quality for each line, and the words exist in both languages', () => {
    const at = keys.indexOf('grade');
    // Only a line of goods has a quality; a total row has an empty cell in that column.
    const labels = page.rows.filter((row) => row.kind === 'normal').map((row) => row.cells[at]);
    expect(labels.length).toBeGreaterThan(0);
    for (const cell of labels) {
      expect(cell?.type).toBe('label');
      if (cell?.type !== 'label') continue;
      for (const language of ['ur', 'en'] as const) {
        expect(message(language, cell.id)).not.toBe('');
      }
    }
  });
});

describe('the columns on the screen when a line has a quality (no sideways scroll, NFR-USE-15)', () => {
  const shown = (id: 'R2' | 'R35', page: typeof GRADE_R2, step: 'compact' | 'normal' | 'wide') =>
    reportColumns(id, page.columns, step).map((column) => column.field);

  it('R2 keeps the quality and the buyer total at every step, and gives the day and the value a wider step', () => {
    for (const step of ['compact', 'normal', 'wide'] as const) {
      expect(shown('R2', GRADE_R2, step)).toContain('grade');
      expect(shown('R2', GRADE_R2, step)).toContain('buyer_total');
    }
    expect(shown('R2', GRADE_R2, 'compact')).not.toContain('date');
    expect(shown('R2', GRADE_R2, 'normal')).toContain('date');
    expect(shown('R2', GRADE_R2, 'normal')).not.toContain('value');
    expect(shown('R2', GRADE_R2, 'wide')).toContain('value');
  });

  it("R35's slip keeps the quality and drops the packing word until the wide step", () => {
    expect(shown('R35', GRADE_R35_AKRAM, 'normal')).toContain('grade');
    expect(shown('R35', GRADE_R35_AKRAM, 'normal')).not.toContain('packing');
    expect(shown('R35', GRADE_R35_AKRAM, 'wide')).toContain('packing');
  });
});
