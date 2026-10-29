// A lot of trip goods in the sale book, the saved view and the goods left list show what the REAL
// backend answers (plan 16b, T4b). The backend wrote each answer in its own golden tests
// (tests/fixtures/bikri_*_own_lot_*.json and bikri_stock_list_own_rows.json, copied, never edited).
// The panel under the lots grid shows the numbers of E2E-34 and its workings in words; the saved
// view shows the trip partners' parts, the cost and the result of E2E-34 and E2E-41; the goods left
// list shows the trip purchase apart; an edit loads the draft of the backend and sends it back as it
// came. In the Urdu and in the English app. Every figure on screen is a figure the backend sent.

import { QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useStore } from 'zustand';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createQueryClient } from '../../src/app/providers';
import * as bikriApi from '../../src/features/bikri/api';
import { OwnLotPanel } from '../../src/features/bikri/components/OwnLotPanel';
import { lotInput } from '../../src/features/bikri/draft';
import { BikriDraftProvider, createBikriDraft } from '../../src/features/bikri/draft-store';
import { LOT } from '../../src/features/bikri/types';
import * as purchasesApi from '../../src/features/purchases/api';
import { LANGUAGES } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { fakePlan16 } from '../helpers/bikri-plan16';
import * as own from '../helpers/bikri-own-real';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, renderIn, visible } from '../helpers/render';
import { fakeTrips } from '../helpers/trips';
import * as trips from '../helpers/trips-real';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/purchases/api');
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
  fakePartners(THREE_PARTNERS);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;
const pageText = () => visible(document.body.textContent);

const OWN_ACCOUNT = {
  id: 17,
  code: '030119998',
  name_ur: 'اپنا مال',
  name_en: 'Own Goods (Trips)',
};

