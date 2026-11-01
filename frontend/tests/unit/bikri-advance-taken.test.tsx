// The warning ERR_ADVANCE_TAKEN_FROM_LATER on a lot of the sale book (plan 47, PRD FR-13.11): a پختہ
// seller's own lot, with the advance tick on, is dated before another entry of his that holds part
// of his advance, so this lot takes 5,000 of it from the later entry and his balance will not be 0.
// The backend says so in the `warnings` of the preview, on `lots[0].cash_paid` (its own real answer,
// tests/fixtures/bikri_preview_pukhta_party_seller_advance_taken.json, copied and never edited). The
// screen shows it amber on the lot's نقد cell and in the list of problems, in the app language. The
// save is allowed, and the default نقد the backend filled stays what it was (5,160). Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import taken from '../fixtures/bikri_preview_pukhta_party_seller_advance_taken.json';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { previewFor } from '../helpers/bikri-data';
import {
  cellOf,
  focusedIn,
  lotsGrid,
  named,
  openBook,
  textOf,
  type User,
} from '../helpers/bikri-screen';
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

// The fixture file is the backend's own JSON, read as the generated type.
const TAKEN = taken as unknown as BikriPreview;

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  // The backend's files have no اڈا and no filled fields: the truck has no memory.
  vi.mocked(bikriApi.assistTruckMemory).mockResolvedValue({
    last: null,
    same_day: null,
    suggestions: [],
  });
});

afterEach(() => {
  leaveAppWindow();
});

/** The preview of Event 1 with the totals and the warning of the backend's real answer on the lot. */
function takenPreview(draft: BikriDraft): BikriPreview {
  const preview = previewFor(draft);
  const lot = preview.lots[0];
  const real = TAKEN.lots[0]?.totals;
  // Only while نقد is left empty does the backend fill it (and say so, FR-13.10).
  if (lot?.totals && real && draft.lots[0]?.cash_paid === null) {
    lot.totals.cash_paid = real.cash_paid;
    lot.totals.net = real.net;
    lot.totals.workings.cash_paid = real.workings.cash_paid;
  }
  return { ...preview, warnings: [...preview.warnings, ...TAKEN.warnings] };
}

/** The lot of Event 1, with نقد left empty (the 5,000 of Event 1 is not typed). */
async function typeLotWithoutCash(user: User, language: Language) {
  await user.keyboard('LES-1234{Enter}');
  await screen.findByRole('combobox', { name: named(message(language, 'glossary-zamindar')) });
  await waitFor(() => {
    expect(textOf(lotsGrid(language), 0, 'seller')).not.toBe('');
  });
  await user.keyboard('{Enter}{Enter}');
  await waitFor(() => {
    expect(focusedIn(lotsGrid(language))).toBe('0:seller');
  });
  await waitFor(() => {
    expect(textOf(lotsGrid(language), 0, 'packing')).not.toBe('');
  });
  await user.keyboard('{Enter}{Enter}{Enter}50{Enter}3000{Enter}5{Enter}50{Enter}{Enter}');
}

describe('the backend’s answer', () => {
  it('is a warning only, on the cash of the first lot, with the advance it takes in whole rupees', () => {
    expect(TAKEN.errors).toEqual([]);
    expect(TAKEN.warnings).toHaveLength(1);
    expect(TAKEN.warnings[0]).toMatchObject({
      code: 'ERR_ADVANCE_TAKEN_FROM_LATER',
      field: 'lots[0].cash_paid',
      values: { taken_paisa: 500000 },
    });
    // The default نقد is what it is without the warning: 10,160 less the 5,000 advance.
    expect(TAKEN.lots[0]?.totals?.cash_paid).toBe(516000);
  });
});

describe.each(LANGUAGES)('an advance taken from a later sale (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('shows the warning amber on the نقد cell and in the problems, and keeps the default نقد', async () => {
    const { backend, user } = await openBook(language, takenPreview);
    await typeLotWithoutCash(user, language);
    await waitFor(() => {
      expect(cellOf(lotsGrid(language), 0, 'cash')).toHaveClass('cell-warn');
    });
    // The words name the amount in rupees, in the app language, and never the code.
    expect(visible(document.body.textContent)).toContain(
      message(language, 'err-advance-taken-from-later', { taken: '5,000' }),
    );
    expect(visible(document.body.textContent)).not.toContain('ERR_ADVANCE_TAKEN_FROM_LATER');
    // The number in the cell is the backend's default, left empty by the screen as before.
    expect(textOf(lotsGrid(language), 0, 'cash')).toBe('5,160');
    expect(backend.lastDraft().lots[0]?.cash_paid).toBeNull();
  });
});
