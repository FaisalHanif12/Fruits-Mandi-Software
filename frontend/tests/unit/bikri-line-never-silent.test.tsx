// D8 Bug 1, real cause, second half (plan 61, PRD FR-7.52, FR-7.6): a line with something typed on
// it and no figures always says why in its row, whichever error the backend sent, and even when it
// sent none. Umar's third line held a buyer, a quantity and a rate and showed nothing: the one error
// the backend sent was an "empty" answer for a part of the line that has no cell (a walk-in buyer's
// name), held back until the save. The rule is by what the line is, not by the name of a code, so
// it holds when the code is renamed (plan 64). The real screen, the real grid, the real store.
// A blank row stays silent. A buyer typed in the cell and never chosen is told in words too.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import { usePendingRestore } from '../../src/features/bikri/draft-offer';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { parseSnapshot, serialiseDraft } from '../../src/features/bikri/draft-snapshot';
import { SALE } from '../../src/features/bikri/types';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fillUmarsLot, umarAnswer } from '../helpers/bikri-umar-data';
import { openUmarsBook, seedUmarsLot } from '../helpers/bikri-umar';
import { cellOf, focusedIn, salesGrid } from '../helpers/bikri-screen';
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

// The shared account of walk-in buyers, as a chosen record: Umar's third line held it.
const SHARED = {
  id: 9,
  code: '0030',
  name_ur: 'چکتا کھاتہ',
  name_en: 'Chukta Khata',
};

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

/** Makes the next screen the sale book opens as Umar's lot with one line: the shared account, 130 at 900. */
function seedSharedBuyerLine(): void {
  const stores = createBikriDraft('10-10-2026');
  fillUmarsLot(stores);
  const lot = stores.lots.getState().rows[0];
  if (lot === undefined) throw new Error('no lot row');
  const sales = stores.salesFor(lot.rowId);
  const first = sales.getState().rows[0];
  if (first === undefined) throw new Error('no line row');
  sales.getState().patchRow(first.rowId, {
    picked: { [SALE.buyer]: SHARED },
    typed: { qty: '130', qtyFilled: '1', rate: '900' },
  });
  const snapshot = parseSnapshot(serialiseDraft(stores));
  if (snapshot === null) throw new Error('the seeded draft is not a snapshot');
  usePendingRestore.getState().offer(snapshot);
}

/**
 * The answer of the backend for that line when it cannot be worked out: no figures, and one "empty"
 * answer (code ERR_REQUIRED, the plainest) for a part of the line that has no cell of its own. It
 * uses no code that plan 64 renames.
 */
function noFiguresAnswer(error: boolean) {
  return (draft: BikriDraft): BikriPreview => {
    const reply = umarAnswer(draft);
    const line = reply.lots[0]?.lines[0];
    if (line === undefined) return reply;
    if (draft.lots[0]?.sales[0]?.buyer_account_id == null) return reply;
    line.calc = null;
    const totals = reply.lots[0];
    if (totals !== undefined) totals.totals = null;
    reply.errors = error
      ? [{ code: 'ERR_REQUIRED', field: 'lots[0].sales[0].walkin.name', values: {} }]
      : [];
    return reply;
  };
}

const alertsText = () =>
  screen.queryAllByRole('alert').map((element) => visible(element.textContent));

describe.each(LANGUAGES)('a line with no figures says why (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('an "empty" answer for a part of the line with no cell is told in the row at once, not held back until the save', async () => {
    seedSharedBuyerLine();
    await openUmarsBook(language, noFiguresAnswer(true));
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(cellOf(sales, 0, 'buyer')).toBeInTheDocument();
    }, SLOW_MS);
    // The line has a buyer, a quantity and a rate, and shows no figures.
    await waitFor(() => {
      expect(alertsText().some((text) => text.includes(message(language, 'err-required')))).toBe(
        true,
      );
    }, SLOW_MS);
    // It names the line: "Line 1: ...".
    const note = alertsText().find((text) => text.includes(message(language, 'err-required')));
    expect(note).toContain(message(language, 'bikri-qty-note-line', { line: 1 }));
  });

  it('a line with no figures and no error at all still says so, in words', async () => {
    seedSharedBuyerLine();
    await openUmarsBook(language, noFiguresAnswer(false));
    await waitFor(() => {
      expect(
        alertsText().some((text) => text.includes(message(language, 'bikri-line-no-figures'))),
      ).toBe(true);
    }, SLOW_MS);
  });

  it('a blank row has no note at all', async () => {
    seedUmarsLot();
    await openUmarsBook(language);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(cellOf(sales, 0, 'buyer')).toBeInTheDocument();
    }, SLOW_MS);
    await new Promise((done) => setTimeout(done, 400));
    expect(
      alertsText().filter((text) =>
        text.includes(message(language, 'bikri-qty-note-line', { line: 1 })),
      ),
    ).toEqual([]);
  });

  it('a buyer typed in the cell and never chosen is told in words, and the note goes when he is chosen', async () => {
    seedUmarsLot();
    const { user } = await openUmarsBook(language);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(cellOf(sales, 0, 'buyer')).toBeInTheDocument();
    }, SLOW_MS);
    await user.click(cellOf(sales, 0, 'buyer'));
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:buyer');
    }, SLOW_MS);
    // Typed, then the cell is left with Tab before anything is chosen.
    await user.keyboard('zzzz{Tab}');
    await waitFor(() => {
      expect(focusedIn(sales)).not.toBe('0:buyer');
    }, SLOW_MS);
    await waitFor(() => {
      expect(
        alertsText().some((text) =>
          text.includes(message(language, 'bikri-line-buyer-left', { text: 'zzzz' })),
        ),
      ).toBe(true);
    }, SLOW_MS);
    // The user goes back to the cell and chooses a buyer by his code: the note goes.
    await user.click(cellOf(sales, 0, 'buyer'));
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:buyer');
    }, SLOW_MS);
    await user.keyboard('0');
    await screen.findAllByRole('option', undefined, SLOW_MS);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
    await waitFor(() => {
      expect(
        alertsText().some((text) =>
          text.includes(message(language, 'bikri-line-buyer-left', { text: 'zzzz' })),
        ),
      ).toBe(false);
    }, SLOW_MS);
  });
});
