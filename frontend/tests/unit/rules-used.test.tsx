// A rule that saved lines were charged by (plan 44, PRD FR-7.70, FR-11.5, FR-11.6), in the Urdu and the
// English app: its form is shut but for the end date, the active switch and the way out ("write a new
// rule"); a rule no saved line used keeps every field open.

import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters, record } from '../helpers/masters';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';
import { fakeRules, rule } from '../helpers/rules';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/rules/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const AKRAM = account(21, 'buyer', '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax');
const BANANA = record('item', 5, '0005', 'کیلا', 'Banana');
const named = (text: string) => (name: string) => visible(name) === text;

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  fakeMasters({ item: [BANANA] });
  fakeAccounts([AKRAM]);
  fakeAccountSearch([AKRAM]);
});
afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('a rule that saved lines used (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const field = (id: string) =>
    screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

  it('shuts every field but the end date, the active switch and the way out', async () => {
    fakeRules([rule(3, { used: true, from_date: '2026-10-01', to_date: null })]);
    const { router } = await renderApp('/index/rules/3');
    expect(await screen.findByText(message(language, 'rules-used-note'))).toBeInTheDocument();
    expect(await field('rules-field-from')).toBeDisabled();
    expect(await field('rules-field-note')).toBeDisabled();
    expect(await field('rules-field-to')).toBeEnabled();
    expect(
      screen.getByRole('checkbox', { name: named(message(language, 'rules-field-active')) }),
    ).toBeEnabled();
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: named(message(language, 'rules-used-new')) }));
    expect(router.state.location.pathname).toBe('/index/rules/new');
  });

  it('a rule no saved line used keeps every field open', async () => {
    fakeRules([rule(4, { used: false, from_date: '2026-10-01', to_date: null })]);
    await renderApp('/index/rules/4');
    expect(await field('rules-field-from')).toBeEnabled();
    expect(await field('rules-field-note')).toBeEnabled();
    expect(screen.queryByText(message(language, 'rules-used-note'))).toBeNull();
  });
});
