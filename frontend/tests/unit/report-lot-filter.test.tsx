// The lot filter of the sales reports (R2 to R6, R10, R27; PRD FR-7.77, FR-12.45, plan 71 L9): when
// the assistant narrows a report to one lot, the report says so. The screen's filter line draws the
// lot name the backend echoed in `ReportResolvedFilters.lot`, in the app language, and the shared
// print header draws the backend's `report-filter-lot` fact with the lot name it wrote for the print
// language. Nothing is built from parts here: the name is the lot's own, with its counter.

import { render } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { ReportResolvedFilters } from '../../src/bindings/ReportResolvedFilters';
import { ReportRange } from '../../src/features/reports/components/ReportRange';
import { ReportPrintDocument } from '../../src/features/reports/print/ReportPrintDocument';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { printFrameFor } from '../helpers/print-frame';
import { message, renderIn, visible } from '../helpers/render';
import { CATALOG } from '../helpers/reports';
import { PRINT_R13_AKRAM_UR, PRINT_R7_EN } from '../helpers/reports-real';

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
});

const LOT = { ur: 'احسان، آم-2', en: 'Ehsan, Mango-2' };

function filters(extra: Partial<ReportResolvedFilters> = {}): ReportResolvedFilters {
  return {
    shortcut: null,
    from: '2026-10-01',
    to: '2026-10-31',
    as_of: null,
    party: null,
    seller: null,
    zamindar: null,
    item: null,
    city: null,
    adda: null,
    partner: null,
    detail: false,
    ...extra,
  };
}

/** The filters the catalogue lists for the sales register R2, the first report the lot box narrows. */
const R2_TAKES = CATALOG.find((entry) => entry.id === 'R2')?.filters ?? [];

function showRange(language: Language, resolved: ReportResolvedFilters) {
  useLanguageStore.setState({ language });
  return renderIn(language, <ReportRange filters={resolved} takes={R2_TAKES} />);
}

describe('the filter line of a report screen', () => {
  it.each(LANGUAGES)('names the lot in %s, in the app language', (language) => {
    const { container } = showRange(language, filters({ lot: LOT }));
    const lines = [...container.querySelectorAll('p')].map((line) => visible(line.textContent));
    expect(lines).toContain(message(language, 'report-lot-shown', { name: LOT[language] }));
  });

  it('writes the lot name as the Urdu text of the lot, with the lot word first', () => {
    const { container } = showRange('ur', filters({ lot: LOT }));
    expect(visible(container.textContent)).toContain('لاٹ: احسان، آم-2');
  });

  it('writes the lot name as the English text of the lot in English', () => {
    const { container } = showRange('en', filters({ lot: LOT }));
    expect(visible(container.textContent)).toContain('Lot: Ehsan, Mango-2');
  });

  it.each(LANGUAGES)('shows no lot line in %s when no lot was picked', (language) => {
    const { container } = showRange(language, filters());
    expect(visible(container.textContent)).not.toContain(message(language, 'report-filter-lot'));
    expect(container.querySelectorAll('p')).toHaveLength(1);
  });
});

describe('the shared print header', () => {
  function printWithLot(data: ReportPrintData, value: string): HTMLElement {
    const withLot = {
      ...data,
      filters: [...data.filters, { label_id: 'report-filter-lot', value }],
    };
    const frame = {
      ...printFrameFor({
        language: withLot.language,
        title_id: withLot.title_id,
        facts: withLot.filters,
      }),
      dir: withLot.dir,
    };
    const { container } = render(
      <ReportPrintDocument data={withLot} frame={frame} applyRoot={false} />,
    );
    const root = container.querySelector<HTMLElement>('.print-doc');
    if (root === null) throw new Error('no print document');
    return root;
  }

  it('draws the lot as a header fact in an Urdu print, with its name in Urdu', () => {
    const root = printWithLot(PRINT_R13_AKRAM_UR, LOT.ur);
    const facts = visible(root.querySelector('.print-facts')?.textContent);
    expect(facts).toContain(message('ur', 'report-filter-lot'));
    expect(facts).toContain(LOT.ur);
  });

  it('draws the lot as a header fact in an English print, with its name in English', () => {
    const root = printWithLot(PRINT_R7_EN, LOT.en);
    const facts = visible(root.querySelector('.print-facts')?.textContent);
    expect(facts).toContain(message('en', 'report-filter-lot'));
    expect(facts).toContain(LOT.en);
  });

  it('keeps the other filters of the print beside the lot', () => {
    const root = printWithLot(PRINT_R7_EN, LOT.en);
    const facts = root.querySelector('.print-facts');
    expect(facts?.querySelectorAll('dt').length).toBe(PRINT_R7_EN.filters.length + 1);
  });

  it('has the label in both languages, so the header never shows a bare message id', () => {
    expect(message('ur', 'report-filter-lot')).toBe('لاٹ');
    expect(message('en', 'report-filter-lot')).toBe('Lot');
  });
});
