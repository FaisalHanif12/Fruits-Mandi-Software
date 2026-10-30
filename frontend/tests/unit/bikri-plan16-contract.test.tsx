// The screens of plan 16 against the real answers of the backend (T4). The backend wrote, in its own
// golden tests, what the 11 new commands answer and what they were asked (tests/fixtures/*.json,
// copied, never edited). Each screen here shows those numbers and sends what the backend's own
// tests send: the draft an edit loads goes back unchanged, a changed rate gives exactly the request
// file, a later sale sends the lines of E2E-16b, the cancel and the write-off send their files.
// Money in the files is paisa; the screen shows whole rupees. The shapes do not depend on the
// language, so these tests run in Urdu, the default.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { SaleInput } from '../../src/bindings/SaleInput';
import * as bikriApi from '../../src/features/bikri/api';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { PAGE_ROWS } from '../helpers/pager';
import { GET_EVENT1, GET_EVENT2 } from '../helpers/bikri-real';
import {
  CANCEL_DONE,
  CANCEL_PREVIEW,
  CANCEL_REQUEST,
  EDIT_PREVIEW,
  EDIT_PREVIEW_DAY_CHANGED,
  EDIT_PREVIEW_REQUEST,
  EDIT_PREVIEW_SELLER_LOCKED,
  EDIT_SAVE,
  EDIT_SAVE_REQUEST,
  LATER_PREVIEW,
  LATER_PREVIEW_REQUEST,
  LATER_SAVE,
  LATER_SAVE_REQUEST,
  OPEN_FOR_EDIT,
  STOCK_LIST,
  UNDO,
  WRITE_OFF,
  WRITE_OFF_REQUEST,
  WRITE_OFF_UNDO,
  WRITE_OFF_UNDO_REQUEST,
} from '../helpers/bikri-plan16-real';
import { blankLine } from '../helpers/bikri-data';
import {
  cellOf,
  lotsGrid,
  named,
  openBook,
  salesGrid,
  textOf,
  typeHeaderAndLot,
  typeTwoLines,
} from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const language = 'ur';
const said = (text: string) => (content: string) => visible(content) === text;
const starts = (text: string) => (name: string) => visible(name).startsWith(text);
const pageText = () => visible(document.body.textContent);

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  resetClientState(language);
  signedIn(sessionUser({ language }));
  fakeBikri();
});

afterEach(() => {
  leaveAppWindow();
});

/** A sale line without the screen's own key and its id of a saved line: what the backend's file has too. */
function withoutKeys(sale: SaleInput | undefined): Omit<SaleInput, 'key' | 'sale_id'> | undefined {
  if (sale === undefined) return undefined;
  const rest: Partial<SaleInput> = { ...sale };
  delete rest.key;
  delete rest.sale_id;
  return rest as Omit<SaleInput, 'key' | 'sale_id'>;
}

