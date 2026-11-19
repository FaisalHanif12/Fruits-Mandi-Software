// ادھاری تفصیل, R5 in the reports menu (plan 80 Q-R5-2, PRD R5 v1.43, FR-19.11, FR-12.12) in the Urdu and the
// English app, against a fake backend: `/reports/R5` lands on the v1.43 buyers-only sheet (`reports_adhari_sheet`),
// not on the v1.41 اگراہی sheet. It lists the buyers who bought on the day with the goods in short
// ("(Banana 20 . 600)"), today's bill, the cash paid on the spot, today's credit, the balance before and the
// total credit now (E2E numbers of 10-10-26), shows the backend's totals under the columns, then the buyers who
// bought nothing today and the total credit of all buyers, lists a person in the dead amount and a buyer in جمع
// apart, keeps the day in the address, opens a buyer's ledger (not a چکتا buyer's), shows the print preview in the
// shared print layout, and has its four states. It has no search box and no parchi button. The screen adds
// nothing up.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as dialog from '../../src/shared/api/dialog';
import * as events from '../../src/shared/api/events';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import {
  ADHARI_AKRAM,
  ADHARI_SHEET,
  adhariBuyer,
  dr,
  fakeCollectionSheet,
  rupees,
} from '../helpers/collection-sheet';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/api/dialog');
vi.mock('../../src/shared/api/events');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeCollectionSheet();
  // The Excel button listens for the backend's progress and asks where to save (shared/export).
  vi.mocked(events.listenEvent).mockImplementation(() => Promise.resolve(() => undefined));
  vi.mocked(events.emitEvent).mockResolvedValue(undefined);
  vi.mocked(dialog.chooseSavePath).mockResolvedValue('C:\\Reports\\adhari');
});

afterEach(() => {
  leaveAppWindow();
});

const credit = (value: number) => ({ amount_paisa: rupees(value), side: 'cr' }) as const;
const sums = (bill: number, cash: number, previous: number) => ({
  bill_paisa: rupees(bill),
  cash_paisa: rupees(cash),
  today_credit_paisa: rupees(bill - cash),
  previous_due: dr(previous),
  total_due: dr(previous + bill - cash),
});

