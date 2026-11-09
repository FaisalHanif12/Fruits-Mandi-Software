// The quality of a sale line (درجہ, PRD FR-12.42, plan 58) on the prints of a بکری: the seller bill and
// the buyer slip write its word after the item (the slip's words are the backend's own, so only the bill is drawn here), in the print's own language, and a line with no
// quality is written exactly as before. The data is the backend's own answer of the worked day
// with the quality put on its lines (the backend sends `grade` on a line only when there is one).

import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { PrintData } from '../../src/bindings/PrintData';
import type { SaleGrade } from '../../src/bindings/SaleGrade';
import { BikriPrintDocument } from '../../src/features/bikri/print/BikriPrintDocument';
import { realFor } from '../helpers/print-real';
import { message, visible } from '../helpers/render';

const WORD: Record<SaleGrade, string> = {
  first: 'glossary-darja-awwal',
  second: 'glossary-darja-doyam',
  third: 'glossary-darja-soyam',
};

/** The print data with the quality put on every line of every page. */
function graded(data: PrintData, grade: SaleGrade): PrintData {
  const copy = structuredClone(data);
  for (const page of copy.pages) for (const line of page.lines) line.grade = grade;
  return copy;
}

/** The text of the item cells of the goods table, one for each line. */
function itemCells(data: PrintData): string[] {
  const { container } = render(<BikriPrintDocument data={data} />);
  const itemHead = message(data.language, 'print-col-item');
  // The goods table is the one that has the item column; a slip has other tables too.
  const goods = [...container.querySelectorAll('table')].find((table) =>
    [...table.querySelectorAll('thead th')].some((cell) => visible(cell.textContent) === itemHead),
  );
  if (goods === undefined) return [];
  const heads = [...goods.querySelectorAll('thead th')].map((cell) => visible(cell.textContent));
  const at = heads.indexOf(itemHead);
  return [...goods.querySelectorAll('tbody tr')]
    .map((row) => visible(row.querySelectorAll('td')[at]?.textContent))
    .filter((text) => text !== '');
}

// The buyer slips (R5) are lines of words the backend writes itself, with the quality already in them
// (its own tests); the seller bill has a goods table the screen draws.
describe.each(['seller_bill'] as const)('the %s', (doc) => {
  describe.each(['ur', 'en'] as const)('in %s', (language) => {
    const data = realFor(doc, language);

    it('writes the item alone when the line has no quality, as before', () => {
      const cells = itemCells(data);
      expect(cells.length).toBeGreaterThan(0);
      for (const cell of cells) {
        for (const grade of Object.values(WORD)) {
          expect(cell).not.toContain(message(language, grade));
        }
      }
    });

    it('writes the quality word after the item when the line has one', () => {
      const plain = itemCells(data);
      const marked = itemCells(graded(data, 'second'));
      const word = message(language, WORD.second);
      expect(marked).toHaveLength(plain.length);
      marked.forEach((cell, index) => {
        expect(cell).toBe(
          message(language, 'common-item-grade', { item: plain[index] ?? '', grade: word }),
        );
      });
    });
  });
});