describe('goods left against the backend files', () => {
  const open = async () => {
    vi.mocked(bikriApi.bikriStockList).mockResolvedValue(STOCK_LIST);
    const view = await renderApp('/bikri/stock');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'layout-module-stock')),
    });
    return { ...view, user: userEvent.setup() };
  };

  it('shows the lot of the file and its total, and asks with the filters of the contract', async () => {
    await open();
    const row = STOCK_LIST.rows[0];
    if (row === undefined) throw new Error('the file has no lot');
    await waitFor(() => {
      expect(document.querySelector(`.ag-row[row-id="${String(row.lot_id)}"]`)).not.toBeNull();
    });
    const cell = (col: string) =>
      visible(
        document.querySelector(`.ag-row[row-id="${String(row.lot_id)}"] [col-id="${col}"]`)
          ?.textContent,
      );
    expect(cell('number')).toBe(String(row.bikri_number));
    expect(cell('seller')).toBe('رحیم');
    expect(cell('balance')).toBe(String(row.balance));
    expect(visible(document.querySelector('.ag-row-pinned [col-id="balance"]')?.textContent)).toBe(
      String(STOCK_LIST.balance_total),
    );
    expect(bikriApi.bikriStockList).toHaveBeenCalledWith({
      date: null,
      from: null,
      to: null,
      seller_id: null,
      item_id: null,
      include_written_off: false,
      lot_id: null,
      offset: 0,
      limit: PAGE_ROWS,
      search: null,
      sort: null,
    });
  });

  it('a write-off sends the quantity and the reason of the backend file, and tells what was written off', async () => {
    vi.mocked(bikriApi.bikriWriteOff).mockResolvedValue(WRITE_OFF);
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', { name: message(language, 'bikri-stock-write-off') }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('textbox', {
        name: named(message(language, 'bikri-write-off-qty')),
      }),
      WRITE_OFF_REQUEST.qty,
    );
    await user.type(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'bikri-write-off-reason')),
      }),
      WRITE_OFF_REQUEST.reason,
    );
    await user.click(
      within(dialog).getByRole('button', { name: message(language, 'bikri-write-off-confirm') }),
    );
    await waitFor(() => {
      expect(bikriApi.bikriWriteOff).toHaveBeenCalledWith(
        WRITE_OFF_REQUEST.lot_id,
        WRITE_OFF_REQUEST.qty,
        WRITE_OFF_REQUEST.reason,
      );
    });
    expect(
      await screen.findByText(
        said(message(language, 'bikri-write-off-done', { qty: WRITE_OFF.qty })),
      ),
    ).toBeInTheDocument();
  });

  it('undoing a write-off sends the move and the reason of the backend file', async () => {
    const row = STOCK_LIST.rows[0];
    if (row === undefined) throw new Error('the file has no lot');
    vi.mocked(bikriApi.bikriStockList).mockResolvedValue({
      ...STOCK_LIST,
      rows: [
        {
          ...row,
          balance: WRITE_OFF.balance,
          qty_written_off: WRITE_OFF.qty_written_off,
          write_offs: [
            {
              move_id: WRITE_OFF_UNDO_REQUEST.move_id,
              move_date: row.received_date,
              qty: WRITE_OFF.qty,
              reason: WRITE_OFF_REQUEST.reason,
            },
          ],
        },
      ],
    });
    vi.mocked(bikriApi.bikriWriteOffUndo).mockResolvedValue(WRITE_OFF_UNDO);
    await renderApp('/bikri/stock');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', { name: message(language, 'bikri-stock-undo-write-off') }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'bikri-write-off-reason')),
      }),
      WRITE_OFF_UNDO_REQUEST.reason,
    );
    await user.click(
      within(dialog).getByRole('button', {
        name: message(language, 'bikri-write-off-undo-confirm'),
      }),
    );
    await waitFor(() => {
      expect(bikriApi.bikriWriteOffUndo).toHaveBeenCalledWith(
        WRITE_OFF_UNDO_REQUEST.move_id,
        WRITE_OFF_UNDO_REQUEST.reason,
      );
    });
  });
});

describe('a later sale against the backend files (E2E-16b)', () => {
  it('sends the line of the backend file, shows its numbers, and saves what the file saves', async () => {
    vi.mocked(bikriApi.bikriStockList).mockResolvedValue(STOCK_LIST);
    const finished = LATER_PREVIEW.lots[0]?.lines[0];
    if (finished === undefined) throw new Error('the file has no line');
    // The backend matches each line to the key it was sent; the screen's keys are its own.
    vi.mocked(bikriApi.bikriLaterSalePreview).mockImplementation((_lot, _date, sales) => {
      const first = sales[0];
      const typed = first?.buyer_account_id != null && first.seller_rate !== '';
      const lot = LATER_PREVIEW.lots[0];
      return Promise.resolve({
        ...LATER_PREVIEW,
        lots:
          lot === undefined
            ? []
            : [
                {
                  ...lot,
                  lines: sales.map((sale, index) =>
                    typed && index === 0
                      ? { ...finished, key: sale.key }
                      : blankLine(sale.key, index + 1, 5),
                  ),
                },
              ],
      });
    });
    vi.mocked(bikriApi.bikriLaterSaleSave).mockResolvedValue(LATER_SAVE);
    await renderApp(`/bikri/later?lot=${String(LATER_PREVIEW_REQUEST.lot_id)}`);
    const user = userEvent.setup();
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-later-title', { number: 1 })),
    });
    await waitFor(() => {
      expect(textOf(salesGrid(language), 0, 'qty')).toBe('5');
    });
    // Arshad (short code 1) takes the 5 that are left, at 560.
    await user.click(cellOf(salesGrid(language), 0, 'buyer'));
    await user.keyboard('1');
    await screen.findAllByRole('option');
    await user.keyboard('{Enter}{Enter}560{Enter}');
    await waitFor(() => {
      expect(textOf(salesGrid(language), 0, 'amount')).toBe('2,800');
    });
    expect(textOf(salesGrid(language), 0, 'buyerTotal')).toBe('3,008');
    // The request has the lot, the day and the line of the backend's own file.
    const sent = vi.mocked(bikriApi.bikriLaterSalePreview).mock.calls.at(-1);
    expect(sent?.[0]).toBe(LATER_PREVIEW_REQUEST.lot_id);
    expect(sent?.[1]).toBe(LATER_PREVIEW_REQUEST.date);
    expect(withoutKeys(sent?.[2][0])).toEqual(withoutKeys(LATER_PREVIEW_REQUEST.sales[0]));
    press('F10');
    await waitFor(() => {
      expect(bikriApi.bikriLaterSaleSave).toHaveBeenCalled();
    });
    const saved = vi.mocked(bikriApi.bikriLaterSaleSave).mock.calls[0];
    expect(saved?.[0]).toBe(LATER_SAVE_REQUEST.lot_id);
    expect(saved?.[1]).toBe(LATER_SAVE_REQUEST.date);
    expect(saved?.[3]).toBe(LATER_SAVE_REQUEST.confirm_cash_below_zero);
    expect(withoutKeys(saved?.[2][0])).toEqual(withoutKeys(LATER_SAVE_REQUEST.sales[0]));
  });
});

