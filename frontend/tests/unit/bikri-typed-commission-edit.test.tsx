// Plan 72 P13 (PRD FR-11.10), the edit: a commission rate typed on a SAVED line of a بکری open for edit
// re-works the line at once. Same case as bikri-typed-commission.test.tsx: 12,000 at 4% is 480. The real
// screen, the real grid and the real store; the fake backend answers like the real one for a typed
// percent (method manual, source typed, the rate in basis points, the amount from the base).

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { answerWithTyped } from '../helpers/bikri-typed';
import { fakePlan16 } from '../helpers/bikri-plan16';
import { cellOf, named, salesGrid, textOf } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/edit/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

describe.each(LANGUAGES)('a commission rate typed on a saved line, in an edit (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  it('12,000 at 4% gives 480 in the grid of the edit, at once', async () => {
    fakePlan16();
    vi.mocked(bikriApi.bikriEditPreview).mockImplementation((_entryId, draft) =>
      Promise.resolve(answerWithTyped(draft)),
    );
    await renderApp('/bikri/book?id=41&edit=7');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-edit-title', { number: 1 })),
    });
    const sales = salesGrid(language);
    // The entry is loaded as a draft; the first preview is asked when something changes.
    await waitFor(() => {
      expect(textOf(sales, 0, 'buyer')).not.toBe('');
    }, SLOW_MS);
    const user = userEvent.setup();
    await user.click(cellOf(sales, 0, 'qty'));
    const field = await screen.findByRole('textbox', {
      name: named(message(language, 'bikri-line-seller-comm')),
    });
    await user.click(field);
    await user.keyboard('4');
    await waitFor(() => {
      expect(textOf(sales, 0, 'sellerComm')).toBe('480');
    }, SLOW_MS);
  });
});
