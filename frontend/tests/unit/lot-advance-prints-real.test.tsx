// The پیشگی typed on a lot on the paper (plan 95 item 11, PRD FR-12.52, FR-19.12, FR-19.15), drawn from the backend's
// REAL answers of the worked case (`tests/helpers/lot-advance-real.ts`), in Urdu and in English: the seller's day slip
// (R35) and the seller bills made from slips show a line پیشگی 20,000 right after the cash paid (نقد ادائیگی 30,000)
// and before the rest of the credit; the seller bill of the sale book (R1) shows it after the net sale, outside the
// deductions; the whole sale print's sellers table has it as its last column, with a total. Every figure and word is
// the backend's; the page lays them out and adds nothing up (CLAUDE.md rule 1).

import { render } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import { BikriPrintDocument } from '../../src/features/bikri/print/BikriPrintDocument';
import { DaySlipsDocument } from '../../src/features/reports/print/DaySlipsDocument';
import { ReportPrintDocument } from '../../src/features/reports/print/ReportPrintDocument';
import { SalePrintDocument } from '../../src/features/reports/print/SalePrintDocument';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import { bareDigits } from '../helpers/bare-digits';
import {
  LOT_ADVANCE_BILLS_EN,
  LOT_ADVANCE_BILLS_UR,
  LOT_ADVANCE_R1_EN,
  LOT_ADVANCE_R1_UR,
  LOT_ADVANCE_SALE_EN,
  LOT_ADVANCE_SALE_UR,
  LOT_ADVANCE_SLIP_EN,
  LOT_ADVANCE_SLIP_UR,
} from '../helpers/lot-advance-real';
import { printFrameFor } from '../helpers/print-frame';
import { message, visible } from '../helpers/render';

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  document.documentElement.style.removeProperty(FOOTER_PROPERTY);
});

const textOf = (element: Element | null | undefined) => visible(element?.textContent);
const cellTexts = (row: Element | null | undefined) =>
  [...(row?.querySelectorAll('th, td') ?? [])].map(textOf);

/** The labelled lines of a slip: each label with its figure. */
function linesOf(root: ParentNode): [string, string][] {
  return [...root.querySelectorAll('.print-slip-lines tr')].map((line) => [
    textOf(line.querySelector('th')),
    textOf(line.querySelector('td')),
  ]);
}

function drawSlip(data: ReportPrintData) {
  useLanguageStore.setState({ language: data.language });
  const frame = {
    ...printFrameFor({ language: data.language, title_id: data.title_id, facts: data.filters }),
    dir: data.dir,
  };
  return render(<ReportPrintDocument data={data} frame={frame} />).container;
}

