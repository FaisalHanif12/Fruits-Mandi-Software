// The quality of a sale line (درجہ, PRD FR-12.42) after the item in the ledger details: R13 and R14
// write "(سیب اول ...)" in the print and on screen, and a line with no quality reads exactly as it
// did. The words are Fluent's, in the print's own language. The shared helper is tested for both
// languages; the print of R13 is the backend's own answer with the quality put on its sale line.

import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import { ReportPrintDocument } from '../../src/features/reports/print/ReportPrintDocument';
import { gradedItem } from '../../src/shared/format';
import { printFrameFor } from '../helpers/print-frame';
import { PRINT_R13_AKRAM_UR } from '../helpers/reports-real';
import { message, visible } from '../helpers/render';

/** The print data with the quality put on every sale line of it (the backend sends it that way). */
function graded(data: ReportPrintData, grade: 'first' | 'second' | 'third'): ReportPrintData {
  const copy = structuredClone(data);
  const mark = (value: unknown): void => {
    if (Array.isArray(value)) {
      value.forEach(mark);
    } else if (typeof value === 'object' && value !== null) {
      const record = value as Record<string, unknown>;
      if (record['kind'] === 'sale') record['grade'] = grade;
      Object.values(record).forEach(mark);
    }
  };
  mark(copy.rows);
  return copy;
}

function detailOf(data: ReportPrintData): string {
  const frame = {
    ...printFrameFor({ language: data.language, title_id: data.title_id, facts: data.filters }),
    dir: data.dir,
    urdu_digits: data.urdu_digits,
  };
  const { container } = render(<ReportPrintDocument data={data} frame={frame} applyRoot={false} />);
  return visible(container.querySelector('tbody tr .print-lines')?.textContent);
}

describe('gradedItem', () => {
  const say = (language: 'ur' | 'en') => (id: string, vars?: Record<string, string>) =>
    message(language, id, vars ?? {});

  it('writes the quality word after the item in both languages, and the item alone without one', () => {
    expect(gradedItem(say('ur'), 'سیب', 'first')).toBe('سیب اول');
    expect(gradedItem(say('ur'), 'سیب', 'second')).toBe('سیب دوم');
    expect(gradedItem(say('ur'), 'سیب', 'third')).toBe('سیب سوم');
    expect(gradedItem(say('en'), 'Apple', 'first')).toBe('Apple First');
    expect(gradedItem(say('en'), 'Apple', 'second')).toBe('Apple Second');
    expect(gradedItem(say('en'), 'Apple', 'third')).toBe('Apple Third');
    expect(gradedItem(say('ur'), 'سیب', undefined)).toBe('سیب');
    expect(gradedItem(say('en'), 'Apple', undefined)).toBe('Apple');
  });
});

describe('the ledger of a buyer in print (R13)', () => {
  const plain = detailOf(PRINT_R13_AKRAM_UR);

  it('reads exactly as before when the line has no quality', () => {
    expect(plain).toContain('کیلا');
    expect(plain).not.toContain(message('ur', 'glossary-darja-awwal'));
  });

  it('writes the quality after the item when the line has one', () => {
    const text = detailOf(graded(PRINT_R13_AKRAM_UR, 'second'));
    expect(text).toContain(`کیلا ${message('ur', 'glossary-darja-doyam')}`);
    // Only the item changed: the same figures, in the same order.
    expect(text.replace(` ${message('ur', 'glossary-darja-doyam')}`, '')).toBe(plain);
  });
});
