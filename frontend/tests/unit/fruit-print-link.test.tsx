// The "Print the list" button of the پھل screen (round 14, PRD FR-16.12, FR-19.10), in the Urdu and the
// English app: it opens the fruit register (R37) with the same days, the same partner or guests and
// the same item as the list under the form, so the page a partner or the Owner prints is the list he
// is looking at. The register opens a line on its entry (`?entry=`), which the fruit screen reads.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeFruit, fakeFruitPickers } from '../helpers/fruit';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/fruit/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeFruitPickers();
  fakeFruit();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the fruit screen prints its list as R37 (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);

  it('opens the fruit register with the days of the list, today by default', async () => {
    const { router } = await renderApp('/rokar/fruit');
    await screen.findByRole('heading', { level: 1, name: named(m('fruit-title')) });
    const user = userEvent.setup();
    await user.click(await screen.findByRole('button', { name: named(m('fruit-print')) }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R37');
    });
    // The day typed in the list is the day the register asks for; no partner and no guests.
    const search = router.state.location.search as Record<string, unknown>;
    expect(typeof search['from']).toBe('string');
    expect(search['from']).toBe(search['to']);
    expect(search['partner']).toBeUndefined();
    expect(search['guests']).toBeUndefined();
  });

  it('carries the guests choice to the register', async () => {
    const { router } = await renderApp('/rokar/fruit');
    await screen.findByRole('heading', { level: 1, name: named(m('fruit-title')) });
    const user = userEvent.setup();
    await user.click(screen.getByRole('combobox', { name: named(m('fruit-filter-for')) }));
    await user.click(await screen.findByRole('option', { name: m('fruit-for-guests') }));
    await user.click(screen.getByRole('button', { name: named(m('fruit-print')) }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R37');
    });
    expect((router.state.location.search as Record<string, unknown>)['guests']).toBe(true);
  });

  it('carries a partner to the register, and the register starts with him chosen and no guests', async () => {
    const { router } = await renderApp('/rokar/fruit');
    await screen.findByRole('heading', { level: 1, name: named(m('fruit-title')) });
    const user = userEvent.setup();
    await user.click(screen.getByRole('combobox', { name: named(m('fruit-filter-for')) }));
    await user.click(
      await screen.findByRole('option', { name: language === 'ur' ? 'الف' : 'Partner A' }),
    );
    await user.click(screen.getByRole('button', { name: named(m('fruit-print')) }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R37');
    });
    const search = router.state.location.search as Record<string, unknown>;
    expect(search['partner']).toBe(1);
    expect(search['guests']).toBeUndefined();
  });
});