describe.each(LANGUAGES)('the lot advance on the seller’s papers (%s)', (language: Language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const ur = language === 'ur';

  it('the day slip (R35): net sale, the cash paid, the advance, the rest of the credit, today’s left, the balances', () => {
    const lines = linesOf(drawSlip(ur ? LOT_ADVANCE_SLIP_UR : LOT_ADVANCE_SLIP_EN));
    expect(lines.map(([label]) => label)).toEqual(
      [
        'report-r35-gross',
        'report-r35-net-sale',
        'report-r35-paid-today-seller',
        'report-r35-advance',
        'report-r35-sale-rest',
        'report-r35-today-left-seller',
        'report-r35-previous',
        'report-r35-new-balance',
      ].map((id) => m(id)),
    );
    const byLabel = new Map(lines);
    // The cash paid and the advance are two lines, each with its own words and figure; the rest follows them.
    expect(byLabel.get(m('report-r35-paid-today-seller'))).toBe('30,000');
    expect(byLabel.get(m('report-r35-advance'))).toBe('20,000');
    expect(byLabel.get(m('report-r35-sale-rest'))).toBe('32,160');
    expect(m('report-r35-paid-today-seller')).toBe(m('glossary-naqd-adaigi'));
    expect(m('report-r35-advance')).toBe(m('glossary-peshgi'));
  });

  it('the seller bills made from slips carry the same advance line, in its place, with no raw id and no bare digit', () => {
    useLanguageStore.setState({ language });
    const data = ur ? LOT_ADVANCE_BILLS_UR : LOT_ADVANCE_BILLS_EN;
    const page = render(<DaySlipsDocument data={data} variant="seller-bills" />).container;
    const labels = linesOf(page).map(([label]) => label);
    const at = labels.indexOf(m('report-r35-paid-today-seller'));
    expect(at).toBeGreaterThan(-1);
    expect(labels[at + 1]).toBe(m('report-r35-advance'));
    expect(labels[at + 2]).toBe(m('report-r35-sale-rest'));
    expect(textOf(page)).not.toMatch(/\b(report|print|common)-[a-z0-9-]+/);
    expect(bareDigits(page)).toEqual([]);
  });

  it('the seller bill of the sale book (R1): the advance follows the net sale and is no deduction', () => {
    useLanguageStore.setState({ language });
    const data = ur ? LOT_ADVANCE_R1_UR : LOT_ADVANCE_R1_EN;
    const page = render(<BikriPrintDocument data={data} />).container;
    const rows = [...page.querySelectorAll('.print-amounts tr')].map((row): [string, string] => [
      textOf(row.querySelector('th')),
      textOf(row.querySelector('td')),
    ]);
    const labels = rows.map(([label]) => label);
    expect(labels.at(-1)).toBe(m('print-seller-advance'));
    expect(labels.at(-2)).toBe(m('print-net-sale'));
    // The words are the glossary's; the cash paid reads نقد ادائیگی; the figures are the backend's.
    expect(m('print-seller-advance')).toBe(m('glossary-peshgi'));
    const byLabel = new Map(rows);
    expect(byLabel.get(m('print-seller-advance'))).toBe('20,000');
    expect(byLabel.get(m('print-cash-paid'))).toBe('30,000');
    expect(byLabel.get(m('print-net-sale'))).toBe('22,900');
    // It is not in the total deductions (30,000, the cash paid alone).
    expect(byLabel.get(m('print-total-deductions'))).toBe('30,000');
    expect(textOf(page)).not.toMatch(/\bprint-[a-z0-9-]+/);
  });

  it('the whole sale print: the sellers table ends with the advance column, a total under it, and nothing else moves', () => {
    useLanguageStore.setState({ language });
    const data = ur ? LOT_ADVANCE_SALE_UR : LOT_ADVANCE_SALE_EN;
    const page = render(<SalePrintDocument data={data} />).container;
    const sellers = page.querySelectorAll('table')[0];
    const heads = [...(sellers?.querySelectorAll('thead th') ?? [])].map(textOf);
    expect(heads.at(-1)).toBe(m('print-sale-col-advance'));
    expect(heads.at(-2)).toBe(m('report-col-net'));
    expect(heads).toContain(m('report-col-cash-paid'));
    expect(m('report-col-cash-paid')).toBe(m('glossary-naqd-adaigi'));
    const rows = [...(sellers?.querySelectorAll('tbody tr') ?? [])].map(cellTexts);
    // Two lots: the first has no advance (0), the second has 20,000; each cell follows its column.
    expect(rows.map((row) => row.at(-1))).toEqual(['0', '20,000']);
    expect(rows.map((row) => row.at(-2))).toEqual(['9,260', '22,900']);
    expect(cellTexts(sellers?.querySelector('tfoot tr')).at(-1)).toBe('20,000');
    // The advance is in no deduction and not in the net: the deductions are the cash paid alone.
    expect(rows[1]?.at(-3)).toBe('30,000');
    expect(textOf(page)).not.toMatch(/\b(report|print|common)-[a-z0-9-]+/);
    expect(bareDigits(page)).toEqual([]);
  });
});
