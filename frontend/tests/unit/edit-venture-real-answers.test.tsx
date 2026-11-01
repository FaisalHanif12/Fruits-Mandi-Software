// The edits of the trip, the garden and the purchase against the backend's own answers (plan 44, T2a
// part 2): every real review is drawn in words in both languages with a name for every field, and a
// trip cost goes through the screen with the real preview and the real save answers.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as tripsApi from '../../src/features/trips/api';
import { LANGUAGES } from '../../src/i18n';
import { EditReviewCard } from '../../src/shared/ui';
import * as editApi from '../../src/shared/edit/api';
import * as urduApi from '../../src/shared/urdu/api';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import {
  GARDENS_ARRIVAL_EDIT_PREVIEW_120,
  GARDENS_ENTRY_EDIT_PREVIEW_32000,
  GARDENS_HEADER_EDIT_PREVIEW_RENT_60000,
  PURCHASES_EDIT_PREVIEW_RATE_520,
  TRIPS_COST_EDIT_PREVIEW_4400,
  TRIPS_COST_EDIT_SAVE_4400,
  TRIPS_HEADER_EDIT_PREVIEW_MARKET,
  TRIPS_OTHER_SALE_EDIT_PREVIEW_9000,
} from '../helpers/edit-real';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, renderIn, visible } from '../helpers/render';
import { costEntry, fakeItemPickers, fakeTrips } from '../helpers/trips';
import { paisaForTest } from '../helpers/paisa';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/purchases/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

beforeEach(() => {
  vi.resetAllMocks();
});
afterEach(() => {
  leaveAppWindow();
});

const REAL = [
  ['a trip cost 4,000 to 4,400', TRIPS_COST_EDIT_PREVIEW_4400, ['4,000', '4,400']],
  ['a sale in another shop to 9,000', TRIPS_OTHER_SALE_EDIT_PREVIEW_9000, ['9,000']],
  ['the market of a trip', TRIPS_HEADER_EDIT_PREVIEW_MARKET, ['Multan']],
  ['a purchase rate to 520', PURCHASES_EDIT_PREVIEW_RATE_520, ['520']],
  ['a garden installment to 32,000', GARDENS_ENTRY_EDIT_PREVIEW_32000, ['32,000']],
  ['goods arrived to 120', GARDENS_ARRIVAL_EDIT_PREVIEW_120, ['120']],
  ['the rent of a garden to 60,000', GARDENS_HEADER_EDIT_PREVIEW_RENT_60000, ['60,000']],
] as const;

describe.each(LANGUAGES)('the real reviews of the edits are drawn in words (%s)', (language) => {
  it.each(REAL)('%s', (_name, preview, figures) => {
    renderIn(language, <EditReviewCard review={preview.review} />);
    const text = visible(document.body.textContent);
    for (const figure of figures) expect(text).toContain(figure);
    // Every field the backend names has a name of its own: none falls back to "other field".
    expect(text).not.toContain(message(language, 'edit-field-other'));
  });
});

describe.each(LANGUAGES)(
  'a trip cost through the screen, with the real answers (%s)',
  (language) => {
    beforeEach(() => {
      healthyBackend();
      fakePartners(THREE_PARTNERS);
      fakeItemPickers();
      vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
      vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
      vi.mocked(editApi.auditEditStamp).mockResolvedValue([]);
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    it("the backend's review is shown and the save is asked as the backend's own request", async () => {
      fakeTrips({
        entries: [
          costEntry({
            id: 2,
            version: 2,
            amount_paisa: paisaForTest(400_000),
            reason: 'truck',
          }),
        ],
      });
      vi.mocked(tripsApi.tripsCostEditPreview).mockResolvedValue(TRIPS_COST_EDIT_PREVIEW_4400);
      vi.mocked(tripsApi.tripsCostEditSave).mockResolvedValue(TRIPS_COST_EDIT_SAVE_4400);
      await renderApp('/trips/1');
      const user = userEvent.setup();
      const row = await waitFor(() => {
        const found = document.querySelector<HTMLElement>(
          '.ag-row[row-id="cost-2"] [col-id="entry"]',
        );
        if (found === null) throw new Error('no cost row');
        return found;
      });
      await user.click(row);
      await user.click(
        screen.getByRole('button', {
          name: (name) => visible(name).startsWith(message(language, 'edit-action-edit')),
        }),
      );
      const amount = await screen.findByRole('textbox', {
        name: named(message(language, 'trips-cost-amount')),
      });
      await user.clear(amount);
      await user.type(amount, '4,400');
      // The review of the backend's own file is on the screen, in words.
      await waitFor(() => {
        expect(visible(document.body.textContent)).toContain('4,400');
      });
      press('F10');
      await waitFor(() => {
        expect(tripsApi.tripsCostEditSave).toHaveBeenCalledTimes(1);
      });
      // The same request as the backend's file, but for the sources: the screen sends one row for every
      // partner (the contract asks for all), where the backend's file left them to the trip's defaults.
      expect(vi.mocked(tripsApi.tripsCostEditSave).mock.calls[0]?.[0]).toMatchObject({
        id: 2,
        version: 2,
        reason: '',
        cost: {
          trip_id: 1,
          date: '10-10-2026',
          amount: '4,400',
          reason: 'truck',
          confirm_cash_below_zero: false,
        },
      });
    });
  },
);
