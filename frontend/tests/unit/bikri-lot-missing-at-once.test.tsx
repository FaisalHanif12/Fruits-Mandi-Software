// A lot that cannot work out its lines says why at once (plan 85, PRD FR-12.6 v1.49, plan 79 item 4), in the Urdu
// and the English app. A lot with no packing (a new seller with no last lot), or no seller, item or quantity
// received, gives every line of it no figures. The backend's "this is empty" answers for the lot wait until the
// user tries to save (problems.ts), and the line note was dropped when the lot had an error of its own
// (grid-views.ts), so a line with a quantity and a rate showed nothing at all. Now the lot row says what is
// missing, red on its cell and in words under the lots grid, as soon as a line of that lot has a quantity and a
// rate. A lot with no typed line stays silent (only critical messages, plan 79). The real screen and store.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { ErrorCode } from '../../src/bindings/ErrorCode';
import { usePendingRestore } from '../../src/features/bikri/draft-offer';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { parseSnapshot, serialiseDraft } from '../../src/features/bikri/draft-snapshot';
import { LOT, SALE } from '../../src/features/bikri/types';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { MANI, fillUmarsLot, umarAnswer } from '../helpers/bikri-umar-data';
import { openUmarsBook } from '../helpers/bikri-umar';
import { cellOf, lotsGrid, salesGrid } from '../helpers/bikri-screen';
import { fakePartners } from '../helpers/partners';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

/** What is missing from the lot: the picked cell or typed text taken away, the backend's answer, the lot's column. */
interface Need {
  word: 'seller' | 'item' | 'packing' | 'qty';
  code: ErrorCode;
  field: string;
  column: string;
  /** Takes the part out of the lot row. */
  take: (stores: ReturnType<typeof createBikriDraft>, rowId: string) => void;
}

const NEEDS: readonly Need[] = [
  {
    word: 'packing',
    code: 'ERR_REQUIRED',
    field: 'lots[0].packing_id',
    column: 'packing',
    take: (stores, rowId) => {
      stores.lots.getState().patchRow(rowId, { picked: { [LOT.packing]: null } });
    },
  },
  {
    word: 'item',
    code: 'ERR_REQUIRED',
    field: 'lots[0].item_id',
    column: 'item',
    take: (stores, rowId) => {
      stores.lots.getState().patchRow(rowId, { picked: { [LOT.item]: null } });
    },
  },
  {
    word: 'seller',
    code: 'ERR_SELLER_REQUIRED',
    field: 'lots[0].seller_account_id',
    column: 'seller',
    take: (stores, rowId) => {
      stores.lots.getState().patchRow(rowId, { picked: { [LOT.seller]: null } });
    },
  },
  {
    word: 'qty',
    code: 'ERR_REQUIRED',
    field: 'lots[0].qty_received',
    column: 'qty',
    take: (stores, rowId) => {
      stores.lots.getState().patchRow(rowId, { typed: { [LOT.qty]: '' } });
    },
  },
];

/**
 * Makes the next sale book open with Umar's lot missing one part, and `lines` as the lines of the lot: مانی 130 at
 * 900 (a quantity and a rate), or none (every row blank).
 */
function seed(need: Need, withLine: boolean): void {
  const stores = createBikriDraft('10-10-2026');
  fillUmarsLot(stores);
  const lot = stores.lots.getState().rows[0];
  if (lot === undefined) throw new Error('no lot row');
  need.take(stores, lot.rowId);
  if (withLine) {
    const sales = stores.salesFor(lot.rowId);
    const first = sales.getState().rows[0];
    if (first === undefined) throw new Error('no line row');
    sales.getState().patchRow(first.rowId, {
      picked: { [SALE.buyer]: { id: MANI.id, code: MANI.code, name_ur: 'مانی', name_en: 'Mani' } },
      typed: { qty: '130', qtyFilled: '1', rate: '900' },
    });
  }
  const snapshot = parseSnapshot(serialiseDraft(stores));
  if (snapshot === null) throw new Error('the seeded draft is not a snapshot');
  usePendingRestore.getState().offer(snapshot);
}

/**
 * The backend's answer for a lot that misses one part: its own "this is empty" error for that part, and no
 * figures for any line, the lot or the sale (domain/src/bikri/process.rs: an incomplete lot works nothing out).
 * A row with no buyer and no rate is blank and is not touched.
 */
function answerFor(need: Need) {
  return (draft: BikriDraft): BikriPreview => {
    const reply = umarAnswer(draft);
    for (const lot of reply.lots) {
      lot.totals = null;
      for (const line of lot.lines) line.calc = null;
    }
    reply.errors = [{ code: need.code, field: need.field, values: {} }];
    return reply;
  };
}

const alertsText = () =>
  screen.queryAllByRole('alert').map((element) => visible(element.textContent));

describe.each(LANGUAGES)('a lot that cannot work out its lines says why (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it.each(NEEDS)(
    'a lot with no $word and a line with a quantity and a rate says what is missing at once, before any save',
    async (need) => {
      seed(need, true);
      await openUmarsBook(language, answerFor(need));
      const sales = salesGrid(language);
      await waitFor(() => {
        expect(cellOf(sales, 0, 'buyer')).toBeInTheDocument();
      }, SLOW_MS);
      const words = `${message(language, 'bikri-where-lot', { lot: 1 })} ${message(
        language,
        `bikri-lot-missing-${need.word}`,
      )}`;
      await waitFor(() => {
        expect(alertsText().some((text) => text.includes(words))).toBe(true);
      }, SLOW_MS);
      // The cell of that part is red too (colour is never the only signal).
      await waitFor(() => {
        expect(cellOf(lotsGrid(language), 0, need.column)).toHaveClass('cell-error');
      }, SLOW_MS);
    },
  );

  it('a lot with no packing and only blank rows says nothing: only critical messages (plan 79)', async () => {
    const need = NEEDS[0];
    if (need === undefined) throw new Error('no need');
    seed(need, false);
    await openUmarsBook(language, answerFor(need));
    const lots = lotsGrid(language);
    await waitFor(() => {
      expect(cellOf(lots, 0, 'seller')).toBeInTheDocument();
    }, SLOW_MS);
    // Let the answer arrive and be drawn before looking for silence.
    await waitFor(() => {
      expect(cellOf(lots, 0, 'packing')).toBeInTheDocument();
    }, SLOW_MS);
    await new Promise((resolve) => setTimeout(resolve, 400));
    expect(
      alertsText().some((text) => text.includes(message(language, 'bikri-lot-missing-packing'))),
    ).toBe(false);
    expect(cellOf(lots, 0, 'packing')).not.toHaveClass('cell-error');
  });
});
