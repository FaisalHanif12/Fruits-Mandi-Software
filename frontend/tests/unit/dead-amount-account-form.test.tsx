// The dead amount on the account form (plan 40, PRD FR-10.10, E71, E73): an active buyer who owes
// has "ڈوبی رقم میں ڈالیں" beside "غیر فعال کریں"; a person already in the dead amount shows the move
// with its date and reason, and the button "فعال کریں" becomes "دوبارہ فعال کریں", which asks for a
// reason; a person who owes nothing, or an inactive one who is not dead, is as before. In the Urdu
// and in the English app, against a fake backend.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as mastersApi from '../../src/features/masters/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, balance, cardOf, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { ARSHAD_DEAD, ARSHAD_PREVIEW } from '../helpers/dead';
import { fakeMasters } from '../helpers/masters';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const OWES = account(22, 'buyer', '0021', 'ارشد ملتانی', 'Arshad Multani', {
  balance: balance(18684, 'dr'),
});
const SETTLED = account(23, 'buyer', '0022', 'ناصر', 'Nasir', { balance: balance(0, 'zero') });
const GONE = account(24, 'buyer', '0023', 'عمر', 'Umar', {
  balance: balance(5000, 'dr'),
  is_active: false,
});
const JUST_OLD = account(41, 'buyer', '0028', 'پرانا', 'Old One', { is_active: false });

const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeMasters();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue({ pairs: [], total: 0 } as never);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
  vi.mocked(mastersApi.deadPreview).mockResolvedValue(ARSHAD_PREVIEW);
  vi.mocked(mastersApi.deadUndo).mockResolvedValue({ ok: true });
  fakeAccounts([OWES, SETTLED, GONE, JUST_OLD]);
  vi.mocked(accountApi.assistPartyCard).mockImplementation((id) => {
    const row = [OWES, SETTLED, GONE, JUST_OLD].find((one) => one.id === id);
    if (row === undefined) return Promise.reject(new Error('not found'));
    return Promise.resolve(
      id === GONE.id ? cardOf(row, { dead: ARSHAD_DEAD }) : cardOf(row, { dead: null }),
    );
  });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the dead amount on the account form (%s)', (language: Language) => {
  const m = (id: string) => message(language, id);
  const named = (text: string) => (name: string) => visible(name) === text;
  const button = (id: string) => screen.findByRole('button', { name: named(m(id)) }, SLOW_MS);

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('an active buyer who owes can be moved: the button opens the dialog for his account', async () => {
    await renderApp('/index/accounts/22');
    await button('accounts-make-inactive');
    const move = await button('dead-move-button');
    await userEvent.setup().click(move);
    const dialog = await screen.findByRole('dialog');
    expect(mastersApi.deadPreview).toHaveBeenCalledWith({
      account_id: 22,
      pukhta_person_key: null,
    });
    expect(visible(dialog.textContent)).toContain(
      language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani',
    );
    await within(dialog).findByText('18,684');
  });

  it('a buyer who owes nothing has no move button', async () => {
    await renderApp('/index/accounts/23');
    await button('accounts-make-inactive');
    expect(screen.queryByRole('button', { name: named(m('dead-move-button')) })).toBeNull();
  });

  it('a person in the dead amount shows the move, and "make active again" asks for a reason', async () => {
    await renderApp('/index/accounts/24');
    const undo = await button('dead-undo-button');
    expect(
      await screen.findByText(
        (_, element) =>
          element?.tagName === 'SPAN' &&
          visible(element.textContent) ===
            message(language, 'dead-info', { date: '11-10-2026', reason: m('dead-reason-died') }),
        undefined,
        SLOW_MS,
      ),
    ).toBeInTheDocument();
    // The plain "make active" would leave the move behind, so it is not offered.
    expect(screen.queryByRole('button', { name: named(m('accounts-make-active')) })).toBeNull();
    expect(screen.queryByRole('button', { name: named(m('dead-move-button')) })).toBeNull();
    const user = userEvent.setup();
    await user.click(undo);
    const dialog = await screen.findByRole('alertdialog');
    await user.type(within(dialog).getByRole('combobox'), 'Back in business');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(mastersApi.deadUndo).toHaveBeenCalledWith({
        mark_id: ARSHAD_DEAD.mark_id,
        reason: 'Back in business',
      });
    });
  });

  it('an inactive account that is not in the dead amount keeps its plain "make active"', async () => {
    await renderApp('/index/accounts/41');
    await button('accounts-make-active');
    expect(screen.queryByRole('button', { name: named(m('dead-undo-button')) })).toBeNull();
    expect(screen.queryByRole('button', { name: named(m('dead-move-button')) })).toBeNull();
  });
});
