// The اگراہی sheet, R5 (plan 76 point 2, PRD R5, FR-19.11, FR-19.15) in the Urdu and the English app, against a
// fake backend: `/reports/R5` lands on it, it lists every buyer of the day with his code, name, goods in
// short ("(Banana 20 . 600)"), previous due, the day's bill, what he paid on the spot and what he owes now
// (E2E numbers of 10-10-26), shows the backend's totals under the columns, marks a person in the dead
// amount and lists him apart, keeps the day in the address, searches through the backend, opens a buyer's
// ledger, shows the print preview in the shared print layout, and has its four states. The screen adds
// nothing up.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { AKRAM, SHEET, buyer, fakeCollectionSheet, rupees, dr } from '../helpers/collection-sheet';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeCollectionSheet();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the collection sheet, R5 (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const open = (search = '') => renderApp(`/reports/collection-sheet${search}`);
  const rowOf = (id: string) => document.querySelector<HTMLElement>(`.ag-row[row-id="${id}"]`);
  const cell = (row: HTMLElement | null, column: string) =>
    visible(row?.querySelector(`[col-id="${column}"]`)?.textContent);
  const heading = () =>
    screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('collection-title'),
    });

  it('R5 lands on the sheet, and the sheet opens on today with every buyer of the day', async () => {
    const { router } = await renderApp('/reports/R5');
    await heading();
    expect(router.state.location.pathname).toBe('/reports/collection-sheet');
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    // Nothing asked is today: the backend is asked with an empty day.
    expect(vi.mocked(reportsApi.reportsCollectionSheet).mock.calls[0]?.[0]).toMatchObject({
      date: '',
      search: null,
      offset: 0,
    });
    expect(screen.getByText((text) => visible(text) === m('report-day-now'))).toBeInTheDocument();
  });

  it('each buyer has his code, name, goods in short, previous due, bill, paid on the spot and what he owes now', async () => {
    await open();
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    const akram = rowOf('20');
    expect(cell(akram, 'code')).toBe('0020');
    expect(cell(akram, 'party')).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(cell(akram, 'detail')).toBe(
      m('report-detail-sale-short', {
        item: language === 'ur' ? 'کیلا' : 'Banana',
        qty: '20',
        rate: '600',
      }),
    );
    expect(cell(akram, 'previous')).toContain('0');
    expect(cell(akram, 'bill')).toBe('12,880');
    expect(cell(akram, 'paid')).toBe('0');
    expect(cell(akram, 'total_due')).toContain('12,880');
    expect(cell(akram, 'total_due')).toContain(m('report-side-dr'));
    expect(cell(rowOf('21'), 'bill')).toBe('18,684');
    // شاہد bought and paid in full: he is on the sheet with nothing left (he has no balance).
    const shahid = rowOf('p-1');
    expect(cell(shahid, 'bill')).toBe('12,800');
    expect(cell(shahid, 'paid')).toBe('12,800');
    expect(cell(shahid, 'total_due')).toContain('0');
    expect(cell(shahid, 'total_due')).not.toContain(m('report-side-dr'));
  });

  it('shows the backend’s totals under the columns, and no وصولی column', async () => {
    await open();
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    const totals = document.querySelector<HTMLElement>('.ag-row-pinned');
    expect(cell(totals, 'bill')).toBe('44,364');
    expect(cell(totals, 'paid')).toBe('12,800');
    expect(cell(totals, 'total_due')).toContain('31,564');
    // The columns are the code, the buyer, his goods, the previous due, the bill, paid on the spot and what
    // he owes now: there is no وصولی column (a وصولی shows in the next day's previous due).
    const heads = screen
      .getAllByRole('columnheader')
      .map((head) => visible(head.textContent).replace(/\s+/g, ' ').trim());
    expect(heads).toEqual(
      [
        'accounts-col-code',
        'collection-col-party',
        'collection-col-detail',
        'collection-col-previous',
        'collection-col-bill',
        'collection-col-paid',
        'collection-col-total-due',
      ].map((id) => m(id)),
    );
  });

  it('a person in the dead amount is marked and his total is listed apart', async () => {
    const dead = buyer(22, '0022', 'مرحوم', 'Late Buyer', 0, 0, 18_684, { dead: true });
    vi.mocked(reportsApi.reportsCollectionSheet).mockResolvedValue({
      ...SHEET,
      rows: [AKRAM, dead],
      total_count: 2,
      totals: {
        previous_due: dr(0),
        bill_paisa: rupees(12_880),
        paid_paisa: rupees(0),
        total_due: dr(12_880),
      },
      dead_totals: {
        previous_due: dr(18_684),
        bill_paisa: rupees(0),
        paid_paisa: rupees(0),
        total_due: dr(18_684),
      },
    });
    await open();
    await waitFor(() => {
      expect(rowOf('22')).not.toBeNull();
    });
    expect(cell(rowOf('22'), 'party')).toContain(m('dead-tag'));
    // His total is under the table, apart from the totals row: previous due, bill and total due.
    const apart = language === 'ur' ? 'الگ شمار' : 'listed apart';
    const line = await screen.findByText((text) => visible(text).includes(apart), {
      selector: 'p',
    });
    expect(visible(line.textContent)).toContain('18,684');
    // All three figures of his line are written, the last one too (the markup of the message is closed).
    expect(visible(line.textContent)).toContain(m('collection-col-total-due'));
    expect(cell(document.querySelector<HTMLElement>('.ag-row-pinned'), 'total_due')).toContain(
      '12,880',
    );
  });

  it('a buyer whose total due ends in جمع is in the rows with his side word, and his total is listed apart (not in the main total)', async () => {
    const credit = (value: number) => ({ amount_paisa: rupees(value), side: 'cr' }) as const;
    const inCredit = {
      ...buyer(23, '0023', 'جمع والا', 'Credit Buyer', 3_000, 0, 0),
      previous_due: credit(5_000),
      total_due: credit(2_000),
    };
    vi.mocked(reportsApi.reportsCollectionSheet).mockResolvedValue({
      ...SHEET,
      rows: [AKRAM, inCredit],
      total_count: 2,
      totals: {
        previous_due: dr(0),
        bill_paisa: rupees(12_880),
        paid_paisa: rupees(0),
        total_due: dr(12_880),
      },
      credit_totals: {
        previous_due: credit(5_000),
        bill_paisa: rupees(3_000),
        paid_paisa: rupees(0),
        total_due: credit(2_000),
      },
    });
    await open();
    await waitFor(() => {
      expect(rowOf('23')).not.toBeNull();
    });
    // His own row shows the amount with the side word.
    expect(cell(rowOf('23'), 'total_due')).toContain('2,000');
    expect(cell(rowOf('23'), 'total_due')).toContain(m('common-side-cr'));
    // His total is under the table, apart from the totals row, with the side word on the balances.
    const apart = language === 'ur' ? 'الگ شمار' : 'listed apart';
    const line = await screen.findByText((text) => visible(text).includes(apart), {
      selector: 'p',
    });
    const said = visible(line.textContent);
    expect(said).toContain('2,000');
    expect(said).toContain('5,000');
    expect(said).toContain('3,000');
    expect(said).toContain(m('collection-col-total-due'));
    expect(said).toContain(m('common-side-cr'));
    // The main total stays what the buyers owe.
    expect(cell(document.querySelector<HTMLElement>('.ag-row-pinned'), 'total_due')).toContain(
      '12,880',
    );
  });

  it('keeps the day in the address: a day asked for goes to the backend, and "today" comes back', async () => {
    const user = userEvent.setup();
    const { router } = await open('?date=2026-10-09');
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsCollectionSheet).mock.calls[0]?.[0]).toMatchObject({
        date: '2026-10-09',
      });
    });
    expect(
      await screen.findByText((text) => visible(text).includes('09-10-2026')),
    ).toBeInTheDocument();
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name) === m('collection-today'),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.search).not.toHaveProperty('date');
    });
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsCollectionSheet).mock.calls.at(-1)?.[0]).toMatchObject({
        date: '',
      });
    });
  });

  it('the go-to-day box takes a typed day, 9, and moves the sheet to it', async () => {
    const user = userEvent.setup();
    const { router } = await open();
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    const box = await screen.findByLabelText<HTMLInputElement>((name) =>
      visible(name).includes(m('collection-goto')),
    );
    await user.type(box, '9{Enter}');
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ date: '2026-10-09' });
    });
  });

  it('the search goes to the backend with its words, and a clear brings the list back', async () => {
    const user = userEvent.setup();
    await open();
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    await user.type(screen.getByPlaceholderText(m('collection-search')), 'arshad');
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsCollectionSheet).mock.calls.at(-1)?.[0]).toMatchObject({
        search: 'arshad',
        offset: 0,
      });
    });
    await waitFor(() => {
      expect(rowOf('20')).toBeNull();
    });
    expect(rowOf('21')).not.toBeNull();
  });

  it('Enter on a buyer opens his ledger', async () => {
    const user = userEvent.setup();
    const { router } = await open();
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    const target = rowOf('20')?.querySelector<HTMLElement>('[col-id="party"]');
    if (target === null || target === undefined) throw new Error('no cell');
    await user.click(target);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R13');
    });
    expect(router.state.location.search).toMatchObject({ party: 20 });
  });

  it('the print button shows the preview of the same day and search in the shared print layout, and Back returns', async () => {
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
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsCollectionSheetPrintData).mock.calls[0]?.[0]).toEqual({
        date: '2026-10-09',
        search: null,
        sort: null,
        language,
        paper: 'a4',
      });
    });
    // The page is the backend's table in the print layout: its rows and its totals.
    await waitFor(() => {
      expect(visible(document.querySelector('.print-sheet')?.textContent)).toContain(
        '0020 Akram Toll Tax',
      );
    });
    expect(visible(document.querySelector('.print-sheet')?.textContent)).toContain('44,364');
    expect(document.querySelector('.print-sheet')).not.toBeNull();
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name) === m('common-back'),
      }),
    );
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
  });

  it('the parchi button shows one short slip for each buyer of the day, with the line for the وصولی, and Back returns', async () => {
    const user = userEvent.setup();
    await open('?date=2026-10-09');
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).includes(m('collection-parchi')),
      }),
    );
    await waitFor(() => {
      expect(vi.mocked(reportsApi.reportsBuyerParchiPrintData).mock.calls[0]?.[0]).toEqual({
        date: '2026-10-09',
        bikri_id: null,
        bikri_ids: null,
        buyer_ids: null,
        sections: 'buyers',
        language,
        paper: 'a4',
      });
    });
    await waitFor(() => {
      expect(document.querySelectorAll('.print-parchi-slip').length).toBe(1);
    });
    expect(document.querySelector('.print-parchi-collect .print-write-line')).not.toBeNull();
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name) === m('common-back'),
      }),
    );
    await waitFor(() => {
      expect(rowOf('20')).not.toBeNull();
    });
  });

  it('says that nothing is there for the day, and that nothing matches a search, with a way out', async () => {
    vi.mocked(reportsApi.reportsCollectionSheet).mockResolvedValue({
      ...SHEET,
      rows: [],
      total_count: 0,
    });
    const user = userEvent.setup();
    await open();
    expect(
      await screen.findByText((text) => visible(text) === m('collection-empty')),
    ).toBeInTheDocument();
    await user.type(screen.getByPlaceholderText(m('collection-search')), 'zzz');
    const clear = await screen.findByRole('button', {
      name: (name) => visible(name) === m('collection-clear'),
    });
    await user.click(clear);
    expect(
      await screen.findByText((text) => visible(text) === m('collection-empty')),
    ).toBeInTheDocument();
  });

  it('an error is told with a retry that asks again', async () => {
    vi.mocked(reportsApi.reportsCollectionSheet).mockRejectedValueOnce(
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
