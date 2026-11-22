// The prints and the statement of plan 78 (A1 to A7), drawn from the backend's OWN answers of the worked day
// (Events 1 to 5, 10-10-26), in Urdu and in English: the whole sale print with the 15 columns of its sellers table,
// the buyer parchi with one slip for each buyer, the seller bills, ادھاری تفصیل with its totals, and a seller's
// statement with the opening balance as its first row. The names are in the print language, the figures are the
// backend's: this file checks that the screen draws all of them and adds up nothing (CLAUDE.md rule 1).

import { render, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DaySlipsPrintData } from '../../src/bindings/DaySlipsPrintData';
import * as appApi from '../../src/app/api';
import * as ledgersApi from '../../src/features/party_ledgers/api';
import { DaySlipsDocument } from '../../src/features/reports/print/DaySlipsDocument';
import { ReportPrintDocument } from '../../src/features/reports/print/ReportPrintDocument';
import { printTableOf } from '../../src/features/reports/print/table-print';
import { SalePrintDocument } from '../../src/features/reports/print/SalePrintDocument';
import { LANGUAGES } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { adhariPrintData, daySlipsData, salePrintData } from '../helpers/day-slips';
import { fakePartyLedgers } from '../helpers/party-ledgers';
import { entryRow, footerRow, leadRow, treegrid } from '../helpers/party-statement-rows';
import * as real from '../helpers/plan78-real';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/party_ledgers/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

const SLOW_MS = { timeout: 8_000 };

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  document.documentElement.style.removeProperty(FOOTER_PROPERTY);
  vi.unstubAllGlobals();
  leaveAppWindow();
});

const cellTexts = (row: Element | null | undefined) =>
  [...(row?.querySelectorAll('th, td') ?? [])].map((cell) => visible(cell.textContent));

describe.each(LANGUAGES)('the whole sale print of the worked day (%s)', (language) => {
  const wrapper = () => {
    useLanguageStore.setState({ language });
    const data = salePrintData(language);
    return render(<SalePrintDocument data={data} />).container;
  };
  const m = (id: string) => message(language, id);
  const ur = language === 'ur';

  it('A4: the sellers table has all 15 columns, a row for each lot, and a totals row', () => {
    const page = wrapper();
    const [sellers] = [...page.querySelectorAll('table')];
    const headings = [...(sellers?.querySelectorAll('thead th') ?? [])].map((th) =>
      visible(th.textContent),
    );
    expect(headings).toEqual(
      [
        'report-col-date',
        'report-col-doc',
        'report-col-seller',
        'report-col-item',
        'report-col-qty',
        'report-col-gross',
        'report-col-freight',
        'report-col-commission',
        'report-col-labour',
        'report-col-munshiana',
        'report-col-cash-paid',
        'report-col-store-rent',
        'report-col-phone-charge',
        'report-col-deductions',
        'report-col-net',
      ].map(m),
    );
    const rows = [...(sellers?.querySelectorAll('tbody tr') ?? [])].map(cellTexts);
    expect(rows).toHaveLength(2);
    // رحیم, کیلا-1: 50 نگ, 29,400, 3,000, 1,764, 250, 50, 5,000, 0, 20, 10,084 and 19,316.
    expect(rows[0]?.slice(4)).toEqual([
      '50',
      '29,400',
      '3,000',
      '1,764',
      '250',
      '50',
      '5,000',
      '0',
      '20',
      '10,084',
      '19,316',
    ]);
    expect(rows[0]?.[3]).toBe(ur ? 'کیلا-1' : 'Banana-1');
    expect(rows[1]?.slice(4)).toEqual([
      '10',
      '12,000',
      '1,000',
      '720',
      '50',
      '50',
      '0',
      '0',
      '20',
      '1,840',
      '10,160',
    ]);
    // The totals row is the backend's: 60 نگ, 41,400 and 29,476.
    const totals = cellTexts(sellers?.querySelector('tfoot tr'));
    expect(totals).toHaveLength(15);
    expect(totals.filter((cell) => cell !== '')).toEqual(
      expect.arrayContaining([m('report-total'), '60', '41,400', '29,476']),
    );
  });

  it('the names are in the print language: Urdu names on an Urdu print, English names on an English one', () => {
    const text = visible(wrapper().textContent);
    for (const [urdu, english] of [
      ['رحیم', 'Rahim'],
      ['کریم', 'Karim'],
      ['اکرم ٹول ٹیکس', 'Akram Toll Tax'],
      ['ارشد ملتانی', 'Arshad Multani'],
      ['شاہد', 'Shahid'],
    ] as const) {
      expect(text).toContain(ur ? urdu : english);
      expect(text).not.toContain(ur ? english : urdu);
    }
  });

  it('the buyers table totals read 44,364 / 12,800 / 31,564', () => {
    const page = wrapper();
    const buyers = [...page.querySelectorAll('table')][1];
    expect(cellTexts(buyers?.querySelector('tfoot tr')).filter((cell) => cell !== '')).toEqual(
      expect.arrayContaining(['44,364', '12,800', '31,564']),
    );
    expect(buyers?.querySelectorAll('tbody tr')).toHaveLength(3);
  });
});