describe('an edit against the backend files (E2E-13)', () => {
  const openEdit = async () => {
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(GET_EVENT1);
    vi.mocked(bikriApi.bikriOpenForEdit).mockResolvedValue(OPEN_FOR_EDIT);
    vi.mocked(bikriApi.bikriEditPreview).mockResolvedValue(EDIT_PREVIEW);
    vi.mocked(bikriApi.bikriEditSave).mockResolvedValue(EDIT_SAVE);
    const view = await renderApp(
      `/bikri/book?id=${String(OPEN_FOR_EDIT.bikri_id)}&edit=${String(OPEN_FOR_EDIT.entry_id)}`,
    );
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-edit-title', { number: OPEN_FOR_EDIT.number })),
    });
    await waitFor(() => {
      expect(textOf(salesGrid(language), 0, 'buyer')).toBe('اکرم ٹول ٹیکس');
    });
    return { ...view, user: userEvent.setup() };
  };

  it('the draft the backend loads goes back to it unchanged', async () => {
    await openEdit();
    await waitFor(() => {
      expect(bikriApi.bikriEditPreview).toHaveBeenCalled();
    });
    const first = vi.mocked(bikriApi.bikriEditPreview).mock.calls[0];
    expect(first?.[0]).toBe(OPEN_FOR_EDIT.entry_id);
    expect(first?.[1]).toEqual(OPEN_FOR_EDIT.draft);
    // Nothing is changed yet.
    expect(pageText()).toContain(message(language, 'bikri-edit-no-changes'));
  });

  it('a changed rate gives exactly the request file, shows the answer, and F10 saves what the file saves', async () => {
    const { user } = await openEdit();
    await user.click(cellOf(salesGrid(language), 0, 'rate'));
    await user.keyboard('610{Enter}');
    await waitFor(() => {
      expect(cellOf(salesGrid(language), 0, 'rate')).toHaveClass('cell-changed');
    });
    await waitFor(() => {
      const last = vi.mocked(bikriApi.bikriEditPreview).mock.calls.at(-1);
      expect(last?.[0]).toBe(EDIT_PREVIEW_REQUEST.entry_id);
      expect(last?.[1]).toEqual(EDIT_PREVIEW_REQUEST.draft);
    });
    // The backend's answer for rate 610: Akram pays 13,092 (E2E-13).
    await waitFor(() => {
      expect(textOf(salesGrid(language), 0, 'buyerTotal')).toBe('13,092');
    });
    press('F10');
    await waitFor(() => {
      expect(bikriApi.bikriEditSave).toHaveBeenCalled();
    });
    const saved = vi.mocked(bikriApi.bikriEditSave).mock.calls[0];
    expect(saved?.[0]).toBe(EDIT_SAVE_REQUEST.entry_id);
    expect(saved?.[1]).toBe(EDIT_SAVE_REQUEST.version);
    expect(saved?.[2]).toEqual(EDIT_SAVE_REQUEST.draft);
    // The posting is of today, so no reason is asked (the backend's test sends one; it is optional).
    expect(saved?.[3]).toBeNull();
    expect(saved?.[4]).toBe(EDIT_SAVE_REQUEST.confirm_cash_below_zero);
  });
});

