// Plan 78 C1, PRD FR-12.50: trip goods and garden goods are loaded into a بکری BY HAND. The بکری page has three
// load buttons next to "+ لاٹ", each shown only when it has goods: "From باقی مال" (lots of an earlier بکری),
// "Market trip goods" and "Garden goods". A card lists every row of its kind with goods left for our shop; the one
// the munshi picks becomes a lot (a trip purchase is sold by اپنا مال, a garden's goods by the garden), and it is
// posted as before. Nothing comes into a new بکری by itself any more, and a saved بکری, a later sale or an edit
// never has the buttons.
//
// Both languages, against the fake backend of Event 1. The purchases and the arrivals are the backend's real
// rows (E2E-34 and E2E-40), with the number, the day and the goods left set for each test. The numbers on the
// screen after a line is typed are the backend's real preview of E2E-34.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { GoodsSources } from '../../src/bindings/GoodsSources';
import type { ReadyGoodsRow } from '../../src/bindings/ReadyGoodsRow';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import { AKRAM, ARSHAD, KARIM, CHUKTA_KHATA, RAHIM, fakeBikri } from '../helpers/bikri';
import * as arrivals from '../helpers/arrivals-real';
import * as own from '../helpers/bikri-own-real';
import * as real from '../helpers/plan78-real';
import { fakePlan16 } from '../helpers/bikri-plan16';
import { focusedIn, lotsGrid, named, salesGrid, textOf } from '../helpers/bikri-screen';
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
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, paragraph, visible } from '../helpers/render';
import { fakeTrips } from '../helpers/trips';
import * as trips from '../helpers/trips-real';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/purchases/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const OWN_GOODS = account(17, 'seller', '030119998', 'اپنا مال', 'Own Goods (Trips)');

/** The day of the fake backend (helpers/app.tsx): the new بکری opens on it. */
const TODAY = '2026-10-10';

/** The backend's real row of a trip purchase with goods left (bikri_goods_to_load), with the facts a test sets. */
function purchase(
  id: number,
  number: number,
  facts: Partial<Pick<ReadyGoodsRow, 'sell_date' | 'qty_left' | 'item'>> = {},
): ReadyGoodsRow {
  const trip = real.GOODS_TO_LOAD_TRIP.rows[0];
  if (trip === undefined) throw new Error('the fixture has no trip row');
  return {
    ...structuredClone(trip),
    id,
    number,
    sell_date: TODAY,
    // The orange of E2E-34, as the purchase of the real answer has it.
    item: trips.PURCHASE_GET_AFTER_SALES.item,
    qty: 2,
    qty_left: 2,
    ...facts,
  };
}

/** The backend's real row of a garden's goods arrived that is ready, with the facts a test sets. */
function arrival(
  id: number,
  number: number,
  facts: Partial<Pick<ReadyGoodsRow, 'sell_date' | 'qty_left' | 'item'>> = {},
): ReadyGoodsRow {
  const garden = real.GOODS_TO_LOAD_GARDEN.rows[0];
  if (garden === undefined) throw new Error('the fixture has no garden row');
  return {
    ...structuredClone(garden),
    id,
    number,
    sell_date: TODAY,
    qty: 100,
    qty_left: 100,
    ...facts,
  };
}

/** The goods the fake backend has; each test sets it. */
let stock: ReadyGoodsRow[] = [];
/** How many lots of an earlier بکری have goods left (the "From باقی مال" button). */
let stockLots = 0;

/** Answers the two new reads the way the backend does: the counts, and the rows of one kind. */
function answerLikeTheBackend() {
  vi.mocked(bikriApi.bikriGoodsSources).mockImplementation(() =>
    Promise.resolve<GoodsSources>({
      stock_lots: stockLots,
      trip_rows: stock.filter((row) => row.kind === 'trip' && row.qty_left > 0).length,
      garden_rows: stock.filter((row) => row.kind === 'garden' && row.qty_left > 0).length,
    }),
  );
  // One page of the card at a time, with the count of the whole card (PRD FR-19.11, plan 80 R2-5).
  vi.mocked(bikriApi.bikriGoodsToLoad).mockImplementation((kind, offset, limit) => {
    const all = stock.filter((row) => row.kind === kind && row.qty_left > 0);
    return Promise.resolve({
      rows: all.slice(offset, offset + (limit === 0 ? 100 : limit)),
      total_count: all.length,
    });
  });
}

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  stock = [];
  stockLots = 0;
});

afterEach(() => {
  leaveAppWindow();
});

const pageText = () => visible(document.body.textContent);

