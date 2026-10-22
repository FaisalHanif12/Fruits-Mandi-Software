// A lot of trip goods typed with the keyboard on the whole sale book (plan 16b, T4b, PRD FR-12.26,
// FR-7.14, FR-7.33, Code Standards F30 and F39), in the Urdu and the English app. The backend’s
// real preview of E2E-34 answers (with the keys of the draft it is asked about); the purchases are
// the backend’s real answers. The seller is اپنا مال, the purchase is chosen with the keys, Enter
// goes through the lot without stopping at the cash cell (it does not apply), the line is typed,
// and the panel shows the numbers of the backend. A language switch in the middle loses nothing,
// and the save sends the purchase, the seller and the quantities that were typed.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import * as bikriApi from '../../src/features/bikri/api';
import * as purchasesApi from '../../src/features/purchases/api';
import * as tripsApi from '../../src/features/trips/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import * as own from '../helpers/bikri-own-real';
import { AKRAM, ARSHAD, KARIM, PUKHTA_BUYER, PUKHTA_SELLER, RAHIM } from '../helpers/bikri';
import {
  cellOf,
  focusedIn,
  lotsGrid,
  named,
  openBook,
  salesGrid,
  textOf,
  typeCodeAndEnter,
} from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
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

/** The goods the real purchase has left: the lot takes them all, and the line sells them. */
const LEFT = String(trips.PURCHASE_GET_AFTER_SALES.qty_left);

const OWN_GOODS = account(17, 'seller', '030119998', 'اپنا مال', 'Own Goods (Trips)');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

const pageText = () => visible(document.body.textContent);

/**
 * The backend’s preview of the own lot of E2E-34, with the keys of the draft it is asked about (the
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

describe.each(LANGUAGES)('a lot of trip goods typed with the keyboard (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeTrips();
    vi.mocked(tripsApi.tripsList).mockResolvedValue(trips.TRIP_LIST_ALL);
    vi.mocked(purchasesApi.purchasesList).mockResolvedValue({
      ...trips.PURCHASES_LIST_ALL,
      rows: [trips.PURCHASE_GET_AFTER_SALES],
    });
    vi.mocked(purchasesApi.purchasesGet).mockResolvedValue(trips.PURCHASE_GET_AFTER_SALES);
  });

  async function openWithOwnGoods() {
    const view = await openBook(language, fitted);
    const everyone = [RAHIM, KARIM, AKRAM, ARSHAD, PUKHTA_BUYER, PUKHTA_SELLER, OWN_GOODS];
    fakeAccounts(everyone);
    fakeAccountSearch(everyone);
    return view;
  }

  /** Where Enter goes from the quantity of the lot until the first sale line. Cash is never a stop. */
  async function enterToTheLines(user: Awaited<ReturnType<typeof openWithOwnGoods>>['user']) {
    const visited: string[] = [];
    for (let step = 0; step < 14; step += 1) {
      if (focusedIn(salesGrid(language)) === '0:buyer') return visited;
      const here = focusedIn(lotsGrid(language));
      if (here !== null) visited.push(here);
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(
          focusedIn(lotsGrid(language)) !== here || focusedIn(salesGrid(language)) !== null,
        ).toBe(true);
      });
    }
    throw new Error(`Enter did not reach the sale lines; it visited ${visited.join(', ')}`);
  }

  it('the seller, the purchase, the quantity and one line, with the numbers of the backend; a language switch in the middle loses nothing; F10 sends what was typed', async () => {
    const { user, backend } = await openWithOwnGoods();
    // The seller of the lot: اپنا مال (the only seller field that offers it).
    const sellerCell = await waitFor(() => cellOf(lotsGrid(language), 0, 'seller'));
    await user.click(sellerCell);
    await typeCodeAndEnter(user, 'own');
    // Focus goes to the choice of the purchase, which is the next thing to do.
    const choice = await screen.findByRole('combobox', {
      name: named(message(language, 'bikri-own-purchase')),
    });
    await waitFor(() => {
      expect(choice).toHaveFocus();
    });
    await user.keyboard('{Enter}');
    await user.click(await screen.findByRole('option'));
    // The quantity is filled from the purchase (the real purchase has 2 left) and is the next stop.
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).toBe('0:qty');
    });
    expect(textOf(lotsGrid(language), 0, 'qty')).toBe(LEFT);
    // Enter walks through the lot to the first sale line and never stops at the cash cell.
    const visited = await enterToTheLines(user);
    expect(visited).not.toContain('0:cash');
    // One line: Akram by his short code, all of it at 600.
    await user.keyboard('1{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
    });
    await user.keyboard(`${LEFT}{Enter}600{Enter}`);
    // The panel shows the backend’s numbers: credit 8,460 → 3,384 / 2,538 / 2,538, cost 7,800, result 660.
    await waitFor(() => {
      const text = pageText();
      for (const figure of ['8,460', '3,384', '2,538', '7,800', '660']) {
        expect(text, figure).toContain(figure);
      }
    });

    // A language switch in the middle: the purchase, the quantity, the buyer and the numbers stay.
    press('KeyL', { altKey: true });
    const other = language === 'ur' ? 'en' : 'ur';
    await waitFor(() => {
      expect(pageText()).toContain(message(other, 'bikri-own-credit'));
    });
    await waitFor(() => {
      expect(textOf(lotsGrid(other), 0, 'qty')).toBe(LEFT);
      expect(textOf(salesGrid(other), 0, 'qty')).toBe(LEFT);
      expect(textOf(salesGrid(other), 0, 'rate')).toBe('600');
    });
    expect(pageText()).toContain(other === 'ur' ? AKRAM.name_ur : AKRAM.name_en);
    expect(pageText()).toContain(other === 'ur' ? OWN_GOODS.name_ur : OWN_GOODS.name_en);
    for (const figure of ['8,460', '3,384', '7,800', '660']) expect(pageText()).toContain(figure);

    // F10 sends the purchase, the seller and the quantities that were typed; no cash, no advance.
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    const sent = backend.saves[0]?.draft.lots[0];
    expect(sent?.seller_account_id).toBe(OWN_GOODS.id);
    expect(sent?.purchase_id).toBe(trips.PURCHASE_GET_AFTER_SALES.id);
    expect(sent?.qty_received).toBe(LEFT);
    expect(sent?.cash_paid).toBeNull();
    expect(sent?.adjust_advance).toBe(false);
    expect(sent?.sales[0]).toMatchObject({
      buyer_account_id: AKRAM.id,
      qty: LEFT,
      seller_rate: '600',
    });
    expect(bikriApi.bikriSave).toHaveBeenCalledTimes(1);
  });
});
