// ادھاری تفصیل (R5) printed with موقع پر جمع and کھاتے میں باقی (D8 Feature 4 and 8, plans 65 and 68, PRD
// FR-12.44, FR-19.13): each buyer's slip ends with what he paid on the spot and what is left, and the
// totals row of the slip holds both. It is drawn from the backend's own print data of the D8 example
// world (one بکری, three buyers who paid all, a part and nothing), in Urdu and English. The page lays
// out what it is given: every figure is the backend's and nothing is added up here.

import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { BuyerBillsPrintData } from '../../src/bindings/BuyerBillsPrintData';
import { BuyerBillsPrintDocument } from '../../src/features/reports/print/BuyerBillsPrintDocument';
import { printFrameFor } from '../helpers/print-frame';
import { message, visible } from '../helpers/render';
import {
  BUYER_BILLS_BIKRI_UR,
  BUYER_BILLS_PAID_EN,
  BUYER_BILLS_PAID_UR,
  BUYER_BILLS_PAID_UR_REQUEST,
} from '../helpers/reports-sales-real';

function draw(data: BuyerBillsPrintData) {
  const frame = printFrameFor({
    language: data.language,
    title_id: data.title_id,
    facts: data.filters,
  });
  const { container } = render(
    <BuyerBillsPrintDocument data={data} frame={{ ...frame, dir: data.dir }} />,
  );
  return Array.from(container.querySelectorAll<HTMLElement>('.print-sheet'));
}

/** The money cells of the row of a slip, as the reader sees them: the figure cells after the serial. */
const rowFigures = (sheet: HTMLElement) =>
  Array.from(sheet.querySelectorAll('tbody tr td.print-num'))
    .slice(1)
    .map((cell) => visible(cell.textContent));

/** The names of the three buyers of the example, in each language, in the order the backend sent. */
const EXAMPLE = [
  { ur: 'انس', en: 'Anas', value: '45,000', total: '48,100', paid: '10,000', left: '38,100' },
  { ur: 'مانی', en: 'Mani', value: '20,000', total: '21,360', paid: '21,360', left: '0' },
  { ur: 'ہارون', en: 'Haroon', value: '117,000', total: '125,060', paid: '0', left: '125,060' },
] as const;

describe.each([
  ['Urdu', 'ur', BUYER_BILLS_PAID_UR],
  ['English', 'en', BUYER_BILLS_PAID_EN],
] as const)('the slips of the D8 example in %s', (_name, language, data) => {
  it('has the paid and left columns after the buyer total, with their own words', () => {
    expect(data.columns.map((column) => column.key)).toEqual([
      'serial',
      'detail',
      'value',
      'buyer_total',
      'paid',
      'left',
    ]);
    const sheets = draw(data);
    expect(sheets).toHaveLength(3);
    const headings = Array.from(sheets[0]?.querySelectorAll('thead th') ?? []).map((th) =>
      visible(th.textContent),
    );
    expect(headings.slice(-2)).toEqual([
      message(language, 'report-col-paid-on-spot'),
      message(language, 'report-col-left-on-account'),
    ]);
  });

  it('shows each buyer with the money he paid on the spot and what is left, as the backend sent them', () => {
    const sheets = draw(data);
    expect(sheets).toHaveLength(EXAMPLE.length);
    for (const sheet of sheets) {
      const text = visible(sheet.textContent);
      const figures = rowFigures(sheet);
      const found = EXAMPLE.find((buyer) => text.includes(buyer[language]));
      expect(found, text).toBeDefined();
      if (found === undefined) continue;
      // value, buyer total, paid, left: the four money cells of the row, in the order of the columns.
      expect(figures).toEqual([found.value, found.total, found.paid, found.left]);
      // The slip's own totals row holds the same four (one buyer, one line).
      const totals = Array.from(sheet.querySelectorAll('tfoot td'))
        .map((cell) => visible(cell.textContent))
        .filter((cell) => /\d/.test(cell));
      expect(totals).toEqual([found.value, found.total, found.paid, found.left]);
    }
  });

  it('writes انس exactly as the example: (آم 50 . 900), 45,000, 48,100, 10,000, 38,100', () => {
    const sheet = draw(data).find((candidate) =>
      visible(candidate.textContent).includes(EXAMPLE[0][language]),
    );
    expect(sheet).toBeDefined();
    const text = visible(sheet?.textContent);
    const mango = language === 'ur' ? 'آم' : 'Mango';
    expect(text).toContain(`(${mango} 50 . 900)`);
    for (const figure of ['45,000', '48,100', '10,000', '38,100']) expect(text).toContain(figure);
  });

  it('writes every number with the digits 0-9, grouped', () => {
    const text = draw(data)
      .map((sheet) => visible(sheet.textContent))
      .join(' ');
    expect(text).not.toMatch(/[۰-۹٠-٩]/);
    expect(text).toMatch(/\d{1,3}(?:,\d{3})+/);
  });
});

describe('R5 for this بکری only (plan 68)', () => {
  it('asks the backend for one بکری, which is in the header facts, and draws only its buyers', () => {
    expect(BUYER_BILLS_PAID_UR_REQUEST.filters.bikri_id).toBe(1);
    expect(BUYER_BILLS_BIKRI_UR.filters.map((fact) => fact.label_id)).toContain(
      'report-filter-bikri',
    );
    const sheets = draw(BUYER_BILLS_BIKRI_UR);
    expect(sheets).toHaveLength(BUYER_BILLS_BIKRI_UR.slips.length);
    // The slips hold both columns too.
    expect(BUYER_BILLS_BIKRI_UR.columns.slice(-2).map((column) => column.key)).toEqual([
      'paid',
      'left',
    ]);
  });
});
