// Plan 33, PRD FR-12.39: a new بکری opens with the stock that is ready to sell already in it as lots.
// Every purchase of an open trip with goods left whose day is on or before the new بکری's day is a
// lot (its date to sell starts as its day): the seller is اپنا مال, the purchase is chosen, the item
// and packing are the purchase's and the quantity is the goods left. Goods not sold come again the
// next day, oldest day first. The lots follow the date until the first line is typed, the user can
// remove one, and nothing is added to a saved بکری, a later sale or an edit.
//
// Both languages, against the fake backend of Event 1. The purchases are the backend's real row
// (E2E-34), with the day, the number and the goods left set for each test. The numbers on the
// screen after a line is typed are the backend's real preview of E2E-34.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { ReadyGoodsRow } from '../../src/bindings/ReadyGoodsRow';
import * as bikriApi from '../../src/features/bikri/api';
import * as purchasesApi from '../../src/features/purchases/api';
import * as tripsApi from '../../src/features/trips/api';
import * as draftsApi from '../../src/shared/drafts/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  AKRAM,
  ARSHAD,
  KARIM,
  PUKHTA_BUYER,
  PUKHTA_SELLER,
  RAHIM,
  fakeBikri,
} from '../helpers/bikri';
import * as arrivals from '../helpers/arrivals-real';
import * as own from '../helpers/bikri-own-real';
import { fakePlan16 } from '../helpers/bikri-plan16';
import { cellOf, focusedIn, lotsGrid, named, salesGrid, textOf } from '../helpers/bikri-screen';
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
import { message, visible } from '../helpers/render';
import { fakeTrips } from '../helpers/trips';
import * as trips from '../helpers/trips-real';

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

const OWN_GOODS = account(17, 'seller', '030119998', 'اپنا مال', 'Own Goods (Trips)');

/** The day of the fake backend (helpers/app.tsx): the new بکری opens on it. */
const TODAY = '2026-10-10';
const YESTERDAY = '2026-10-09';
const TOMORROW = '2026-10-11';

const BANANA = { id: 6, code: '0045', name_ur: 'کیلا', name_en: 'Banana' };

/** The backend's real row of a trip purchase that is ready (bikri_ready_goods), with the facts a test sets. */
function purchase(
  id: number,
  number: number,
  facts: Partial<Pick<ReadyGoodsRow, 'sell_date' | 'qty_left' | 'item'>> = {},
): ReadyGoodsRow {
  const trip = arrivals.READY_GOODS.rows.find((row) => row.kind === 'trip');
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
  const garden = arrivals.READY_GOODS.rows.find((row) => row.kind === 'garden');
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

/** The stock the fake backend has; each test sets it. */
let stock: ReadyGoodsRow[] = [];

/** The day of a `DD-MM-YYYY` text, `YYYY-MM-DD`. */
const isoOf = (text: string) => text.split('-').reverse().join('-');

/** Answers the list the way the backend does: the goods left whose day to sell is not after the day. */
function answerLikeTheBackend() {
  vi.mocked(bikriApi.bikriReadyGoods).mockImplementation((date) =>
    Promise.resolve({
      rows: stock.filter((row) => row.sell_date <= isoOf(date) && row.qty_left > 0),
    }),
  );
}

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  stock = [];
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
  const everyone = [RAHIM, KARIM, AKRAM, ARSHAD, PUKHTA_BUYER, PUKHTA_SELLER, OWN_GOODS];
  fakeAccounts(everyone);
  fakeAccountSearch(everyone);
  const app = await renderApp('/bikri/book');
  await screen.findByRole('heading', { level: 1, name: message(language, 'glossary-bikri-book') });
  return { ...app, backend, user: userEvent.setup() };
}