describe('the edit rules against the backend files', () => {
  const openWith = async (answer: typeof EDIT_PREVIEW) => {
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(GET_EVENT1);
    vi.mocked(bikriApi.bikriOpenForEdit).mockResolvedValue(OPEN_FOR_EDIT);
    vi.mocked(bikriApi.bikriEditPreview).mockResolvedValue(answer);
    await renderApp(
      `/bikri/book?id=${String(OPEN_FOR_EDIT.bikri_id)}&edit=${String(OPEN_FOR_EDIT.entry_id)}`,
    );
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-edit-title', { number: OPEN_FOR_EDIT.number })),
    });
  };

  it('the seller of a lot that a later sale rests on is told in words at its lot', async () => {
    await openWith(EDIT_PREVIEW_SELLER_LOCKED);
    expect(
      await screen.findByText(
        said(message(language, 'err-edit-not-allowed', { rule: 'lot_seller_locked' })),
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(said(message(language, 'bikri-where-lot', { lot: 1 }))),
    ).toBeInTheDocument();
    expect(pageText()).not.toContain('lot_seller_locked');
  });

  it('a changed day is told under the day field, next to the shares problem of that day', async () => {
    await openWith(EDIT_PREVIEW_DAY_CHANGED);
    expect(
      await screen.findByText(
        said(message(language, 'err-edit-not-allowed', { rule: 'day_changed' })),
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(said(message(language, 'err-shares-not-100', { total: '0' }))),
    ).toBeInTheDocument();
    expect(pageText()).not.toContain('ERR_EDIT_NOT_ALLOWED');
  });
});

describe('a cancel against the backend files (E2E-15)', () => {
  it('names the entry and the cash of the file, and sends the reason and the version of the file', async () => {
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(GET_EVENT2);
    vi.mocked(bikriApi.bikriCancelPreview).mockResolvedValue(CANCEL_PREVIEW);
    vi.mocked(bikriApi.bikriCancel).mockResolvedValue(CANCEL_DONE);
    await renderApp(`/bikri/book?id=${String(CANCEL_PREVIEW.bikri_id)}`);
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', {
        name: said(message(language, 'bikri-cancel-bikri')),
      }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await waitFor(() => {
      // The entry total 25,600 and the cash before and after of the file.
      expect(visible(dialog.textContent)).toContain('25,600');
    });
    expect(visible(dialog.textContent)).toContain('57,800');
    expect(visible(dialog.textContent)).toContain('45,000');
    await user.type(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'bikri-cancel-reason')),
      }),
      CANCEL_REQUEST.reason,
    );
    await user.click(
      within(dialog).getByRole('button', {
        name: starts(message(language, 'bikri-cancel-confirm')),
      }),
    );
    await waitFor(() => {
      expect(bikriApi.bikriCancel).toHaveBeenCalledWith(
        CANCEL_REQUEST.id,
        CANCEL_REQUEST.version,
        CANCEL_REQUEST.reason,
        CANCEL_REQUEST.confirm_cash_below_zero,
      );
    });
  });
});

describe('the undo against the backend file', () => {
  it('Alt+Z sends the id and brings the values of the file back in a new entry', async () => {
    vi.mocked(bikriApi.bikriCancelPreview).mockResolvedValue(CANCEL_PREVIEW);
    vi.mocked(bikriApi.bikriUndo).mockResolvedValue(UNDO);
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
    });
    press('F10');
    await screen.findByRole('button', { name: starts(message(language, 'bikri-undo')) });
    press('KeyZ', { altKey: true });
    await waitFor(() => {
      expect(bikriApi.bikriUndo).toHaveBeenCalled();
    });
    // The id of the saved بکری (the fake backend's 41) and the confirmation flag of the file.
    expect(vi.mocked(bikriApi.bikriUndo).mock.calls[0]).toEqual([41, false]);
    // The draft of the file (keys lot:3 and sale:7, no ids) is in the new entry.
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe('رحیم');
    });
    expect(textOf(lotsGrid(language), 0, 'qty')).toBe('50');
    expect(textOf(salesGrid(language), 0, 'buyer')).toBe('اکرم ٹول ٹیکس');
    expect(textOf(salesGrid(language), 0, 'qty')).toBe('20');
    expect(textOf(salesGrid(language), 0, 'rate')).toBe('600');
    expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
    expect(textOf(salesGrid(language), 1, 'rate')).toBe('580');
  });
});