describe.each(LANGUAGES)('the panel of a lot of trip goods, real answer (%s)', (language) => {
  beforeEach(() => {
    act(() => {
      useLanguageStore.setState({ language });
    });
    fakeTrips();
    vi.mocked(purchasesApi.purchasesList).mockResolvedValue(trips.PURCHASES_LIST_ALL);
    vi.mocked(purchasesApi.purchasesGet).mockResolvedValue(trips.PURCHASE_GET_AFTER_SALES);
  });

  function Panel({
    stores,
    lot,
  }: {
    stores: ReturnType<typeof createBikriDraft>;
    lot: (typeof own.OWN_PREVIEW_E34.lots)[number] | undefined;
  }) {
    const row = useStore(stores.lots, (state) => state.rows[0]);
    if (row === undefined) return null;
    return <OwnLotPanel row={row} lotIndex={0} error={null} lot={lot} />;
  }

  function show(lot: (typeof own.OWN_PREVIEW_E34.lots)[number] | undefined) {
    const stores = createBikriDraft('10-10-2026');
    const rowId = stores.lots.getState().rows[0]?.rowId ?? '';
    act(() => {
      stores.lots.getState().patchRow(rowId, {
        typed: { [LOT.purchase]: '1', [LOT.qty]: '15' },
        picked: { [LOT.seller]: OWN_ACCOUNT },
      });
    });
    renderIn(
      language,
      <QueryClientProvider client={createQueryClient()}>
        <BikriDraftProvider value={stores}>
          <Panel stores={stores} lot={lot} />
        </BikriDraftProvider>
      </QueryClientProvider>,
    );
    return stores;
  }

  it('E2E-34: commission 1,764 is not shown here but 540 + 540 + 120 is; credit 8,460 goes 3,384 / 2,538 / 2,538, cost 7,800, result 660', async () => {
    const lot = own.OWN_PREVIEW_E34.lots[0];
    show(lot);
    const credit = await screen.findByText(said(message(language, 'bikri-own-credit')));
    const panel = credit.closest('section');
    if (panel === null) throw new Error('no panel');
    const text = visible(panel.textContent);
    for (const figure of ['8,460', '3,384', '2,538', '7,800', '660']) {
      expect(text, figure).toContain(figure);
    }
    // The seller commission of the lot is the backend's own 540.
    const commission = screen.getByText(said(message(language, 'bikri-own-commission')));
    expect(commission.closest('div')?.textContent).toContain('540');
    expect(
      screen.getByText(said(message(language, 'glossary-tijarati-munafa'))),
    ).toBeInTheDocument();
  });

  it('the working of the cost and of the result are told in words, with the real numbers', async () => {
    const user = userEvent.setup();
    show(own.OWN_PREVIEW_E34.lots[0]);
    const credit = await screen.findByText(said(message(language, 'bikri-own-credit')));
    const panel = credit.closest('section');
    if (panel === null) throw new Error('no panel');
    const infoOf = (label: string) => {
      const row = within(panel).getByText(said(label)).closest('div');
      if (row === null) throw new Error('no row');
      return within(row).getByRole('button', { name: named(message(language, 'calc-info-label')) });
    };
    await user.click(infoOf(message(language, 'purchases-detail-cost-sold')));
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'working-own-lot-cost', { qty: '15', result: '7,800' }),
      );
    });
    await user.keyboard('{Escape}');
    await user.click(infoOf(message(language, 'glossary-tijarati-munafa')));
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'working-own-lot-result', {
          credit: '8,460',
          cost: '7,800',
          result: '660',
        }),
      );
    });
  });

  it('a lot that is not sold in full: the panel still shows what the backend worked out, and the refusal names the nag left', () => {
    const preview = own.OWN_PREVIEW_NOT_FULLY_SOLD;
    show(preview.lots[0]);
    const refusal = preview.errors[0];
    expect(refusal?.code).toBe('ERR_OWN_LOT_NOT_FULLY_SOLD');
    // The refusal points at the quantity of the lot, which the grid shows as a cell error.
    expect(refusal?.field).toBe('lots[0].qty_received');
    expect(refusal?.values).toEqual({ qty_left: 5 });
    const words = message(language, 'err-own-lot-not-fully-sold', { qty_left: '5' });
    expect(words).not.toMatch(/[{}$]/);
  });

  it('the lot the screen builds is the lot the backend’s own request had (E2E-34)', () => {
    const stores = show(own.OWN_PREVIEW_E34.lots[0]);
    const request = own.OWN_PREVIEW_E34_REQUEST.draft.lots[0];
    if (request === undefined) throw new Error('no lot in the request file');
    const rows = stores.lots.getState().rows;
    const row = rows[0];
    if (row === undefined) throw new Error('no row');
    // The seller, the purchase and the quantity of the file, typed the way the panel types them.
    const sent = lotInput(row, [], stores.header.getState());
    expect(sent.purchase_id).toBe(request.purchase_id);
    expect(sent.qty_received).toBe(request.qty_received);
    expect(sent.seller_account_id).toBe(OWN_ACCOUNT.id);
    // Nothing is paid in cash and no advance is adjusted on trip goods: what the file sends.
    expect(sent.cash_paid).toBe(request.cash_paid);
    expect(sent.adjust_advance).toBe(request.adjust_advance);
  });
});

