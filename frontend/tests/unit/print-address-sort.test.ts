// The address of a report or analytics print window carries the screen's order and month rows (plan 38,
// backend contract): `&sort=<JSON>` and `&months=1`. The window reads them and gives them to the
// print-data command, so the printed list is in the order of the screen. A value that is not right is
// a failed print, never a guess (the page in another order).

import { describe, expect, it } from 'vitest';
import { parseAnalyticsPrintParams } from '../../src/features/analytics/print/params';
import { parseReportPrintParams } from '../../src/features/reports/print/params';

const FILTERS = encodeURIComponent(JSON.stringify({ from: '01-10-2026' }));
const SORT = encodeURIComponent(
  JSON.stringify({ column: 'debit', direction: 'desc', language: null }),
);

const report = (extra: string) =>
  `report=R13&language=ur&paper=a4&filters=${FILTERS}${extra}&job=print-1`;
const analytics = (extra: string) =>
  `analytics=A&language=en&paper=a4&filters=${FILTERS}${extra}&job=print-1`;

describe('the print address of a report', () => {
  it('has the report’s own order and no month rows when the address has neither', () => {
    const params = parseReportPrintParams(report(''));
    expect(params?.input.sort).toBeNull();
    expect(params?.input.group_by_month).toBe(false);
  });

  it('carries the sort of the screen and its month rows', () => {
    const params = parseReportPrintParams(report(`&sort=${SORT}&months=1`));
    expect(params?.input.sort).toEqual({ column: 'debit', direction: 'desc', language: null });
    expect(params?.input.group_by_month).toBe(true);
  });

  it('carries month rows without a sort (the date order of the screen)', () => {
    const params = parseReportPrintParams(report('&months=1'));
    expect(params?.input.sort).toBeNull();
    expect(params?.input.group_by_month).toBe(true);
  });

  it('is a failed print when the sort or the months are not what the backend writes', () => {
    expect(parseReportPrintParams(report('&sort=not-json'))).toBeNull();
    expect(
      parseReportPrintParams(
        report(`&sort=${encodeURIComponent(JSON.stringify({ column: '', direction: 'up' }))}`),
      ),
    ).toBeNull();
    expect(parseReportPrintParams(report('&months=2'))).toBeNull();
  });
});

describe('the print address of an analytics view', () => {
  it('has the view’s own order when the address has no sort', () => {
    expect(parseAnalyticsPrintParams(analytics(''))?.input.sort).toBeNull();
  });

  it('carries the sort of the screen', () => {
    const params = parseAnalyticsPrintParams(analytics(`&sort=${SORT}`));
    expect(params?.input.sort).toEqual({ column: 'debit', direction: 'desc', language: null });
  });

  it('is a failed print when the sort is not what the backend writes', () => {
    expect(parseAnalyticsPrintParams(analytics('&sort=%7B%7D'))).toBeNull();
  });
});
