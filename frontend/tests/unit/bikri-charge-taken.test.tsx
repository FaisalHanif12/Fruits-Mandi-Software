// The warning ERR_ENTRY_CHARGE_ALREADY_TAKEN on a lot of the sale book (plan 37, PRD FR-11.9): a
// lot charge (labour, munshiana or phone) that an earlier lot of the same goods-arrived entry has
// already taken. The backend says so in the `warnings` of the preview; the screen shows it on the
// lot's cell and in the list of problems, in the app language. The save is allowed and the typed
// amount stays. Urdu and English.

import { waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { FieldError } from '../../src/bindings/FieldError';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { previewFor } from '../helpers/bikri-data';
import { cellOf, lotsGrid, openBook, typeEvent1Lot } from '../helpers/bikri-screen';
import * as bikriApi from '../../src/features/bikri/api';
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

/** The backend's files have no اڈا and no filled fields: the truck has no memory. */
function noTruckMemory() {
  vi.mocked(bikriApi.assistTruckMemory).mockResolvedValue({
    last: null,
    same_day: null,
    suggestions: [],
  });
}

/** The preview of Event 1 with one more warning from the backend. */
function withWarning(warning: FieldError) {
  return (draft: BikriDraft): BikriPreview => {
    const preview = previewFor(draft);
    return { ...preview, warnings: [...preview.warnings, warning] };
  };
}

describe.each(LANGUAGES)('a charge already taken by an earlier lot (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('shows the warning on the labour cell and in the problems, and keeps what was typed', async () => {
    const { user } = await openBook(
      language,
      withWarning({
        code: 'ERR_ENTRY_CHARGE_ALREADY_TAKEN',
        field: 'lots[0].labour',
        values: { lot_no: 1, bikri_number: 0, entry: 3 },
      }),
    );
    noTruckMemory();
    await typeEvent1Lot(user, language);
    await waitFor(() => {
      expect(cellOf(lotsGrid(language), 0, 'labour')).toHaveClass('cell-warn');
    });
    // A lot of this same sale (0): the words name the lot and the entry only.
    expect(visible(document.body.textContent)).toContain(
      message(language, 'err-entry-charge-already-taken', {
        lot_no: 1,
        bikri_number: 0,
        bikri_number_zero: 'yes',
        entry: 3,
      }),
    );
    // The typed amount is still in the cell: the warning does not clear it.
    expect(visible(cellOf(lotsGrid(language), 0, 'labour').textContent)).toContain('5');
  });

  it('names the sale of the earlier lot when it is another sale, on the munshiana cell', async () => {
    const { user } = await openBook(
      language,
      withWarning({
        code: 'ERR_ENTRY_CHARGE_ALREADY_TAKEN',
        field: 'lots[0].munshiana',
        values: { lot_no: 2, bikri_number: 12, entry: 3 },
      }),
    );
    noTruckMemory();
    await typeEvent1Lot(user, language);
    await waitFor(() => {
      expect(cellOf(lotsGrid(language), 0, 'munshiana')).toHaveClass('cell-warn');
    });
    expect(visible(document.body.textContent)).toContain(
      message(language, 'err-entry-charge-already-taken', {
        lot_no: 2,
        bikri_number: 12,
        bikri_number_zero: 'no',
        entry: 3,
      }),
    );
  });
});