/** The rows of a grid, counted once each (a row can be drawn in more than one part). */
const rowsIn = (grid: HTMLElement) =>
  new Set(Array.from(grid.querySelectorAll('.ag-row')).map((row) => row.getAttribute('row-index')))
    .size;

/**
 * The lots that have a quantity. A lot row that only waits to be typed in stays after the lots that were loaded
 * (like after a carried lot), and it may already carry the seller of the lot before it (PRD FR-7.51), so the count
 * of rows is one more than the count of lots on screen and the quantity is what tells a lot from that row.
 */
function filledLots(language: Language): number {
  const grid = lotsGrid(language);
  const count = rowsIn(grid);
  let filled = 0;
  for (let index = 0; index < count; index += 1) {
    if (textOf(grid, index, 'qty') !== '') filled += 1;
  }
  return filled;
}

/**
 * The backend's preview of the own lot of E2E-34, with the keys of the draft it is asked about (the
 * real preview echoes the key of each lot and line it was given).
 */
function fitted(draft: BikriDraft): BikriPreview {
  const preview = structuredClone(own.OWN_PREVIEW_E34);
  preview.lots.forEach((lot, lotIndex) => {
    lot.key = draft.lots[lotIndex]?.key ?? lot.key;
    lot.lines.forEach((line, lineIndex) => {
      line.key = draft.lots[lotIndex]?.sales[lineIndex]?.key ?? line.key;
    });
  });
  return preview;
}

/**
 * Opens the Sale Book on the fake backend of Event 1 with اپنا مال among the accounts. The fake of
 * the Sale Book sets its own accounts, so these are set after it and before the screen opens.
 */
async function openWithOwnGoods(language: Language, answer?: (draft: BikriDraft) => BikriPreview) {
  const backend = fakeBikri(answer);
  const everyone = [RAHIM, KARIM, AKRAM, ARSHAD, CHUKTA_KHATA, OWN_GOODS];
  fakeAccounts(everyone);
  fakeAccountSearch(everyone);
  const app = await renderApp('/bikri/book');
  await screen.findByRole('heading', { level: 1, name: message(language, 'glossary-bikri-book') });
  return { ...app, backend, user: userEvent.setup() };
}

/** The backend's real preview of a garden's lot with its arrival, with the keys of the draft. */
function fittedGarden(draft: BikriDraft): BikriPreview {
  const preview = structuredClone(arrivals.GARDEN_LOT_PREVIEW);
  preview.lots.forEach((lot, lotIndex) => {
    lot.key = draft.lots[lotIndex]?.key ?? lot.key;
    lot.lines.forEach((line, lineIndex) => {
      line.key = draft.lots[lotIndex]?.sales[lineIndex]?.key ?? line.key;
    });
  });
  return preview;
}