describe.each(LANGUAGES)(
  'the buyer parchi and the seller bills of the worked day (%s)',
  (language) => {
    const ur = language === 'ur';
    const draw = (data: DaySlipsPrintData, variant: 'parchi' | 'seller-bills') => {
      useLanguageStore.setState({ language });
      return render(<DaySlipsDocument data={data} variant={variant} />).container;
    };

    it('A1: one slip for each buyer of the day, not the same buyer on every slip', () => {
      const page = draw(daySlipsData(language, 'parchi'), 'parchi');
      const titles = [...page.querySelectorAll('.print-slip-title')].map((title) =>
        visible(title.textContent),
      );
      expect(titles).toHaveLength(3);
      for (const name of ur
        ? ['اکرم ٹول ٹیکس', 'ارشد ملتانی', 'شاہد']
        : ['Akram Toll Tax', 'Arshad Multani', 'Shahid']) {
        expect(titles.filter((title) => title.includes(name))).toHaveLength(1);
      }
      expect(page.querySelectorAll('[data-slip]')).toHaveLength(3);
    });

    it('A1: each slip has its own figures: the three bills of the day are 18,684, 12,880 and 12,800', () => {
      const page = draw(daySlipsData(language, 'parchi'), 'parchi');
      const slips = [...page.querySelectorAll('[data-slip]')].map((slip) =>
        visible(slip.textContent),
      );
      for (const bill of ['18,684', '12,880', '12,800']) {
        expect(slips.filter((slip) => slip.includes(bill))).toHaveLength(1);
      }
    });

    it('A2: a seller’s bill is one sheet for each seller, and the lot names are in the print language', () => {
      const page = draw(daySlipsData(language, 'seller-bills'), 'seller-bills');
      const sheets = [...page.querySelectorAll('.print-sheet')];
      expect(sheets).toHaveLength(2);
      const text = visible(page.textContent);
      expect(text).toContain(ur ? 'رحیم' : 'Rahim');
      expect(text).toContain(ur ? 'کریم' : 'Karim');
      expect(text).toContain(ur ? 'کیلا-1' : 'Banana-1');
      expect(text).not.toContain(ur ? 'Rahim' : 'رحیم');
    });
  },
);

describe.each(LANGUAGES)('ادھاری تفصیل of the worked day (%s)', (language) => {
  it('A5: a row for each buyer, without the چکتا buyer and without a cash column; the totals read 31,564 / 31,564 / 0 / 31,564', () => {
    useLanguageStore.setState({ language });
    const data = adhariPrintData(language);
    const page = render(
      <ReportPrintDocument data={printTableOf(data, 'R5')} frame={data.frame} />,
    ).container;
    expect(page.querySelectorAll('tbody tr')).toHaveLength(2);
    // The columns of ادھاری تفصیل (PRD R5 v1.57, plan 95 item 17): no cash column, no cash total.
    const headings = [...page.querySelectorAll('thead th')].map((th) => visible(th.textContent));
    expect(headings).toEqual(
      [
        'print-adhari-sheet-col-party',
        'print-adhari-sheet-col-detail',
        'print-adhari-sheet-col-bill',
        'print-adhari-sheet-col-today-credit',
        'print-adhari-sheet-col-previous',
        'print-adhari-sheet-col-total',
      ].map((id) => message(language, id)),
    );
    const totals = [...page.querySelectorAll('tfoot tr')].map(cellTexts);
    const all = totals.flat().filter((cell) => cell !== '');
    expect(all).toEqual(expect.arrayContaining(['31,564', '0']));
    expect(all).not.toContain('44,364');
    expect(visible(page.textContent)).not.toContain(language === 'ur' ? 'شاہد' : 'Shahid');
    // The last row of the totals is what every buyer owes: 31,564.
    expect(totals.at(-1)?.some((cell) => cell.startsWith('31,564'))).toBe(true);
    const text = visible(page.textContent);
    expect(text).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(text).not.toContain(language === 'ur' ? 'Akram Toll Tax' : 'اکرم ٹول ٹیکس');
    // The line of what he bought is one piece: "(item qty . rate)" never wraps.
    expect(page.querySelectorAll('p.print-nowrap').length).toBeGreaterThanOrEqual(2);
  });
});

