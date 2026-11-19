// A fill that lands while a buyer code is being typed (plan 14 finding, plan 12 PRD FR-7.52).
// After the lot is typed, the screen fills the first sale line's quantity from the backend's answer
// (what the lot has left), a moment later. A munshi types the buyer code straight away, so the fill
// can land in the middle of his typing, most of all on a slow computer where the answer is late.
// The grid must keep the cell he is typing in: if it closed the cell, the code he typed would be
// lost and half of it would go to the wrong place.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { previewFor } from '../helpers/bikri-data';
import { focusedIn, openBook, salesGrid, textOf, typeHeaderAndLot } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';

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

describe.each(LANGUAGES)('a fill while a buyer code is typed (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('keeps the cell he is typing in, and the whole code reaches the picker', async () => {
    // Every answer waits until the test lets it go, so the fill lands when the test says.
    let open = false;
    const waiting: (() => void)[] = [];
    const { user } = await openBook(language, (draft) =>
      open
        ? previewFor(draft)
        : new Promise((resolve) => {
            waiting.push(() => {
              resolve(previewFor(draft));
            });
          }),
    );
    await typeHeaderAndLot(user, language);
    const sales = salesGrid(language);
    // He starts the code at once: the first digit opens the cell, more digits follow.
    await user.keyboard('0');
    await waitFor(() => {
      expect(document.activeElement?.tagName).toBe('INPUT');
    });
    await user.keyboard('02');
    // The answer for the lot lands now, in the middle of the code, and the quantity is filled.
    open = true;
    waiting.splice(0).forEach((release) => {
      release();
    });
    await waitFor(() => {
      expect(textOf(sales, 0, 'qty')).toBe('50');
    });
    // The cell is still the one he is typing in, with what he typed so far, and typing goes on.
    expect(document.activeElement?.tagName).toBe('INPUT');
    expect((document.activeElement as HTMLInputElement).value).toBe('002');
    await user.keyboard('0');
    expect((document.activeElement as HTMLInputElement).value).toBe('0020');
    await screen.findAllByRole('option', undefined, { timeout: 8_000 });
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    });
    expect(bikriApi.bikriPreview).toHaveBeenCalled();
  });
});
