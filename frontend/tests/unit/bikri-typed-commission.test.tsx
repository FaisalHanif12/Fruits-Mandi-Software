// Plan 72 P13 (PRD FR-11.10): a commission or لاگا rate typed on a sale line must re-work the line at once,
// like any typed value. Umar's case: line 1 is 20 at 600 = 12,000; the seller commission is 6% by default
// (720). He opens the details of the line, types 4 in the seller commission, and the commission must
// become 480 (12,000 × 4 ÷ 100) in the grid and in the details. The real screen, the real grid, the real
// store, typed key by key. The fake backend answers like the real one for a typed percent (the services
// test `a_percent_typed_on_the_line_is_manual_and_the_split_follows`): method manual, source typed, the
// rate in basis points, the amount worked out from the base.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { answerWithTyped } from '../helpers/bikri-typed';
import {
  cellOf,
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
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

describe.each(LANGUAGES)('a commission rate typed on a line re-works it (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('12,000 at 4% gives 480 in the grid and in the details, at once', async () => {
    const { backend, user } = await openBook(language, answerWithTyped);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(textOf(sales, 0, 'sellerComm')).toBe('720');
    }, SLOW_MS);
    // The details are those of the line that has the cursor.
    await user.click(cellOf(sales, 0, 'qty'));
    const field = await screen.findByRole('textbox', {
      name: named(message(language, 'bikri-line-seller-comm')),
    });
    await user.click(field);
    await user.keyboard('4');
    // The typed rate is in the draft the backend is asked about, with its method.
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.seller_comm).toMatchObject({
        method: 'percent',
        value: '4',
      });
    }, SLOW_MS);
    // And the answer is drawn: the commission is 480 in the grid and in the details of the line.
    await waitFor(() => {
      expect(textOf(sales, 0, 'sellerComm')).toBe('480');
    }, SLOW_MS);
    const section = field.closest('section');
    if (section === null) throw new Error('no details of the line');
    expect(visible(section.textContent)).toContain('480');
  });
});
