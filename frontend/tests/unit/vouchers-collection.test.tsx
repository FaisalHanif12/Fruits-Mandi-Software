// اگراہی فہرست and the collection round (plan 15, PRD FR-15.6, FR-7.61, FR-15.1): the buyers planned
// for a day with their dues from the backend, the amounts typed by keyboard, the receipts saved
// together (each its own posting), a row the backend refuses keeping its reason at its cell while
// the others are saved, and a row left empty skipped. In the Urdu and in the English app.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts, money } from '../helpers/accounts';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters, record } from '../helpers/masters';
import { message, visible } from '../helpers/render';
import { refusedReceiptRow, savedReceiptRow, skippedReceiptRow } from '../helpers/voucher-books';
import { AKRAM, ARSHAD, collectionList, fakeVouchers } from '../helpers/vouchers';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const pageText = () => visible(document.body.textContent);
const named = (text: string) => (name: string) => visible(name) === text;
const shown = (text: string) => (content: string) => visible(content) === text;

function cell(rowId: string, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${rowId}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

/** The ids of the rows in the grid, top to bottom (random ids made by the entry store). */
function rowIds(): string[] {
  return Array.from(document.querySelectorAll('.ag-row[row-index]'))
    .sort(
      (a, b) => Number(a.getAttribute('row-index') ?? 0) - Number(b.getAttribute('row-index') ?? 0),
    )
    .map((row) => row.getAttribute('row-id') ?? '');
}

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  fakeAccounts([AKRAM, ARSHAD]);
  fakeAccountSearch([AKRAM, ARSHAD]);
  fakeMasters({
    staff: [record('staff', 81, '0001', 'اسلم', 'Aslam')],
    city: [record('city', 91, '0001', 'کوٹ رادھا کشن', 'Kot Radha Kishan')],
  });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the collection round (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeVouchers();
    vi.mocked(vouchersApi.assistCollectionList).mockResolvedValue(collectionList());
    vi.mocked(vouchersApi.vouchersSaveReceiptsBatch).mockImplementation((rows) =>
      Promise.resolve({
        rows: rows.map((row, index) =>
          row.amount.includes('.')
            ? refusedReceiptRow(index, 'ERR_DECIMALS_NOT_ALLOWED')
            : row.amount === '0'
              ? skippedReceiptRow()
              : savedReceiptRow(index),
        ),
        saved_paisa: money(7880),
      }),
    );
  });

  async function open() {
    const app = await renderApp('/bikri/collection');

    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'vouchers-collection-title')),
    });
    await waitFor(() => {
      expect(rowIds()).toHaveLength(2);
    });
    return { ...app, user: userEvent.setup() };
  }

  async function typeInto(user: ReturnType<typeof userEvent.setup>, rowId: string, text: string) {
    const target = document.querySelector<HTMLElement>(
      `.ag-row[row-id="${rowId}"] [col-id="received"]`,
    );
    if (target === null) throw new Error('no cell');
    await user.click(target);
    await user.keyboard(`${text}{Enter}`);
  }

  it('reads every page of the list before the round starts: 130 buyers come in two requests', async () => {
    const base = collectionList();
    const group = base.groups[0];
    if (group === undefined) throw new Error('no group');
    const first = group.buyers[0];
    if (first === undefined) throw new Error('no buyer');
    const everyBuyer = Array.from({ length: 130 }, (_, index) => ({
      ...first,
      account: { id: 1000 + index, code: String(index), name_ur: 'خ', name_en: 'B' },
    }));
    vi.mocked(vouchersApi.assistCollectionList).mockImplementation((input) =>
      Promise.resolve({
        ...base,
        groups: [{ ...group, buyers: everyBuyer.slice(input.offset, input.offset + input.limit) }],
        total_count: 130,
        offset: input.offset,
        limit: input.limit,
      }),
    );
    await renderApp('/bikri/collection');
    await waitFor(() => {
      expect(vouchersApi.assistCollectionList).toHaveBeenCalledTimes(2);
    });
    expect(vouchersApi.assistCollectionList).toHaveBeenNthCalledWith(1, {
      date: '10-10-2026',
      collector_staff_id: null,
      city_id: null,
      offset: 0,
      limit: 100,
    });
    expect(vouchersApi.assistCollectionList).toHaveBeenNthCalledWith(2, {
      date: '10-10-2026',
      collector_staff_id: null,
      city_id: null,
      offset: 100,
      limit: 100,
    });
    // The round is drawn once, from the joined list; the grid draws the rows in view.
    await waitFor(() => {
      expect(rowIds().length).toBeGreaterThan(0);
    });
  });

  it('asks for today with no collector or city, and lists each buyer with his dues and days from the backend', async () => {
    await open();
    expect(vouchersApi.assistCollectionList).toHaveBeenCalledWith({
      date: '10-10-2026',
      collector_staff_id: null,
      city_id: null,
      offset: 0,
      limit: 100,
    });
    const [first, second] = rowIds();
    if (first === undefined || second === undefined) throw new Error('rows');
    expect(cell(first, 'code')).toBe('0020');
    expect(cell(first, 'name')).toBe(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(cell(first, 'dues')).toBe('2,880');
    expect(cell(first, 'days')).toBe('3');
    expect(cell(second, 'dues')).toBe('18,684');
    expect(cell(first, 'group')).toBe(
      language === 'ur' ? 'اسلم، کوٹ رادھا کشن' : 'Aslam, Kot Radha Kishan',
    );
    expect(pageText()).toContain(message(language, 'vouchers-collection-count', { count: '2' }));
    expect(pageText()).toContain('21,564');
  });

  it('the amounts are typed down the column with Enter, and F10 saves them together as typed', async () => {
    const { user } = await open();
    const [first, second] = rowIds();
    if (first === undefined || second === undefined) throw new Error('rows');
    await typeInto(user, first, '2880');
    // Enter went on to the amount of the next buyer.
    await user.keyboard('5000{Enter}');
    expect(cell(first, 'received')).toContain('2,880');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSaveReceiptsBatch).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersSaveReceiptsBatch).mock.calls[0]?.[0]).toEqual([
      {
        date: '10-10-2026',
        account_id: 21,
        person_key: null,
        amount: '2880',
        collector_staff_id: 81,
        narration: '',
      },
      expect.objectContaining({ account_id: 22, amount: '5000', collector_staff_id: 81 }),
    ]);
    const toast = message(language, 'vouchers-collection-saved', { count: '2' });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
  });

  it('a row left empty is not sent, and nothing to save leaves the button off', async () => {
    const { user } = await open();
    const save = () =>
      screen.getByRole('button', {
        name: new RegExp(message(language, 'vouchers-collection-save')),
      });
    expect(save()).toBeDisabled();
    const [first] = rowIds();
    if (first === undefined) throw new Error('rows');
    await typeInto(user, first, '1000');
    await waitFor(() => {
      expect(save()).toBeEnabled();
    });
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSaveReceiptsBatch).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersSaveReceiptsBatch).mock.calls[0]?.[0]).toHaveLength(1);
  });

  it('a refused row keeps its amount and its reason at the cell while the other is saved', async () => {
    const { user } = await open();
    const [first, second] = rowIds();
    if (first === undefined || second === undefined) throw new Error('rows');
    await typeInto(user, first, '2880');
    await user.keyboard('50.5{Enter}');
    press('F10');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-decimals-not-allowed'));
    });
    // The second buyer's row is the one with the problem, and its amount is still there.
    const problems = screen.getByRole('alert');
    expect(visible(problems.textContent)).toContain(
      message(language, 'vouchers-collection-problems'),
    );
    expect(visible(problems.textContent)).toContain(
      language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani',
    );
    const byCode = (code: string) => rowIds().find((id) => cell(id, 'code') === code) ?? '';
    await waitFor(() => {
      expect(cell(byCode('0021'), 'received')).toContain('50.5');
    });
    expect(cell(byCode('0020'), 'received')).toBe('');
    // The reason stays after the round was built again, and is also at the cell.
    expect(visible(screen.getByRole('alert').textContent)).toContain(
      message(language, 'err-decimals-not-allowed'),
    );
    const toast = message(language, 'vouchers-collection-some-refused', {
      count: '1',
      refused: '1',
    });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
  });

  it('a row at 0 is sent and the backend skips it without a problem', async () => {
    const { user } = await open();
    const [first] = rowIds();
    if (first === undefined) throw new Error('rows');
    await typeInto(user, first, '0');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSaveReceiptsBatch).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(pageText()).not.toContain(message(language, 'vouchers-collection-problems'));
    });
  });

  it('switching the language in the middle keeps the amounts typed', async () => {
    const { user } = await open();
    const [first] = rowIds();
    if (first === undefined) throw new Error('rows');
    await typeInto(user, first, '2880');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    await waitFor(() => {
      const id = rowIds().find((candidate) => cell(candidate, 'code') === '0020');
      expect(cell(id ?? '', 'received')).toContain('2,880');
    });
    const id = rowIds().find((candidate) => cell(candidate, 'code') === '0020');
    expect(cell(id ?? '', 'name')).toBe(other === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
  });

  it('a day with nobody planned says why, with a way back', async () => {
    vi.mocked(vouchersApi.assistCollectionList).mockResolvedValue(collectionList('none'));
    await renderApp('/bikri/collection');
    expect(
      await screen.findByText(shown(message(language, 'vouchers-collection-empty'))),
    ).toBeInTheDocument();
    expect(
      screen.getAllByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'vouchers-collection-back')),
      }).length,
    ).toBeGreaterThan(0);
  });

  it('a list that cannot be read shows the message with a retry that works', async () => {
    vi.mocked(vouchersApi.assistCollectionList).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    await renderApp('/bikri/collection');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-not-implemented'));
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    await waitFor(() => {
      expect(rowIds()).toHaveLength(2);
    });
  });

  it('the receipt screen has a button to the list', async () => {
    const { router } = await renderApp('/bikri/voucher?type=receipt');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'vouchers-receipt-title')),
    });
    await userEvent.setup().click(
      screen.getByRole('button', {
        name: (name) => visible(name).includes(message(language, 'vouchers-collection-open')),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/collection');
    });
  });
});