describe.each(LANGUAGES)("today's trip goods in a new بکری (%s)", (language: Language) => {
  const ownName = language === 'ur' ? OWN_GOODS.name_ur : OWN_GOODS.name_en;
  const orange = language === 'ur' ? 'مالٹا' : 'Orange';
  const banana = language === 'ur' ? BANANA.name_ur : BANANA.name_en;
  const box = language === 'ur' ? 'پیٹی' : 'Box';
  const note = (count: number) => visible(message(language, 'bikri-ready-lots-note', { count }));

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeTrips();
    vi.mocked(tripsApi.tripsList).mockResolvedValue(trips.TRIP_LIST_ALL);
    vi.mocked(purchasesApi.purchasesGet).mockResolvedValue(trips.PURCHASE_GET_AFTER_SALES);
    answerLikeTheBackend();
  });

  /** The date box of the header: typed over, the way a person changes the day. */
  async function changeDateTo(
    user: Awaited<ReturnType<typeof openWithOwnGoods>>['user'],
    date: string,
  ) {
    const box = await screen.findByLabelText<HTMLInputElement>(
      named(message(language, 'bikri-header-date')),
    );
    await user.click(box);
    await user.keyboard(`{Control>}a{/Control}${date}`);
    await waitFor(() => {
      expect(box.value).toBe(date);
    });
  }

  it('R1, R5: a purchase of today is a lot, with the seller, the item, the packing and the goods left, and a note says where it came from', async () => {
    stock = [purchase(1, 1)];
    await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(ownName);
    });
    const lots = lotsGrid(language);
    expect(textOf(lots, 0, 'item')).toBe(orange);
    expect(textOf(lots, 0, 'packing')).toBe(box);
    expect(textOf(lots, 0, 'qty')).toBe('2');
    expect(rowsIn(lots)).toBe(1);
    // The note above the lots, and the way to leave a lot out.
    expect(pageText()).toContain(note(1));
    expect(pageText()).toContain(
      visible(message(language, 'bikri-ready-lots-note-hint', { key: 'Ctrl+Delete' })),
    );
    // The backend was asked for the purchases up to the day of the بکری with goods left, in the
    // date form it reads.
    expect(bikriApi.bikriReadyGoods).toHaveBeenCalledWith('10-10-2026');
  });

  it('no purchase today: the sale opens as before, one empty lot and no note', async () => {
    stock = [];
    await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(bikriApi.bikriReadyGoods).toHaveBeenCalled();
    });
    const lots = lotsGrid(language);
    expect(rowsIn(lots)).toBe(1);
    expect(textOf(lots, 0, 'seller')).toBe('');
    expect(pageText()).not.toContain(
      visible(message(language, 'bikri-ready-lots-note-hint', { key: 'Ctrl+Delete' })),
    );
  });

  it('two purchases today are two lots, in purchase number order', async () => {
    stock = [purchase(2, 2, { item: BANANA, qty_left: 5 }), purchase(1, 1)];
    await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(rowsIn(lotsGrid(language))).toBe(2);
    });
    const lots = lotsGrid(language);
    expect(textOf(lots, 0, 'item')).toBe(orange);
    expect(textOf(lots, 0, 'qty')).toBe('2');
    expect(textOf(lots, 1, 'seller')).toBe(ownName);
    expect(textOf(lots, 1, 'item')).toBe(banana);
    expect(textOf(lots, 1, 'qty')).toBe('5');
    expect(pageText()).toContain(note(2));
  });

  it('R4: goods to sell on a later day or with no goods left are not lots, even if the backend sends them', async () => {
    const wrong = [
      purchase(1, 1, { sell_date: TOMORROW }),
      purchase(2, 2, { qty_left: 0 }),
      arrival(3, 1, { sell_date: TOMORROW }),
    ];
    vi.mocked(bikriApi.bikriReadyGoods).mockResolvedValue({ rows: wrong });
    await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(bikriApi.bikriReadyGoods).toHaveBeenCalled();
    });
    await waitFor(() => {
      expect(rowsIn(lotsGrid(language))).toBe(1);
    });
    expect(textOf(lotsGrid(language), 0, 'seller')).toBe('');
    expect(pageText()).not.toContain(note(1));
  });

  it('R4: a closed trip has no goods left, so its purchase is not asked for', async () => {
    // A trip closes by itself when nothing is left (PRD FR-12.33): its purchases have no goods left.
    stock = [purchase(1, 1, { qty_left: 0 })];
    await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(bikriApi.bikriReadyGoods).toHaveBeenCalled();
    });
    expect(textOf(lotsGrid(language), 0, 'seller')).toBe('');
  });

  it('R8: goods not sold come again the next day, oldest day first; while no line is typed the lots follow the date of the بکری', async () => {
    stock = [purchase(1, 1), purchase(2, 2, { sell_date: YESTERDAY, item: BANANA, qty_left: 7 })];
    const { user } = await openWithOwnGoods(language, fitted);
    // The purchase of yesterday still has goods left: it comes first, then today's.
    await waitFor(() => {
      expect(rowsIn(lotsGrid(language))).toBe(2);
    });
    expect(textOf(lotsGrid(language), 0, 'item')).toBe(banana);
    expect(textOf(lotsGrid(language), 0, 'qty')).toBe('7');
    expect(textOf(lotsGrid(language), 1, 'item')).toBe(orange);
    expect(textOf(lotsGrid(language), 1, 'qty')).toBe('2');
    expect(pageText()).toContain(note(2));
    // On the day before, today's purchase is not there yet.
    await changeDateTo(user, '09-10-2026');
    await waitFor(() => {
      expect(rowsIn(lotsGrid(language))).toBe(1);
    });
    expect(textOf(lotsGrid(language), 0, 'item')).toBe(banana);
    expect(pageText()).toContain(note(1));
    await changeDateTo(user, '10-10-2026');
    await waitFor(() => {
      expect(rowsIn(lotsGrid(language))).toBe(2);
    });
    expect(textOf(lotsGrid(language), 1, 'item')).toBe(orange);
    // A day before every purchase gives back one empty lot and no note.
    await changeDateTo(user, '08-10-2026');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe('');
    });
    expect(rowsIn(lotsGrid(language))).toBe(1);
    expect(pageText()).not.toContain(
      visible(message(language, 'bikri-ready-lots-note-hint', { key: 'Ctrl+Delete' })),
    );
  });

  it('R3: Ctrl+Delete removes a lot, and it is not put back in the same form, not even when the date goes away and comes back', async () => {
    stock = [purchase(1, 1), purchase(2, 2, { sell_date: YESTERDAY, item: BANANA, qty_left: 7 })];
    const { user } = await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(rowsIn(lotsGrid(language))).toBe(2);
    });
    // Row 0 is yesterday's banana. Remove it.
    await user.click(cellOf(lotsGrid(language), 0, 'qty'));
    await user.keyboard('{Control>}{Delete}{/Control}');
    await waitFor(() => {
      expect(rowsIn(lotsGrid(language))).toBe(1);
    });
    expect(textOf(lotsGrid(language), 0, 'item')).toBe(orange);
    expect(pageText()).toContain(note(1));
    // The day before: today's purchase is not there, and the removed one is not put back.
    await changeDateTo(user, '09-10-2026');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe('');
    });
    expect(rowsIn(lotsGrid(language))).toBe(1);
    expect(pageText()).not.toContain(note(1));
    // Back to today: today's purchase comes, the removed banana still does not.
    await changeDateTo(user, '10-10-2026');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'item')).toBe(orange);
    });
    expect(rowsIn(lotsGrid(language))).toBe(1);
    expect(pageText()).toContain(note(1));
  });

  it('R3: a form that holds only these lots is not unsaved work: leaving asks nothing, and a typed line makes it work', async () => {
    stock = [purchase(1, 1)];
    const { router, user } = await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(ownName);
    });
    // No draft goes to the backend for it.
    expect(draftsApi.draftsSave).not.toHaveBeenCalled();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'layout-nav-dashboard')) }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
  });

  it('R3: once a buyer is typed on a trip lot, leaving asks "save changes?"', async () => {
    stock = [purchase(1, 1)];
    const { router, user } = await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(ownName);
    });
    await user.click(await waitFor(() => cellOf(salesGrid(language), 0, 'buyer')));
    await user.keyboard('0');
    await screen.findAllByRole('option');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
    });
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'layout-nav-dashboard')) }),
    );
    expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
    expect(router.state.location.pathname).toBe('/bikri/book');
  });

  it('a lot that has a line does not follow the date any more', async () => {
    stock = [purchase(1, 1), purchase(2, 2, { sell_date: YESTERDAY, item: BANANA, qty_left: 7 })];
    const { user } = await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(rowsIn(lotsGrid(language))).toBe(2);
    });
    await user.click(await waitFor(() => cellOf(salesGrid(language), 0, 'buyer')));
    await user.keyboard('0');
    await screen.findAllByRole('option');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
    });
    await changeDateTo(user, '09-10-2026');
    // Give a wrong replacement time to happen; the lots stay as he left them.
    await waitFor(() => {
      expect(bikriApi.bikriReadyGoods).toHaveBeenCalledWith('09-10-2026');
    });
    expect(rowsIn(lotsGrid(language))).toBe(2);
    expect(textOf(lotsGrid(language), 0, 'item')).toBe(banana);
    expect(textOf(lotsGrid(language), 1, 'item')).toBe(orange);
  });

  it('the keyboard goes to the buyer of the first lot after the header; a language switch loses nothing; F10 sends the purchase, the seller and the quantity', async () => {
    stock = [purchase(1, 1)];
    const { user, backend } = await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(ownName);
    });
    // The truck fills the اڈا and the زمیندار; Enter goes through them and lands on the first buyer.
    await user.keyboard('LES-1234{Enter}');
    await waitFor(() => {
      expect(
        screen.getByRole('combobox', { name: named(message(language, 'glossary-zamindar')) }),
      ).toHaveValue(language === 'ur' ? 'رحیم' : 'Rahim');
    });
    await user.keyboard('{Enter}{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:buyer');
    });
    // The زمیندار did not take the trip lot: it is still اپنا مال and still has its goods.
    expect(textOf(lotsGrid(language), 0, 'seller')).toBe(ownName);
    expect(textOf(lotsGrid(language), 0, 'qty')).toBe('2');
    // Akram by his short code; the line starts with the goods left; the rate is typed.
    await user.keyboard('0{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
    });
    await user.keyboard('2{Enter}600{Enter}');
    await waitFor(() => {
      expect(pageText()).toContain('8,460');
    });

    // A language switch in the middle: the lot, its note and the line stay.
    press('KeyL', { altKey: true });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await waitFor(() => {
      expect(textOf(lotsGrid(other), 0, 'seller')).toBe(
        other === 'ur' ? OWN_GOODS.name_ur : OWN_GOODS.name_en,
      );
    });
    expect(textOf(lotsGrid(other), 0, 'qty')).toBe('2');
    expect(textOf(salesGrid(other), 0, 'rate')).toBe('600');
    expect(pageText()).toContain(
      visible(message(other, 'bikri-ready-lots-note', { count: 1, today: 1 })),
    );
    expect(rowsIn(lotsGrid(other))).toBe(1);

    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    const sent = backend.saves[0]?.draft.lots[0];
    expect(sent?.seller_account_id).toBe(OWN_GOODS.id);
    expect(sent?.purchase_id).toBe(1);
    expect(sent?.qty_received).toBe('2');
    expect(sent?.cash_paid).toBeNull();
    expect(sent?.adjust_advance).toBe(false);
    expect(sent?.sales[0]).toMatchObject({
      buyer_account_id: AKRAM.id,
      qty: '2',
      seller_rate: '600',
    });
  });

  it('after a save the next new بکری asks for the purchases again, and a purchase that was sold is not offered', async () => {
    stock = [purchase(1, 1)];
    const { user, backend } = await openWithOwnGoods(language, fitted);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(ownName);
    });
    await user.click(await waitFor(() => cellOf(salesGrid(language), 0, 'buyer')));
    await user.keyboard('0');
    await screen.findAllByRole('option');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
    });
    await user.keyboard('2{Enter}600{Enter}');
    await waitFor(() => {
      expect(pageText()).toContain('8,460');
    });
    // The sale takes the goods: the backend has none left when the screen asks again.
    stock = [purchase(1, 1, { qty_left: 0 })];
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe('');
    });
    expect(rowsIn(lotsGrid(language))).toBe(1);
    expect(pageText()).not.toContain(note(1));
  });
});

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

