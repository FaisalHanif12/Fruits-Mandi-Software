// Event 2 (PRD 4.5) by keyboard, to the save (plan 13): a lot from Karim, sold to a one-time
// buyer who pays cash. The draft opens with the lot and the one-time buyer account on the first
// line (typing a whole lot into two grids is the Windows spec's job, 13-bikri-save.e2e.mjs). The
// munshi types the one-time buyer's name and phone, the quantity and the rate, presses F10, and the
// draft that goes to the backend carries the one-time person. In the Urdu and the English app. The backend's own numbers for Event 2 (E2E-5) are
// proved in the services tests; this proves the screen sends the right text.

import { waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { IDS } from '../helpers/bikri-data';
import { seedBikriDraft } from '../helpers/bikri-seed';
import { combo, focusedIn, openBook, salesGrid } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const NAMES = { toUrdu: { shahid: ['شاہد'] }, toEnglish: { شاہد: 'Shahid' } };

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  // Reset, not only clear: a rejection queued with mockRejectedValueOnce by a test that failed early
  // must not reach the next test.
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(suggestionsFor(NAMES, text, direction)),
  );
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('Event 2 by keyboard (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('from the one-time buyer’s name to F10: Shahid with his phone, 10 at 1,200, saved', async () => {
    // The lot of Event 2 is already in the draft (a lot from Karim, PRD 4.5), with the one-time
    // buyer account on its first line. The keys start at his name.
    seedBikriDraft({ lot: 'karim', pukhtaBuyer: true });
    const { backend, user } = await openBook(language);
    const panel = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('[data-pukhta-for]');
      if (found === null) throw new Error('the one-time buyer has no fields yet');
      return found;
    }, SLOW_MS);
    const first = panel.querySelector('input');
    if (first === null) throw new Error('no name field');
    await user.click(first);
    // His name in the language of the app; the other name fills by itself and is an Enter stop
    // (PRD FR-7.24, NFR-USE-27); then his phone; then Enter goes on to the quantity.
    await user.keyboard(language === 'ur' ? 'shahid{Enter}' : 'Shahid{Enter}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.pukhta).toMatchObject({
        name_ur: 'شاہد',
        name_en: 'Shahid',
      });
    });
    await user.keyboard('{Enter}0300-0000000{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
    }, SLOW_MS);
    // 10 at 1,200. A one-time buyer pays cash: the payment type is the backend's default (FR-13.4),
    // so the screen sends none.
    await user.keyboard('10{Enter}1200{Enter}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]).toMatchObject({
        qty: '10',
        seller_rate: '1200',
      });
    });
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    }, SLOW_MS);
    const draft = backend.saves[0]?.draft;
    // Karim's lot, as seeded, and the one-time buyer, as typed.
    expect(draft?.lots[0]).toMatchObject({
      seller_account_id: IDS.karim,
      item_id: IDS.apple,
      qty_received: '10',
      freight: '1000',
      labour: { method: 'manual', value: '50' },
      munshiana: { method: 'manual', value: '50' },
    });
    expect(draft?.lots[0]?.pukhta).toBeNull();
    const sale = draft?.lots[0]?.sales[0];
    expect(sale).toMatchObject({
      buyer_account_id: IDS.pukhtaBuyer,
      qty: '10',
      seller_rate: '1200',
      pukhta: { name_ur: 'شاہد', name_en: 'Shahid', phone: '0300-0000000' },
    });
    expect(sale?.payment_type).toBeNull();
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'bikri-saved-toast', { number: 1 }),
      );
    });
    expect(await combo(language, 'bikri-header-truck')).toHaveValue('');
  });
});
