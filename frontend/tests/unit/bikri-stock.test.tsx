// باقی مال, the lots with goods left (plan 16, PRD 12.8, FR-12.14 to FR-12.16), in the Urdu and the
// English app, against a fake backend: the list and its total, the card of the lot, the four states,
// Enter starting a later sale, a write-off with a reason and its undo. Every figure on the screen is
// a figure the backend sent.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { fakePlan16, refused, stockRow } from '../helpers/bikri-plan16';
import { stockList } from '../helpers/bikri-plan16-data';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS, sliceOf } from '../helpers/paged';
import { PAGE_ROWS, pagerButton, rangeText } from '../helpers/pager';

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

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

/** Matches an element whose own text is `text`, without the isolation marks Fluent adds. */
const said = (text: string) => (content: string) => visible(content) === text;

function cell(rowId: number, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${String(rowId)}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

describe.each(LANGUAGES)('باقی مال (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const names = {
    rahim: language === 'ur' ? 'رحیم' : 'Rahim',
    banana: language === 'ur' ? 'کیلا' : 'Banana',
    box: language === 'ur' ? 'پیٹی' : 'Box',
  };

  const open = async () => {
    const view = await renderApp('/bikri/stock');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'layout-module-stock')),
    });
    return { ...view, user: userEvent.setup() };
  };

  it('lists the lots with goods left, as the backend counted them, with the total', async () => {
    fakePlan16();
    await open();
    await waitFor(() => {
      expect(cell(40, 'number')).toBe('1');
    });
    expect(cell(40, 'seller')).toBe(names.rahim);
    expect(cell(40, 'item')).toBe(
      message(language, 'bikri-stock-item-packing', { item: names.banana, packing: names.box }),
    );
    expect(cell(40, 'balance')).toBe('5');
    // The total row is the backend's figure under the balance column.
    const pinned = document.querySelector('.ag-row-pinned');
    expect(pinned).not.toBeNull();
    expect(visible(pinned?.querySelector('[col-id="balance"]')?.textContent)).toBe('5');
    expect(visible(pinned?.querySelector('[col-id="seller"]')?.textContent)).toBe(
      message(language, 'glossary-meezan'),
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
    });
  });

  it('the card of the lot has every figure of FR-12.14', async () => {
    fakePlan16();
    await open();
    const card = await screen.findByRole('heading', {
      name: named(message(language, 'bikri-stock-lot-title', { number: 1, lot: 1 })),
    });
    const text = visible(card.closest('section')?.textContent);
    expect(text).toContain('09-10-2026');
    for (const figure of ['50', '45', '5']) expect(text).toContain(figure);
    expect(text).toContain(names.rahim);
    expect(text).toContain(names.banana);
    expect(text).toContain(names.box);
  });

  it('fruit taken from the lot is a column and a figure on the card, and the balance is the backend’s (C16)', async () => {
    // 50 received, 45 sold, 2 taken as fruit (E2E-42): the balance is 3, as the backend counted it.
    fakePlan16({ rows: [stockRow({ qty_taken: 2, balance: 3 })] });
    await open();
    await waitFor(() => {
      expect(cell(40, 'number')).toBe('1');
    });
    // The column of the taken fruit comes with the wide layout step; the card shows it at every size.
    expect(cell(40, 'balance')).toBe('3');
    const card = await screen.findByRole('heading', {
      name: named(message(language, 'bikri-stock-lot-title', { number: 1, lot: 1 })),
    });
    const facts = card.closest('section')?.querySelectorAll('dl > div');
    const taken = Array.from(facts ?? []).find((fact) =>
      visible(fact.textContent).includes(message(language, 'bikri-stock-col-taken')),
    );
    expect(taken).toBeDefined();
    expect(visible(taken?.textContent)).toContain('2');
  });

  it('an empty list says why and has a button (FR-7.46)', async () => {
    fakePlan16({ rows: [] });
    await open();
    expect(
      await screen.findByText(said(message(language, 'bikri-stock-empty'))),
    ).toBeInTheDocument();
    const newButton = screen
      .getAllByRole('button')
      .find((button) => visible(button.textContent).includes(message(language, 'bikri-new')));
    expect(newButton).toBeDefined();
  });

  it('an error has a retry that asks again', async () => {
    fakePlan16();
    vi.mocked(bikriApi.bikriStockList).mockRejectedValueOnce(refused('ERR_INTERNAL'));
    const { user } = await open();
    expect(
      await screen.findByText(said(message(language, 'bikri-stock-failed'))),
    ).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    await waitFor(() => {
      expect(cell(40, 'number')).toBe('1');
    });
  });

  it('Enter on a lot starts a later sale of that lot', async () => {
    fakePlan16();
    const { router, user } = await open();
    await waitFor(() => {
      expect(cell(40, 'number')).toBe('1');
    });
    const target = document.querySelector<HTMLElement>('.ag-row[row-id="40"] [col-id="number"]');
    if (target === null) throw new Error('no cell');
    await user.click(target);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/later');
    });
    expect(router.state.location.search).toMatchObject({ lot: 40 });
  });

  it('the tick for written-off lots asks the backend for them', async () => {
    fakePlan16();
    const { user } = await open();
    await user.click(
      screen.getByRole('checkbox', {
        name: named(message(language, 'bikri-stock-include-written-off')),
      }),
    );
    await waitFor(() => {
      expect(bikriApi.bikriStockList).toHaveBeenLastCalledWith(
        expect.objectContaining({ include_written_off: true }),
      );
    });
  });

  it('a write-off sends the quantity as typed and the reason, and tells it was done', async () => {
    const plan = fakePlan16();
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', { name: message(language, 'bikri-stock-write-off') }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('textbox', {
        name: named(message(language, 'bikri-write-off-qty')),
      }),
      '2+3',
    );
    await user.type(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'bikri-write-off-reason')),
      }),
      'spoiled',
    );
    await user.click(
      within(dialog).getByRole('button', { name: message(language, 'bikri-write-off-confirm') }),
    );
    await waitFor(() => {
      expect(plan.writeOffs).toEqual([{ lotId: 40, qty: '2+3', reason: 'spoiled' }]);
    });
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
  });

  it('a write-off the backend refuses stays open with the words and what was typed', async () => {
    fakePlan16();
    vi.mocked(bikriApi.bikriWriteOff).mockRejectedValue(
      refused('ERR_WRITE_OFF_EXCEEDS_BALANCE', 'qty', { available: 5 }),
    );
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', { name: message(language, 'bikri-stock-write-off') }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('textbox', {
        name: named(message(language, 'bikri-write-off-qty')),
      }),
      '9',
    );
    await user.type(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'bikri-write-off-reason')),
      }),
      'spoiled',
    );
    await user.click(
      within(dialog).getByRole('button', { name: message(language, 'bikri-write-off-confirm') }),
    );
    expect(
      await within(dialog).findByText(
        said(message(language, 'err-write-off-exceeds-balance', { available: 5 })),
      ),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByRole('textbox', {
        name: named(message(language, 'bikri-write-off-qty')),
      }),
    ).toHaveValue('9');
  });

  it('a reason is needed: an empty one is told at the field and nothing is sent', async () => {
    const plan = fakePlan16();
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', { name: message(language, 'bikri-stock-write-off') }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.click(
      within(dialog).getByRole('button', { name: message(language, 'bikri-write-off-confirm') }),
    );
    expect(await within(dialog).findByText(message(language, 'err-required'))).toBeInTheDocument();
    expect(plan.writeOffs).toEqual([]);
  });

  it('undoes a write-off by choosing it and giving a reason', async () => {
    const plan = fakePlan16({
      rows: [
        stockRow({
          balance: 0,
          qty_written_off: 5,
          write_offs: [{ move_id: 70, move_date: '2026-10-10', qty: 5, reason: 'spoiled' }],
        }),
      ],
    });
    const { user } = await open();
    await user.click(
      screen.getByRole('checkbox', {
        name: named(message(language, 'bikri-stock-include-written-off')),
      }),
    );
    await user.click(
      await screen.findByRole('button', { name: message(language, 'bikri-stock-undo-write-off') }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'bikri-write-off-reason')),
      }),
      'found it',
    );
    await user.click(
      within(dialog).getByRole('button', {
        name: message(language, 'bikri-write-off-undo-confirm'),
      }),
    );
    await waitFor(() => {
      expect(plan.writeOffUndos).toEqual([{ moveId: 70, reason: 'found it' }]);
    });
  });

  it('150 lots come one page at a time: 25 at once, the count and the total are the backend’s, Next page loads the next 25', async () => {
    fakePlan16();
    const all = Array.from({ length: 150 }, (_, index) =>
      stockRow({ lot_id: 3000 + index, lot_no: index + 1, bikri_number: index + 1 }),
    );
    vi.mocked(bikriApi.bikriStockList).mockImplementation((input) =>
      Promise.resolve({
        ...stockList(all),
        rows: sliceOf(all, input.offset, input.limit),
        // The balance of the whole filter, not the sum of the rows on screen.
        balance_total: 777,
      }),
    );
    await open();
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(rangeText(language, 1, 25, 150));
    });
    await waitFor(() => {
      expect(
        visible(document.querySelector('.ag-row-pinned [col-id="balance"]')?.textContent),
      ).toBe('777');
    });
    expect(bikriApi.bikriStockList).toHaveBeenLastCalledWith(
      expect.objectContaining({ offset: 0, limit: PAGE_ROWS }),
    );
    const user = userEvent.setup();
    await user.click(await pagerButton(language, 'next'));
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(rangeText(language, 26, 50, 150));
    });
    expect(bikriApi.bikriStockList).toHaveBeenLastCalledWith(
      expect.objectContaining({ offset: PAGE_ROWS, limit: PAGE_ROWS }),
    );
    // The balance is over the whole filter on every page.
    expect(visible(document.querySelector('.ag-row-pinned [col-id="balance"]')?.textContent)).toBe(
      '777',
    );
  });
});
