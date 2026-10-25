// The party day sheet printed (R35, PRD FR-19.12, FR-19.13, E2E-43, Umar 20-10-2026 and 26-10-2026):
// the client's two layouts drawn from the backend's real print data, in Urdu and in English, on A4
// and on A5. One party's slip has its lines first and what the day comes to under them; the
// all-parties table has the buyers, then the sellers, with a blank collector column and the footer
// lines under it. Every number is English 0-9 in its own left-to-right isolate, and the page adds
// nothing up.

import { render } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import { ReportPrintDocument } from '../../src/features/reports/print/ReportPrintDocument';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import { bareDigits } from '../helpers/bare-digits';
import { printFrameFor } from '../helpers/print-frame';
import { message, visible } from '../helpers/render';
import {
  PRINT_R35_SLIP_AKRAM_EN,
  PRINT_R35_SLIP_AKRAM_UR,
  PRINT_R35_SLIP_RAHIM_EN,
  PRINT_R35_TABLE_EN,
  PRINT_R35_TABLE_UR,
} from '../helpers/reports-party-real';

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  document.documentElement.style.removeProperty(FOOTER_PROPERTY);
});

function frameOf(data: ReportPrintData) {
  const frame = printFrameFor({
    language: data.language,
    title_id: data.title_id,
    facts: data.filters,
  });
  return { ...frame, dir: data.dir, urdu_digits: data.urdu_digits };
}

function show(data: ReportPrintData) {
  const result = render(<ReportPrintDocument data={data} frame={frameOf(data)} />);
  const root = result.container.querySelector<HTMLElement>('.print-doc');
  if (root === null) throw new Error('no print document');
  return { ...result, root };
}

function must<T extends Element>(element: T | null): T {
  if (element === null) throw new Error('missing element');
  return element;
}

const textOf = (element: Element | null | undefined) => visible(element?.textContent);

const labelsOf = (root: HTMLElement) =>
  Array.from(root.querySelectorAll('.print-summary-lines dt')).map(textOf);