describe.each(LANGUAGES)('the three load buttons of a new بکری (%s, PRD FR-12.50)', (language) => {
  const ownName = language === 'ur' ? OWN_GOODS.name_ur : OWN_GOODS.name_en;
  const gardenName = language === 'ur' ? 'شاہ پور باغ' : 'Shahpur Garden';
  const orange = language === 'ur' ? 'مالٹا' : 'Orange';

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeTrips();
    answerLikeTheBackend();
  });

  const button = (id: string) =>
    screen.queryByRole('button', { name: named(message(language, id)) });
  const tripButton = () => button('bikri-goods-trip-add');
  const gardenButton = () => button('bikri-goods-garden-add');
  const stockButton = () => button('bikri-carried-add');

  /** The buttons are shown a moment after the counts arrive. */
  const buttonsAsked = () =>
    waitFor(() => {
      expect(bikriApi.bikriGoodsSources).toHaveBeenCalled();
    });

  /** Opens a card and clicks the row that holds `text`, then Enter: the row becomes a lot. */
  async function pickRow(
    user: Awaited<ReturnType<typeof openWithOwnGoods>>['user'],
    id: string,
    text: string,
  ) {
    const opener = await waitFor(() => {
      const found = button(id);
      if (found === null) throw new Error(`no button ${id}`);
      return found;
    });
    await user.click(opener);
    const dialog = await screen.findByRole('dialog');
    const cells = await within(dialog).findAllByRole('gridcell');
    const cell = cells.find((candidate) => visible(candidate.textContent).includes(text));
    if (cell === undefined) throw new Error(`no row with ${text}`);
    await user.click(cell);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  }

  it('R1: no goods anywhere, so no load button, and one empty lot as before', async () => {
    await openWithOwnGoods(language, fitted);
    await buttonsAsked();
    expect(tripButton()).toBeNull();
    expect(gardenButton()).toBeNull();
    expect(stockButton()).toBeNull();
    expect(rowsIn(lotsGrid(language))).toBe(1);
    expect(textOf(lotsGrid(language), 0, 'seller')).toBe('');
  });

  it('R2: a trip purchase with goods left shows its button, and nothing comes in by itself', async () => {
    stock = [purchase(1, 1)];
    await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(tripButton()).not.toBeNull();
    });
    // Only the button of the kind that has goods.
    expect(gardenButton()).toBeNull();
    expect(stockButton()).toBeNull();
    // The goods of the day are not a lot until the munshi loads them (the ready lots of plan 33 are gone).
    expect(rowsIn(lotsGrid(language))).toBe(1);
    expect(textOf(lotsGrid(language), 0, 'seller')).toBe('');
  });

  it('R3: a garden arrival shows the garden button only, and every kind with goods shows its own', async () => {
    stock = [arrival(1, 1)];
    stockLots = 2;
    await openWithOwnGoods(language, fittedGarden);
    await waitFor(() => {
      expect(gardenButton()).not.toBeNull();
    });
    expect(stockButton()).not.toBeNull();
    expect(tripButton()).toBeNull();
  });

  it('R4: the trip card lists the purchases; a pick makes a lot of اپنا مال with the item, the packing and the goods left, and the cursor goes to the buyer', async () => {
    stock = [purchase(1, 1), arrival(1, 1)];
    const { user } = await openWithOwnGoods(language, fitted);
    await pickRow(user, 'bikri-goods-trip-add', orange);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(ownName);
    });
    const lots = lotsGrid(language);
    expect(textOf(lots, 0, 'item')).toBe(orange);
    expect(textOf(lots, 0, 'qty')).toBe('2');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:buyer');
    });
    // The loaded purchase is not offered again; the card says there is nothing left to load.
    await user.click(
      await waitFor(() => {
        const found = tripButton();
        if (found === null) throw new Error('no trip button');
        return found;
      }),
    );
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText(
      (text) => visible(text) === message(language, 'bikri-goods-empty'),
    );
    // Esc closes the card and changes nothing.
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(filledLots(language)).toBe(1);
  });

  it('R5: the garden card makes a lot sold by the garden; the other kinds are not touched', async () => {
    stock = [purchase(1, 1), arrival(1, 1)];
    const { user } = await openWithOwnGoods(language, fittedGarden);
    await pickRow(user, 'bikri-goods-garden-add', gardenName);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(gardenName);
    });
    expect(textOf(lotsGrid(language), 0, 'qty')).toBe('100');
    // The trip purchase is still there to load.
    expect(tripButton()).not.toBeNull();
    expect(filledLots(language)).toBe(1);
  });

  it('R6: a trip lot and a garden lot on one screen are two lots, in the order they were loaded', async () => {
    stock = [purchase(1, 1), arrival(1, 1)];
    const { user } = await openWithOwnGoods(language, fittedGarden);
    await pickRow(user, 'bikri-goods-garden-add', gardenName);
    await pickRow(user, 'bikri-goods-trip-add', orange);
    await waitFor(() => {
      expect(filledLots(language)).toBe(2);
    });
    expect(textOf(lotsGrid(language), 0, 'seller')).toBe(gardenName);
    expect(textOf(lotsGrid(language), 1, 'seller')).toBe(ownName);
  });

  it('R7: the item and the packing of a loaded garden lot follow its entry; they cannot be typed over', async () => {
    stock = [arrival(1, 1)];
    const { user } = await openWithOwnGoods(language, fittedGarden);
    await pickRow(user, 'bikri-goods-garden-add', gardenName);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(gardenName);
    });
    // The cursor goes to the buyer of the lot after the pick: wait for it, or it would take the focus back.
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:buyer');
    });
    const cell = lotsGrid(language).querySelector<HTMLElement>(
      '.ag-row[row-index="0"] [col-id="item"]',
    );
    if (cell === null) throw new Error('no item cell');
    await user.click(cell);
    await user.keyboard('x');
    expect(document.activeElement?.tagName).not.toBe('INPUT');
  });

  it('R8: after a language switch the loaded lot and its line stay; F10 sends the purchase, the seller and the quantity', async () => {
    stock = [purchase(1, 1)];
    const { user, backend } = await openWithOwnGoods(language, fitted);
    await pickRow(user, 'bikri-goods-trip-add', orange);
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:buyer');
    });
    await user.keyboard('0{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
    });
    await user.keyboard('2{Enter}600{Enter}');
    await waitFor(() => {
      expect(pageText()).toContain('8,460');
    });
    press('KeyL', { altKey: true });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await waitFor(() => {
      expect(textOf(lotsGrid(other), 0, 'seller')).toBe(
        other === 'ur' ? OWN_GOODS.name_ur : OWN_GOODS.name_en,
      );
    });
    expect(textOf(lotsGrid(other), 0, 'qty')).toBe('2');
    expect(textOf(salesGrid(other), 0, 'rate')).toBe('600');
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    expect(backend.saves[0]?.draft.lots).toHaveLength(1);
    const sent = backend.saves[0]?.draft.lots[0];
    expect(sent?.seller_account_id).toBe(OWN_GOODS.id);
    expect(sent?.purchase_id).toBe(1);
    expect(sent?.qty_received).toBe('2');
    expect(sent?.cash_paid).toBeNull();
    expect(sent?.adjust_advance).toBe(false);
  });

  it('R9: a garden lot is sent with the garden as the seller and the link to its entry', async () => {
    stock = [arrival(1, 1)];
    const { user, backend } = await openWithOwnGoods(language, fittedGarden);
    await pickRow(user, 'bikri-goods-garden-add', gardenName);
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:buyer');
    });
    await user.keyboard('0{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
    });
    await user.keyboard('100{Enter}600{Enter}');
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    // The one empty lot row that waits to be typed in is not a lot: only the loaded lot is sent.
    expect(backend.saves[0]?.draft.lots).toHaveLength(1);
    const sent = backend.saves[0]?.draft.lots[0];
    expect(sent?.seller_account_id).toBe(35);
    expect(sent?.arrival_id).toBe(1);
    expect(sent?.purchase_id ?? null).toBeNull();
    expect(sent?.qty_received).toBe('100');
    expect(sent?.adjust_advance).toBe(false);
  });

  it('R10: after a save the counts are read again, and goods that were sold are not offered', async () => {
    stock = [purchase(1, 1)];
    const { user, backend } = await openWithOwnGoods(language, fitted);
    await pickRow(user, 'bikri-goods-trip-add', orange);
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:buyer');
    });
    await user.keyboard('0{Enter}2{Enter}600{Enter}');
    await waitFor(() => {
      expect(pageText()).toContain('8,460');
    });
    const before = vi.mocked(bikriApi.bikriGoodsSources).mock.calls.length;
    // The sale takes the goods: the backend has none left when the screen asks again.
    stock = [purchase(1, 1, { qty_left: 0 })];
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(vi.mocked(bikriApi.bikriGoodsSources).mock.calls.length).toBeGreaterThan(before);
    });
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe('');
    });
    expect(tripButton()).toBeNull();
    expect(rowsIn(lotsGrid(language))).toBe(1);
  });

  it('R12: the backend’s own answers: three buttons, and a pick of each card makes its lot with the goods left it gave', async () => {
    stock = [...real.GOODS_TO_LOAD_TRIP.rows, ...real.GOODS_TO_LOAD_GARDEN.rows];
    stockLots = real.GOODS_SOURCES.stock_lots;
    const tripLeft = real.GOODS_TO_LOAD_TRIP.rows[0]?.qty_left;
    const gardenLeft = real.GOODS_TO_LOAD_GARDEN.rows[0]?.qty_left;
    if (tripLeft === undefined || gardenLeft === undefined)
      throw new Error('the fixtures have no row');
    const { user } = await openWithOwnGoods(language, fittedGarden);
    await waitFor(() => {
      expect(tripButton()).not.toBeNull();
    });
    expect(gardenButton()).not.toBeNull();
    expect(stockButton()).not.toBeNull();
    await pickRow(user, 'bikri-goods-trip-add', orange);
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:buyer');
    });
    expect(textOf(lotsGrid(language), 0, 'seller')).toBe(ownName);
    expect(textOf(lotsGrid(language), 0, 'qty')).toBe(String(tripLeft));
    await pickRow(user, 'bikri-goods-garden-add', gardenName);
    await waitFor(() => {
      expect(filledLots(language)).toBe(2);
    });
    expect(textOf(lotsGrid(language), 1, 'seller')).toBe(gardenName);
    expect(textOf(lotsGrid(language), 1, 'qty')).toBe(String(gardenLeft));
  });

  it('R13: a card of 130 rows is read a page at a time, with its pager, so no row is out of reach (plan 80 R2-5)', async () => {
    // The backend gave at most 100 rows before; a trip with 130 purchases with goods left lost the last 30.
    stock = Array.from({ length: 130 }, (_, index) => purchase(index + 1, index + 1));
    const { user } = await openWithOwnGoods(language, fitted);
    const opener = await waitFor(() => {
      const found = tripButton();
      if (found === null) throw new Error('no trip button');
      return found;
    });
    await user.click(opener);
    const dialog = await screen.findByRole('dialog');
    // The first page, 25 rows, with the count of the whole card.
    await waitFor(() => {
      expect(bikriApi.bikriGoodsToLoad).toHaveBeenCalledWith('trip', 0, 25);
    });
    await within(dialog).findByText(
      paragraph(message(language, 'common-pager-range', { from: 1, to: 25, total: 130 })),
    );
    // The next page asks for the next 25 rows.
    await user.click(
      within(dialog).getByRole('button', { name: named(message(language, 'common-pager-next')) }),
    );
    await waitFor(() => {
      expect(bikriApi.bikriGoodsToLoad).toHaveBeenCalledWith('trip', 25, 25);
    });
    // The last page holds the last 5 rows: the ones the old card could not reach.
    await user.click(
      within(dialog).getByRole('button', { name: named(message(language, 'common-pager-last')) }),
    );
    await waitFor(() => {
      expect(bikriApi.bikriGoodsToLoad).toHaveBeenCalledWith('trip', 125, 25);
    });
    await within(dialog).findByText(
      paragraph(message(language, 'common-pager-range', { from: 126, to: 130, total: 130 })),
    );
    // A row of the last page becomes a lot, as on the first page.
    const cells = await within(dialog).findAllByRole('gridcell');
    const cell = cells[0];
    if (cell === undefined) throw new Error('no row on the last page');
    await user.click(cell);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(ownName);
    });
  });

  it('R11: a card that cannot be read says so and can be tried again', async () => {
    stock = [purchase(1, 1)];
    const { user } = await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(tripButton()).not.toBeNull();
    });
    vi.mocked(bikriApi.bikriGoodsToLoad).mockRejectedValueOnce(fail({ code: 'ERR_INTERNAL' }));
    const opener = tripButton();
    if (opener === null) throw new Error('no trip button');
    await user.click(opener);
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText(
      (text) => visible(text) === message(language, 'bikri-goods-failed'),
    );
  });
});