describe.each(LANGUAGES)('a seller’s statement of the worked day (%s)', (language) => {
  const m = (id: string) => message(language, id);
  const cr = language === 'ur' ? 'جمع' : 'Credit';

  beforeEach(() => {
    vi.resetAllMocks();
    healthyBackend();
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakePartyLedgers({ statements: { 35: real.STATEMENT_ROWS_WHOLE } });
  });

  it('A7: it opens with the opening balance as its first row, then the بکری, with its side', async () => {
    await renderApp('/bikri/party-ledgers?side=seller&party=35');
    const opening = await waitFor(() => {
      const found = entryRow(real.STATEMENT_ROWS_WHOLE.rows[0]?.entry_id ?? 0);
      expect(found).not.toBeNull();
      return found;
    }, SLOW_MS);
    // The first row is the account's own opening: ابتدائی بقایا, 20,000 on the credit side, and the balance after it.
    const cells = [...(opening?.querySelectorAll('[role="gridcell"]') ?? [])].map((cell) =>
      visible(cell.textContent),
    );
    expect(cells[0]).toBe('10-10-2026');
    expect(cells[1]).toBe(m('report-doc-opening'));
    expect(cells).toContain('20,000');
    expect(cells.at(-1)).toBe(`20,000${cr}`);
    // Nothing is brought forward: the opening is the first row, not a line above it.
    expect(leadRow()).toBeNull();
    // Today is 10-10-2026, so the window holds the opening and his بکری 1 (the payment is on 11-10).
    const rows = [...(treegrid()?.querySelectorAll('[role="row"][aria-expanded]') ?? [])];
    expect(rows).toHaveLength(2);
    expect(visible(rows[0]?.textContent)).toContain(m('report-doc-opening'));
    expect(
      [...(rows[1]?.querySelectorAll('[role="gridcell"]') ?? [])]
        .map((cell) => visible(cell.textContent))
        .at(-1),
    ).toBe(`180,080${cr}`);
  });

  it('A7: a later window starts with the balance brought forward, with its side, then the payment and the closing', async () => {
    // Today is 09-11-2026, so the first window starts on 11-10-2026: the opening and the بکری are before it.
    vi.mocked(appApi.dayStatus).mockResolvedValue({
      today: '2026-11-09',
      closed_up_to: null,
      today_open: true,
      first_entry_date: '2026-09-01',
    });
    fakePartyLedgers({ statements: { 35: real.STATEMENT_ROWS_FROM_11_10 } });
    await renderApp('/bikri/party-ledgers?side=seller&party=35');
    await waitFor(() => {
      expect(leadRow()).not.toBeNull();
    }, SLOW_MS);
    expect(vi.mocked(ledgersApi.partyLedgersStatementRows).mock.calls.at(-1)?.[0]).toMatchObject({
      account_id: 35,
      from: '11-10-2026',
      to: '09-11-2026',
    });
    const lead = [...(leadRow()?.querySelectorAll('[role="gridcell"]') ?? [])].map((cell) =>
      visible(cell.textContent),
    );
    expect(lead).toContain(m('party-ledgers-brought-forward'));
    expect(lead.at(-1)).toBe(`180,080${cr}`);
    const rows = [...(treegrid()?.querySelectorAll('[role="row"][aria-expanded]') ?? [])];
    expect(rows).toHaveLength(1);
    expect(visible(rows[0]?.textContent)).toContain(m('report-doc-payment'));
    // The window's totals and its closing balance are the backend's: 100,000 paid, 80,080 left.
    const totals = [...(footerRow()?.querySelectorAll('[role="gridcell"]') ?? [])].map((cell) =>
      visible(cell.textContent),
    );
    expect(totals).toContain('100,000');
    expect(totals.at(-1)).toBe(`80,080${cr}`);
  });
});