describe.each([
  ['English', PRINT_R35_SLIP_AKRAM_EN, 'en', 'ltr'],
  ['Urdu', PRINT_R35_SLIP_AKRAM_UR, 'ur', 'rtl'],
] as const)('the buyer slip of اکرم in %s', (_name, data, language, dir) => {
  it('is a page in the print language and direction, with English digits', () => {
    const { root } = show(data);
    expect(root.getAttribute('lang')).toBe(language);
    expect(root.getAttribute('dir')).toBe(dir);
    expect(root.getAttribute('data-paper')).toBe('a4');
    expect(root.getAttribute('data-digits')).toBe('latin');
  });

  it('has the firm and the title in the one header, and names the party with his short code', () => {
    const { root } = show(data);
    expect(root.querySelectorAll('.print-header')).toHaveLength(1);
    const header = textOf(root.querySelector('.print-header'));
    expect(header).toContain(frameOf(data).header.firm.name);
    expect(header).toContain(message(language, 'report-r35-title'));
    // The day and the party are the facts of the header, as the backend wrote them.
    const facts = textOf(root.querySelector('.print-facts'));
    expect(facts).toContain(message(language, 'report-filter-day'));
    expect(facts).toContain('10-10-2026');
    expect(facts).toContain(language === 'ur' ? '1 اکرم ٹول ٹیکس' : '1 Akram Toll Tax');
  });

  it('has the lines first, then what the day comes to under them, in the client’s order', () => {
    const { root } = show(data);
    const table = must(root.querySelector('.print-table'));
    const summary = must(root.querySelector('.print-summary-lines'));
    expect(table.compareDocumentPosition(summary) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(labelsOf(root)).toEqual(data.summary.map((line) => message(language, line.label_id)));
    // The slip has its total نگ in the footer and no totals row of its own.
    expect(root.querySelector('tfoot')).toBeNull();
    const last = textOf(root.querySelectorAll('.print-summary-lines .print-fact')[4]);
    expect(last).toContain('2,880');
  });

  it('writes the line as the quantity, the item, the rate and the amount with commission and لاگا', () => {
    const { root } = show(data);
    const cells = Array.from(root.querySelectorAll('tbody tr td')).map(textOf);
    expect(cells).toEqual(['20', language === 'ur' ? 'پیٹی کیلا' : 'Box Banana', '600', '12,880']);
  });

  it('draws every number in its own left-to-right isolate, none bare in the words', () => {
    const { root } = show(data);
    expect(bareDigits(root)).toEqual([]);
  });
});

describe('the seller slip of رحیم in English', () => {
  const data = PRINT_R35_SLIP_RAHIM_EN;

  it('ends with his balance, and has a blank rate cell for the lot', () => {
    const { root } = show(data);
    const labels = labelsOf(root);
    expect(labels.at(-1)).toBe(message('en', 'report-r35-balance'));
    expect(labels).not.toContain(message('en', 'report-r35-remaining'));
    const cells = Array.from(root.querySelectorAll('tbody tr td')).map(textOf);
    expect(cells).toEqual(['50', 'Box Banana', '', '24,316']);
    expect(bareDigits(root)).toEqual([]);
  });
});

describe.each([
  ['English', PRINT_R35_TABLE_EN, 'en'],
  ['Urdu', PRINT_R35_TABLE_UR, 'ur'],
] as const)('the all-parties table in %s', (_name, data, language) => {
  it('has the seven columns with a blank collector column, and a row for each party', () => {
    const { root } = show(data);
    const heads = Array.from(root.querySelectorAll('thead th')).map(textOf);
    expect(heads).toEqual(data.columns.map((column) => message(language, column.title_id)));
    expect(heads).toHaveLength(7);
    const rows = Array.from(root.querySelectorAll('tbody tr'));
    expect(rows).toHaveLength(7);
    // The collector writes the last column by hand.
    for (const row of rows) expect(textOf(Array.from(row.querySelectorAll('td')).at(-1))).toBe('');
  });

  it('has the buyers’ and the sellers’ group rows, with the buyers’ total 21,564 as the balance', () => {
    const { root } = show(data);
    const groups = Array.from(root.querySelectorAll('tbody tr[data-row="group"]'));
    expect(groups).toHaveLength(2);
    expect(textOf(groups[0]?.querySelector('td'))).toBe(message(language, 'report-r35-buyers'));
    expect(textOf(groups[1]?.querySelector('td'))).toBe(message(language, 'report-r35-sellers'));
    expect(textOf(groups[0])).toContain('21,564');
  });

  it('writes the short code in its own column and the party as his name only, with no totals row', () => {
    const { root } = show(data);
    const codes = Array.from(root.querySelectorAll('tbody tr td:nth-child(1)')).map(textOf);
    const names = Array.from(root.querySelectorAll('tbody tr td:nth-child(2)')).map(textOf);
    expect(codes).toContain('1');
    expect(names).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    // A one-time buyer has his full code in the code column and his phone after his name, once each.
    const shahid = names.find((name) => name.includes('0300-0000000'));
    expect(shahid).toBe(language === 'ur' ? 'شاہد 0300-0000000' : 'Shahid 0300-0000000');
    expect(codes).toContain('03049999');
    expect(root.querySelector('tfoot')).toBeNull();
  });

  it('keeps the code column on one line, so an eight-digit code never breaks', () => {
    const { root } = show(data);
    const cell = must(root.querySelector('tbody tr:not([data-row="group"]) td:nth-child(1)'));
    expect(cell.classList.contains('print-nowrap')).toBe(true);
  });

  it('puts the footer lines under the table, buyers’ collection first', () => {
    const { root } = show(data);
    const table = must(root.querySelector('.print-table'));
    const summary = must(root.querySelector('.print-summary-lines'));
    expect(table.compareDocumentPosition(summary) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(labelsOf(root)).toEqual(data.summary.map((line) => message(language, line.label_id)));
    expect(labelsOf(root)[0]).toBe(message(language, 'report-r35-collection'));
  });

  it('draws every number in its own left-to-right isolate, none bare in the words', () => {
    const { root } = show(data);
    expect(bareDigits(root)).toEqual([]);
  });
});

describe('the paper', () => {
  it.each(['a4', 'a5'] as const)('the slip and the table are the same page on %s', (paper) => {
    for (const data of [PRINT_R35_SLIP_AKRAM_EN, PRINT_R35_TABLE_UR]) {
      const { root, unmount } = show({ ...data, paper });
      expect(root.getAttribute('data-paper')).toBe(paper);
      expect(root.querySelectorAll('.print-sheet')).toHaveLength(1);
      unmount();
    }
  });
});