describe.each(LANGUAGES)(
  'a saved بکری and an edit have no load buttons; a carried lot does not stop them on a new one (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
      fakeTrips();
      const everyone = [RAHIM, KARIM, AKRAM, ARSHAD, CHUKTA_KHATA, OWN_GOODS];
      fakeAccounts(everyone);
      fakeAccountSearch(everyone);
      stock = [purchase(1, 1), arrival(1, 1)];
      stockLots = 1;
      answerLikeTheBackend();
    });

    const heading = (id: string, vars: Record<string, number> = {}) =>
      screen.findByRole('heading', { level: 1, name: named(message(language, id, vars)) });
    const anyButton = () =>
      ['bikri-goods-trip-add', 'bikri-goods-garden-add'].some(
        (id) => screen.queryByRole('button', { name: named(message(language, id)) }) !== null,
      );

    it('a saved بکری shows its own lots only, and no goods are asked for', async () => {
      fakeBikri();
      await renderApp('/bikri/book?id=41');
      await heading('bikri-saved-title', { number: 1 });
      await waitFor(() => {
        expect(pageText()).toContain('LES-1234');
      });
      expect(anyButton()).toBe(false);
      expect(bikriApi.bikriGoodsSources).not.toHaveBeenCalled();
    });

    it('an edit shows its own lots only', async () => {
      fakeBikri();
      fakePlan16();
      await renderApp('/bikri/book?id=41&edit=1');
      await heading('bikri-edit-title', { number: 1 });
      await waitFor(() => {
        expect(rowsIn(lotsGrid(language))).toBe(1);
      });
      expect(anyButton()).toBe(false);
      expect(bikriApi.bikriGoodsSources).not.toHaveBeenCalled();
    });

    it('a lot carried over from a later sale is on the new بکری first, and the buttons of trip and garden goods are there beside it', async () => {
      fakeBikri();
      fakePlan16();
      // The plan 16 fake answers the goods reads for its own lots only: ours come after it.
      answerLikeTheBackend();
      const everyone = [RAHIM, KARIM, AKRAM, ARSHAD, CHUKTA_KHATA, OWN_GOODS];
      fakeAccounts(everyone);
      fakeAccountSearch(everyone);
      await renderApp('/bikri/later?lot=40');
      await waitFor(() => {
        expect(textOf(lotsGrid(language), 0, 'seller')).toBe(language === 'ur' ? 'رحیم' : 'Rahim');
      });
      // Nothing comes in by itself: one carried lot, and the buttons for the goods that wait.
      await waitFor(() => {
        expect(anyButton()).toBe(true);
      });
      expect(filledLots(language)).toBe(1);
    });
  },
);