describe.each(LANGUAGES)(
  "a garden's goods arrived in a new بکری (%s, PRD FR-12.40)",
  (language) => {
    const gardenName = language === 'ur' ? 'شاہ پور باغ' : 'Shahpur Garden';
    const note = (count: number) => visible(message(language, 'bikri-ready-lots-note', { count }));

    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
      fakeTrips();
      vi.mocked(purchasesApi.purchasesGet).mockResolvedValue(trips.PURCHASE_GET_AFTER_SALES);
      answerLikeTheBackend();
    });

    it('an entry that is ready is a lot: the garden is the seller, the item, the packing and the goods left come from it, and a panel says where they come from', async () => {
      stock = [arrival(1, 1)];
      await openWithOwnGoods(language, fittedGarden);
      await waitFor(() => {
        expect(textOf(lotsGrid(language), 0, 'seller')).toBe(gardenName);
      });
      const lots = lotsGrid(language);
      expect(textOf(lots, 0, 'qty')).toBe('100');
      expect(rowsIn(lots)).toBe(1);
      expect(pageText()).toContain(note(1));
      // The panel: the garden by its code and name; the entry; what it has left. The seller is the
      // garden's account (code 0080), which is why the cash and the advance cells do not apply.
      const panel = await screen.findByText(
        (text) => visible(text) === message(language, 'bikri-arrival-title'),
      );
      expect(panel).toBeInTheDocument();
      expect(pageText()).toContain('0080');
    });

    it('both kinds come in together, oldest day first, a trip purchase before a garden entry of the same day', async () => {
      stock = [
        arrival(1, 1),
        purchase(1, 1),
        arrival(2, 2, { sell_date: YESTERDAY, item: BANANA, qty_left: 30 }),
      ];
      await openWithOwnGoods(language, fittedGarden);
      await waitFor(() => {
        expect(rowsIn(lotsGrid(language))).toBe(3);
      });
      const lots = lotsGrid(language);
      // Yesterday's garden entry first, then today's trip purchase, then today's garden entry.
      expect(textOf(lots, 0, 'seller')).toBe(gardenName);
      expect(textOf(lots, 0, 'qty')).toBe('30');
      expect(textOf(lots, 1, 'seller')).toBe(
        language === 'ur' ? OWN_GOODS.name_ur : OWN_GOODS.name_en,
      );
      expect(textOf(lots, 2, 'seller')).toBe(gardenName);
      expect(textOf(lots, 2, 'qty')).toBe('100');
      expect(pageText()).toContain(note(3));
    });

    it('goods that were sold in part come again with what is left', async () => {
      stock = [arrival(1, 1, { qty_left: 40 })];
      await openWithOwnGoods(language, fittedGarden);
      await waitFor(() => {
        expect(textOf(lotsGrid(language), 0, 'qty')).toBe('40');
      });
    });

    it('item and packing follow the entry: they cannot be typed on this lot', async () => {
      stock = [arrival(1, 1)];
      const { user } = await openWithOwnGoods(language, fittedGarden);
      await waitFor(() => {
        expect(textOf(lotsGrid(language), 0, 'seller')).toBe(gardenName);
      });
      await user.click(cellOf(lotsGrid(language), 0, 'item'));
      await user.keyboard('x');
      expect(document.activeElement?.tagName).not.toBe('INPUT');
    });

    it('F10 sends the garden as the seller and the link to the entry, with no cash and no advance; the buyer and the rate are typed', async () => {
      stock = [arrival(1, 1)];
      const { user, backend } = await openWithOwnGoods(language, fittedGarden);
      await waitFor(() => {
        expect(textOf(lotsGrid(language), 0, 'seller')).toBe(gardenName);
      });
      await user.click(await waitFor(() => cellOf(salesGrid(language), 0, 'buyer')));
      await user.keyboard('0');
      await screen.findAllByRole('option');
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(focusedIn(salesGrid(language))).toBe('0:qty');
      });
      await user.keyboard('100{Enter}600{Enter}');
      press('F10');
      await waitFor(() => {
        expect(backend.saves).toHaveLength(1);
      });
      const sent = backend.saves[0]?.draft.lots[0];
      expect(sent?.seller_account_id).toBe(31);
      expect(sent?.arrival_id).toBe(1);
      expect(sent?.purchase_id ?? null).toBeNull();
      expect(sent?.qty_received).toBe('100');
      expect(sent?.cash_paid).toBeNull();
      expect(sent?.adjust_advance).toBe(false);
      expect(sent?.sales[0]).toMatchObject({
        buyer_account_id: AKRAM.id,
        qty: '100',
        seller_rate: '600',
      });
    });

    it('a second sale of the same goods is refused with what is left, the words are at the quantity, and the ready goods are read again', async () => {
      stock = [arrival(1, 1)];
      const { user, backend } = await openWithOwnGoods(language, fittedGarden);
      await waitFor(() => {
        expect(textOf(lotsGrid(language), 0, 'seller')).toBe(gardenName);
      });
      await user.click(await waitFor(() => cellOf(salesGrid(language), 0, 'buyer')));
      await user.keyboard('0');
      await screen.findAllByRole('option');
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(focusedIn(salesGrid(language))).toBe('0:qty');
      });
      await user.keyboard('100{Enter}600{Enter}');
      const before = vi.mocked(bikriApi.bikriReadyGoods).mock.calls.length;
      vi.mocked(bikriApi.bikriSave).mockRejectedValueOnce(
        fail({
          code: 'ERR_QTY_EXCEEDS_LOT',
          field: 'lots[0].qty_received',
          values: { available: 40 },
        }),
      );
      press('F10');
      await waitFor(() => {
        expect(pageText()).toContain(message(language, 'err-qty-exceeds-lot', { available: 40 }));
      });
      // Nothing was saved, nothing typed is lost, and the goods are asked for again.
      expect(backend.saves).toHaveLength(0);
      expect(textOf(salesGrid(language), 0, 'rate')).toBe('600');
      await waitFor(() => {
        expect(vi.mocked(bikriApi.bikriReadyGoods).mock.calls.length).toBeGreaterThan(before);
      });
    });
  },
);