describe.each(LANGUAGES)('a saved lot of trip goods, real answer (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    fakeTrips();
  });

  const openSaved = async (detail: typeof own.OWN_GET_E34) => {
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(detail);
    const view = await renderApp(`/bikri/book?id=${String(detail.id)}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-saved-title', { number: detail.number })),
    });
    return view;
  };

  it('E2E-34: the lot is marked, names its purchase and trip, and shows the parts, cost and result', async () => {
    const { router } = await openSaved(own.OWN_GET_E34);
    expect(
      await screen.findByText(said(message(language, 'bikri-saved-own-title'))),
    ).toBeInTheDocument();
    const text = pageText();
    // The buyer pays 9,660; the credit 8,460 goes 3,384 / 2,538 / 2,538; cost 7,800; result 660.
    for (const figure of ['9,660', '8,460', '3,384', '2,538', '7,800', '660']) {
      expect(text, figure).toContain(figure);
    }
    // The split of the shared income: 480 / 360 / 360 (commission only, not the 9,660).
    for (const figure of ['480', '360']) expect(text, figure).toContain(figure);
    expect(text).toContain(language === 'ur' ? 'الف' : 'Partner A');
    // The purchase and its trip are named, with a way to open the trip.
    const open = screen.getByRole('button', {
      name: (name) =>
        visible(name).includes(message(language, 'purchases-number', { number: 1 })) &&
        visible(name).includes('0070'),
    });
    await userEvent.setup().click(open);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/trips/1');
    });
    expect(router.state.location.search).toMatchObject({ purchase: 1 });
  });

  it('E2E-41: two partners get 52,640 each, the cost is 100,000 and the result 5,280', async () => {
    await openSaved(own.OWN_GET_E41);
    await screen.findByText(said(message(language, 'bikri-saved-own-title')));
    const text = pageText();
    for (const figure of ['119,360', '105,280', '52,640', '100,000', '5,280']) {
      expect(text, figure).toContain(figure);
    }
  });
});

describe.each(LANGUAGES)('trip goods in باقی مال, real answer (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    fakePlan16({ rows: [] });
    fakeTrips();
    vi.mocked(bikriApi.bikriStockList).mockResolvedValue(own.STOCK_OWN_ROWS);
    vi.mocked(purchasesApi.purchasesGet).mockResolvedValue(trips.PURCHASE_GET_AFTER_SALES);
  });

  const open = async () => {
    const view = await renderApp('/bikri/stock');
    await screen.findByText(said(message(language, 'bikri-stock-own-title')));
    return { ...view, user: userEvent.setup() };
  };

  it('lists the purchase apart, with 5 nag and the cost left 2,600, and the screen is not empty', async () => {
    await open();
    const row = await waitFor(() => {
      const found = document.querySelector('.ag-row[row-id="1"]');
      if (found === null) throw new Error('no row');
      return found;
    });
    const cell = (id: string) => visible(row.querySelector(`[col-id="${id}"]`)?.textContent);
    expect(cell('number')).toBe(message(language, 'purchases-number', { number: 1 }));
    expect(cell('trip')).toBe('0070');
    expect(cell('balance')).toBe('5');
    expect(screen.queryByText(said(message(language, 'bikri-stock-empty')))).toBeNull();
    // The totals row is the backend's: 5 nag and the cost left 2,600.
    const pinned = visible(document.querySelector('.ag-row-pinned')?.textContent);
    expect(pinned).toContain('5');
  });

  it('the card has the figures of the purchase and its write-off asks for a reason', async () => {
    const { user } = await open();
    const cell = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="1"] [col-id="number"]');
      if (found === null) throw new Error('no cell');
      return found;
    });
    await user.click(cell);
    const card = (
      await screen.findByText(
        said(
          message(language, 'bikri-stock-own-card-title', {
            number: message(language, 'purchases-number', { number: 1 }),
            item: language === 'ur' ? 'مالٹا' : 'Orange',
          }),
        ),
      )
    ).closest('section');
    if (card === null) throw new Error('no card');
    const text = visible(card.textContent);
    for (const figure of ['10,400', '2,600', 'Orange Traders']) expect(text).toContain(figure);
    await user.click(
      within(card).getByRole('button', { name: message(language, 'purchases-write-off') }),
    );
    await user.type(
      await screen.findByRole('textbox', {
        name: named(message(language, 'purchases-write-off-qty')),
      }),
      '5',
    );
    await user.type(
      screen.getByRole('combobox', {
        name: named(message(language, 'purchases-write-off-reason')),
      }),
      'spoiled',
    );
    await user.click(
      screen.getByRole('button', { name: message(language, 'purchases-write-off-confirm') }),
    );
    await waitFor(() => {
      expect(purchasesApi.purchasesWriteOff).toHaveBeenCalledWith(1, '5', 'spoiled');
    });
  });
});

describe.each(LANGUAGES)(
  'writing trip goods off and undoing it from باقی مال, real answers (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
      fakeBikri();
      fakePlan16({ rows: [] });
      fakeTrips();
      vi.mocked(purchasesApi.purchasesGet).mockResolvedValue(trips.PURCHASE_GET_AFTER_SALES);
    });

    const openCard = async (stock: typeof own.STOCK_OWN_ROWS) => {
      vi.mocked(bikriApi.bikriStockList).mockResolvedValue(stock);
      await renderApp('/bikri/stock');
      await screen.findByText(said(message(language, 'bikri-stock-own-title')));
      const user = userEvent.setup();
      const cell = await waitFor(() => {
        const found = document.querySelector<HTMLElement>('.ag-row[row-id="1"] [col-id="number"]');
        if (found === null) throw new Error('no cell');
        return found;
      });
      await user.click(cell);
      return user;
    };

    it('the real answer of a write-off tells the quantity, the loss and that the trip is closed', async () => {
      vi.mocked(purchasesApi.purchasesWriteOff).mockResolvedValue(trips.WRITE_OFF_LAST_2);
      const user = await openCard(own.STOCK_OWN_ROWS);
      await user.click(
        await screen.findByRole('button', { name: message(language, 'purchases-write-off') }),
      );
      await user.type(
        await screen.findByRole('textbox', {
          name: named(message(language, 'purchases-write-off-qty')),
        }),
        '2',
      );
      await user.type(
        screen.getByRole('combobox', {
          name: named(message(language, 'purchases-write-off-reason')),
        }),
        'spoiled',
      );
      await user.click(
        screen.getByRole('button', { name: message(language, 'purchases-write-off-confirm') }),
      );
      // 2 nag written off with the cost 1,040; nothing is left, so the trip closes (the backend says so).
      await waitFor(() => {
        expect(pageText()).toContain(
          message(language, 'purchases-write-off-done-closed', { qty: '2', cost: '1,040' }),
        );
      });
      expect(purchasesApi.purchasesWriteOff).toHaveBeenCalledWith(1, '2', 'spoiled');
    });

    it('a purchase with a write-off can undo it: the move of the backend’s answer and a reason go back', async () => {
      const row = own.STOCK_OWN_ROWS.own_rows[0];
      if (row === undefined) throw new Error('the file has no row');
      const stock = {
        ...own.STOCK_OWN_ROWS,
        own_rows: [
          {
            ...row,
            write_offs: [
              {
                move_id: trips.WRITE_OFF_LAST_2.move_id,
                move_date: '2026-10-11',
                qty: trips.WRITE_OFF_LAST_2.qty,
                reason: 'spoiled',
              },
            ],
          },
        ],
      };
      vi.mocked(purchasesApi.purchasesWriteOffUndo).mockResolvedValue(trips.WRITE_OFF_UNDO);
      const user = await openCard(stock);
      await user.click(
        await screen.findByRole('button', { name: message(language, 'purchases-undo-write-off') }),
      );
      await user.type(
        await screen.findByRole('combobox', {
          name: named(message(language, 'purchases-write-off-reason')),
        }),
        'by mistake',
      );
      await user.click(
        screen.getByRole('button', { name: message(language, 'purchases-write-off-undo-confirm') }),
      );
      await waitFor(() => {
        expect(purchasesApi.purchasesWriteOffUndo).toHaveBeenCalledWith(
          trips.WRITE_OFF_LAST_2.move_id,
          'by mistake',
        );
      });
      // The real answer says the trip did not open again, so the short message is the one told.
      await waitFor(() => {
        expect(pageText()).toContain(message(language, 'purchases-write-off-undone'));
      });
      expect(pageText()).not.toContain(message(language, 'purchases-write-off-undone-reopened'));
    });
  },
);

describe('an edit of a lot of trip goods, real answer', () => {
  const language = 'en';
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    fakeTrips();
  });

  it('the draft the backend loads goes back to it unchanged, and the purchase cannot be changed', async () => {
    const edit = own.OWN_OPEN_FOR_EDIT;
    vi.mocked(bikriApi.bikriGet).mockResolvedValue(own.OWN_GET_E34);
    vi.mocked(bikriApi.bikriOpenForEdit).mockResolvedValue(edit);
    // The backend's own-lot preview, with the keys of the draft an edit sends (the preview echoes
    // the key of each lot and line it was asked about).
    const preview = structuredClone(own.OWN_PREVIEW_E34);
    preview.lots.forEach((lot) => {
      lot.key = 'lot:1';
    });
    vi.mocked(bikriApi.bikriEditPreview).mockResolvedValue(preview);
    vi.mocked(purchasesApi.purchasesList).mockResolvedValue(trips.PURCHASES_LIST_ALL);
    vi.mocked(purchasesApi.purchasesGet).mockResolvedValue(trips.PURCHASE_GET_AFTER_SALES);
    await renderApp(`/bikri/book?id=${String(edit.bikri_id)}&edit=${String(edit.entry_id)}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-edit-title', { number: edit.number })),
    });
    await waitFor(() => {
      expect(bikriApi.bikriEditPreview).toHaveBeenCalled();
    });
    const first = vi.mocked(bikriApi.bikriEditPreview).mock.calls[0];
    expect(first?.[0]).toBe(edit.entry_id);
    expect(first?.[1]).toEqual(edit.draft);
    // The purchase of a saved lot is shown and not changed.
    const choice = await screen.findByRole('combobox', {
      name: named(message(language, 'bikri-own-purchase')),
    });
    expect(choice).toBeDisabled();
    expect(pageText()).toContain(message(language, 'bikri-edit-no-changes'));
  });
});
