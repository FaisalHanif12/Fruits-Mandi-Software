// The buyers' list ادھاری تفصیل (R5) is drawn with the full grid of lines between its columns (Umar, 21-11-2026): the
// table asks for the `print-table-grid` class and the report print style gives the class thin vertical lines in the
// same colour as the horizontal lines of the rows, in the heading and in the totals row too. The backend's own
// recorded print of the worked day is drawn in Urdu and in English on A4; a table of another report does not ask
// for the grid, so nothing else changes.

import { render } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { afterEach, describe, expect, it } from 'vitest';
import { DaySlipsDocument, ReportPrintDocument, printTableOf } from '../../src/features/reports';
import { useLanguageStore } from '../../src/shared/i18n';
import { adhariPrintData, daySlipsData } from '../helpers/day-slips';

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
});

const LANGS = ['ur', 'en'] as const;

describe.each(LANGS)('the buyers’ list R5 on A4 in %s', (language) => {
  it('draws its table with the grid class: heading, rows and totals are in the one table that has it', () => {
    const data = adhariPrintData(language, 'a4');
    const { container } = render(
      <ReportPrintDocument data={printTableOf(data, 'R5')} frame={data.frame} />,
    );
    const tables = container.querySelectorAll('table.print-table');
    expect(tables).toHaveLength(1);
    const table = tables[0];
    expect(table?.classList.contains('print-table-grid')).toBe(true);
    expect(table?.querySelectorAll('thead th').length).toBeGreaterThan(1);
    expect(table?.querySelectorAll('tfoot td').length).toBeGreaterThan(1);
  });
});

describe('the grid is asked for by R5 only', () => {
  it('the day slips of the same day do not ask for it', () => {
    const { container } = render(
      <DaySlipsDocument data={daySlipsData('en', 'parchi', 'a4')} variant="parchi" />,
    );
    expect(container.querySelectorAll('table').length).toBeGreaterThan(0);
    expect(container.querySelector('.print-table-grid')).toBeNull();
  });

  it('the report print style gives the grid class thin vertical lines in the colour of the row lines', () => {
    const css = readFileSync('src/features/reports/reports.css', 'utf8');
    const rule =
      /\.print-report \.print-table-grid th,\s*\.print-report \.print-table-grid td \{([^}]*)\}/.exec(
        css,
      );
    expect(rule).not.toBeNull();
    const body = rule?.[1] ?? '';
    // Logical sides only (rule 8), the print rule width and the same token colour as the horizontal lines of the rows.
    expect(body).toMatch(/border-inline:\s*var\(--print-rule\)\s+solid\s+var\(--color-border\)/);
    expect(body).not.toMatch(/left|right/);
  });
});