describe.each(LANGUAGES)('ادھاری تفصیل, R5 (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const open = (search = '') => renderApp(`/reports/adhari-sheet${search}`);
  const rowOf = (id: string) => document.querySelector<HTMLElement>(`.ag-row[row-id="${id}"]`);
  const cell = (row: HTMLElement | null, column: string) =>
    visible(row?.querySelector(`[col-id="${column}"]`)?.textContent);
  const heading = () =>
    screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('report-r5-title'),
    });
  const sheetCalls = () => vi.mocked(reportsApi.reportsAdhariSheet).mock.calls;

  it('R5 in the reports menu lands on ادھاری تفصیل, not on the v1.41 اگراہی sheet, and opens on today', async () => {
    const { router } = await renderApp('/reports/R5');
    await heading();
    expect(router.state.location.pathname).toBe('/reports/adhari-sheet');
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    // Nothing asked is today: the backend is asked with an empty day, and only the new command is used.
    expect(sheetCalls()[0]?.[0]).toMatchObject({ date: '', offset: 0 });
    expect(vi.mocked(reportsApi.reportsCollectionSheet)).not.toHaveBeenCalled();
    expect(screen.getByText((text) => visible(text) === m('report-day-now'))).toBeInTheDocument();
  });

  it('each buyer has his name, goods in short, bill, cash, credit today, previous balance and total credit', async () => {
    await open();
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    const akram = rowOf('20');
    expect(cell(akram, 'party')).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(cell(akram, 'detail')).toBe(
      m('report-detail-sale-short', {
        item: language === 'ur' ? 'کیلا' : 'Banana',
        qty: '20',
        rate: '600',
      }),
    );
    expect(cell(akram, 'bill')).toBe('12,880');
    expect(cell(akram, 'cash')).toBe('0');
    expect(cell(akram, 'today_credit')).toBe('12,880');
    expect(cell(akram, 'previous')).toContain('0');
    expect(cell(akram, 'total_due')).toContain('12,880');
    expect(cell(akram, 'total_due')).toContain(m('report-side-dr'));
    expect(cell(rowOf('21'), 'bill')).toBe('18,684');
    // شاہد bought and paid in full: he is on the sheet with nothing left.
    const shahid = rowOf('p-1');
    expect(cell(shahid, 'bill')).toBe('12,800');
    expect(cell(shahid, 'cash')).toBe('12,800');
    expect(cell(shahid, 'today_credit')).toBe('0');
    expect(cell(shahid, 'total_due')).not.toContain(m('report-side-dr'));
  });

  it('shows the backend’s totals under the columns, the print’s own column words, and no search box or parchi button', async () => {
    await open();
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    const totals = document.querySelector<HTMLElement>('.ag-row-pinned');
    expect(cell(totals, 'bill')).toBe('44,364');
    expect(cell(totals, 'cash')).toBe('12,800');
    expect(cell(totals, 'today_credit')).toBe('31,564');
    expect(cell(totals, 'total_due')).toContain('31,564');
    const heads = screen
      .getAllByRole('columnheader')
      .map((head) => visible(head.textContent).replace(/\s+/g, ' ').trim());
    expect(heads).toEqual(
      [
        'print-adhari-sheet-col-party',
        'print-adhari-sheet-col-detail',
        'print-adhari-sheet-col-bill',
        'print-adhari-sheet-col-cash',
        'print-adhari-sheet-col-today-credit',
        'print-adhari-sheet-col-previous',
        'print-adhari-sheet-col-total',
      ].map((id) => m(id)),
    );
    // The v1.41 sheet's search box and parchi button are not here: the backend has no search on this sheet and
    // the parchi is a print of the sale book (FR-12.12, FR-19.15).
    expect(screen.queryByRole('searchbox')).toBeNull();
    expect(
      screen.queryByRole('button', { name: (name) => visible(name).includes('parchi') }),
    ).toBeNull();
  });

  it('under the table: the buyers who bought nothing today as one total, and the total credit of all buyers', async () => {
    vi.mocked(reportsApi.reportsAdhariSheet).mockResolvedValue({
      ...ADHARI_SHEET,
      others: sums(0, 0, 5_000),
      others_count: 2,
      all_totals: sums(44_364, 12_800, 5_000),
    });
    await open();
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    const others = await screen.findByText(
      (text) => visible(text).includes(m('print-adhari-sheet-others')),
      { selector: 'p' },
    );
    expect(visible(others.textContent)).toContain('5,000');
    expect(visible(others.textContent)).toContain('(2)');
    const all = await screen.findByText(
      (text) => visible(text).includes(m('print-adhari-sheet-all')),
      { selector: 'p' },
    );
    // The total credit of all buyers is the backend's `all_totals`: 5,000 + 44,364 − 12,800.
    expect(visible(all.textContent)).toContain('36,564');
  });

  it('a person in the dead amount is marked and his total is listed apart', async () => {
    const dead = adhariBuyer(22, '0022', 'مرحوم', 'Late Buyer', 0, 0, 18_684, { dead: true });
    vi.mocked(reportsApi.reportsAdhariSheet).mockResolvedValue({
      ...ADHARI_SHEET,
      rows: [ADHARI_AKRAM, dead],
      total_count: 2,
      totals: sums(12_880, 0, 0),
      all_totals: sums(12_880, 0, 0),
      dead_totals: sums(0, 0, 18_684),
    });
    await open();
    await waitFor(() => {
      expect(rowOf('22')).not.toBeNull();
    });
    expect(cell(rowOf('22'), 'party')).toContain(m('dead-tag'));
    const apart = language === 'ur' ? 'الگ شمار' : 'listed apart';
    const line = await screen.findByText((text) => visible(text).includes(apart), {
      selector: 'p',
    });
    expect(visible(line.textContent)).toContain('18,684');
    expect(visible(line.textContent)).toContain(m('print-adhari-sheet-col-total'));
    // The main total stays what the live buyers owe.
    expect(cell(document.querySelector<HTMLElement>('.ag-row-pinned'), 'total_due')).toContain(
      '12,880',
    );
  });

  it('a buyer whose total credit ends in جمع is in the rows with his side word, and his total is listed apart', async () => {
    const inCredit = {
      ...adhariBuyer(23, '0023', 'جمع والا', 'Credit Buyer', 3_000, 0, 0),
      previous_due: credit(5_000),
      total_due: credit(2_000),
    };
    vi.mocked(reportsApi.reportsAdhariSheet).mockResolvedValue({
      ...ADHARI_SHEET,
      rows: [ADHARI_AKRAM, inCredit],
      total_count: 2,
      totals: sums(12_880, 0, 0),
      all_totals: sums(12_880, 0, 0),
      credit_totals: {
        ...sums(3_000, 0, 0),
        previous_due: credit(5_000),
        total_due: credit(2_000),
      },
    });
    await open();
    await waitFor(() => {
      expect(rowOf('23')).not.toBeNull();
    });
    expect(cell(rowOf('23'), 'total_due')).toContain('2,000');
    expect(cell(rowOf('23'), 'total_due')).toContain(m('common-side-cr'));
    const apart = language === 'ur' ? 'الگ شمار' : 'listed apart';
    const line = await screen.findByText((text) => visible(text).includes(apart), {
      selector: 'p',
    });
    const said = visible(line.textContent);
    expect(said).toContain('2,000');
    expect(said).toContain('5,000');
    expect(said).toContain('3,000');
    expect(said).toContain(m('common-side-cr'));
    expect(cell(document.querySelector<HTMLElement>('.ag-row-pinned'), 'total_due')).toContain(
      '12,880',
    );
  });

  it('keeps the day in the address: a day asked for goes to the backend, and "today" comes back', async () => {
    const user = userEvent.setup();
    const { router } = await open('?date=2026-10-09');
    await waitFor(() => {
      expect(sheetCalls()[0]?.[0]).toMatchObject({ date: '2026-10-09' });
    });
    expect(
      await screen.findByText((text) => visible(text).includes('09-10-2026')),
    ).toBeInTheDocument();
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name) === m('adhari-today'),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.search).not.toHaveProperty('date');
    });
    await waitFor(() => {
      expect(sheetCalls().at(-1)?.[0]).toMatchObject({ date: '' });
    });
  });

  it('the go-to-day box takes a typed day, 9, and moves the sheet to it', async () => {
    const user = userEvent.setup();
    const { router } = await open();
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    const box = await screen.findByLabelText<HTMLInputElement>((name) =>
      visible(name).includes(m('adhari-goto')),
    );
    await user.type(box, '9{Enter}');
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ date: '2026-10-09' });
    });
  });

  it('Enter on a buyer opens his ledger, and Enter on a چکتا buyer opens nothing (he has no ledger)', async () => {
    const user = userEvent.setup();
    const { router } = await open();
    await waitFor(() => {
      expect(rowOf('p-1')).not.toBeNull();
    });
    const shahid = rowOf('p-1')?.querySelector<HTMLElement>('[col-id="party"]');
    if (shahid === null || shahid === undefined) throw new Error('no cell');
    await user.click(shahid);
    await user.keyboard('{Enter}');
    expect(router.state.location.pathname).toBe('/reports/adhari-sheet');
    const target = rowOf('20')?.querySelector<HTMLElement>('[col-id="party"]');
    if (target === null || target === undefined) throw new Error('no cell');
    await user.click(target);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R13');
    });
    expect(router.state.location.search).toMatchObject({ party: 20 });
  });

  it('the print button shows the preview of the same day in the shared print layout, and Back returns', async () => {
    const user = userEvent.setup();
    await open('?date=2026-10-09');
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).includes(m('report-preview')),
      }),
    );
    // It is the print of ادھاری تفصیل, the whole day: the same command the sale book's second print button uses.
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsAdhariSheetPrintData).mock.calls[0]?.[0]).toEqual({
        date: '2026-10-09',
        language,
        paper: 'a4',
      });
    });
    expect(vi.mocked(reportsApi.reportsCollectionSheetPrintData)).not.toHaveBeenCalled();
    // The backend's own print of the worked day (`day-slips.ts`): one table, a row for each buyer, then the totals.
    await waitFor(() => {
      expect(visible(document.querySelector('.print-doc')?.textContent)).toContain('44,364');
    });
    expect(document.querySelectorAll('.print-doc table')).toHaveLength(1);
    expect(visible(document.querySelector('.print-doc')?.textContent)).toContain(
      language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
    );
    await user.click(
      await within(screen.getByRole('main')).findByRole('button', {
        name: (name) => visible(name) === m('common-back'),
      }),
    );
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
  });

  it('has an Excel button: it asks where to save, then writes every buyer of the day in the app language (plan 80 Q-R5-2)', async () => {
    vi.mocked(reportsApi.tablesExportExcel).mockResolvedValue({
      path: 'C:\\Reports\\adhari.xlsx',
      rows: 3,
      size_bytes: 9000,
    });
    const user = userEvent.setup();
    await open('?date=2026-10-09');
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).includes(m('export-excel')),
      }),
    );
    await waitFor(() => {
      expect(reportsApi.tablesExportExcel).toHaveBeenCalledTimes(1);
    });
    // The sheet is the print's own table of the same day: the `adhari_sheet` kind, with no paper.
    expect(vi.mocked(reportsApi.tablesExportExcel).mock.calls[0]?.[0].source).toEqual({
      kind: 'adhari_sheet',
      date: '2026-10-09',
      language,
      paper: null,
    });
  });

  it('says that nothing is there for the day, with the way to another day', async () => {
    vi.mocked(reportsApi.reportsAdhariSheet).mockResolvedValue({
      ...ADHARI_SHEET,
      rows: [],
      total_count: 0,
    });
    await open();
    expect(
      await screen.findByText((text) => visible(text) === m('adhari-empty')),
    ).toBeInTheDocument();
    expect(
      screen.getByText((text) => visible(text) === m('adhari-empty-help')),
    ).toBeInTheDocument();
  });

  it('an error is told with a retry that asks again', async () => {
    vi.mocked(reportsApi.reportsAdhariSheet).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL', field: null, values: {} }])),
    );
    const user = userEvent.setup();
    await open();
    const retry = await screen.findByRole('button', {
      name: (name) => visible(name) === m('common-retry'),
    });
    await user.click(retry);
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    expect(within(document.body).queryByRole('button', { name: m('common-retry') })).toBeNull();
  });
});