describe.each(LANGUAGES)(
  'a saved بکری, an edit and a later sale get no trip lots (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
      fakeTrips();
      const everyone = [RAHIM, KARIM, AKRAM, ARSHAD, PUKHTA_BUYER, PUKHTA_SELLER, OWN_GOODS];
      fakeAccounts(everyone);
      fakeAccountSearch(everyone);
      stock = [purchase(1, 1)];
      answerLikeTheBackend();
    });

    const heading = (id: string, vars: Record<string, number> = {}) =>
      screen.findByRole('heading', { level: 1, name: named(message(language, id, vars)) });

    it('a saved بکری shows its own lots only, and the purchases of the day are not asked for', async () => {
      fakeBikri();
      await renderApp('/bikri/book?id=41');
      await heading('bikri-saved-title', { number: 1 });
      await waitFor(() => {
        expect(pageText()).toContain('LES-1234');
      });
      // The lot of the saved بکری is Rahim's; اپنا مال is not among its lots.
      expect(pageText()).not.toContain(language === 'ur' ? OWN_GOODS.name_ur : OWN_GOODS.name_en);
      expect(pageText()).not.toContain(
        visible(message(language, 'bikri-ready-lots-note-hint', { key: 'Ctrl+Delete' })),
      );
      expect(bikriApi.bikriReadyGoods).not.toHaveBeenCalled();
    });

    it('an edit shows its own lots only', async () => {
      fakeBikri();
      fakePlan16();
      await renderApp('/bikri/book?id=41&edit=1');
      await heading('bikri-edit-title', { number: 1 });
      await waitFor(() => {
        expect(rowsIn(lotsGrid(language))).toBe(1);
      });
      expect(pageText()).not.toContain(
        visible(message(language, 'bikri-ready-lots-note-hint', { key: 'Ctrl+Delete' })),
      );
      expect(bikriApi.bikriReadyGoods).not.toHaveBeenCalled();
    });

    it('a later sale shows the lot it sells, and nothing is added', async () => {
      fakeBikri();
      fakePlan16();
      await renderApp('/bikri/later?lot=40');
      await heading('bikri-later-title', { number: 1 });
      expect(pageText()).not.toContain(language === 'ur' ? OWN_GOODS.name_ur : OWN_GOODS.name_en);
      expect(pageText()).not.toContain(
        visible(message(language, 'bikri-ready-lots-note-hint', { key: 'Ctrl+Delete' })),
      );
      expect(bikriApi.bikriReadyGoods).not.toHaveBeenCalled();
    });
  },
);
