// The پھل list while the partner section is locked (PRD FR-17.11, plan 80 R4-1), in the Urdu and the English
// app. The list answers with `partners_locked: true` and holds the fruit for the shop's guests only: the screen
// shows the code box ("کوڈ درج کریں") in the place of the partners' fruit and the rest of the list works. A request
// for one partner's fruit is refused with ERR_PARTNER_LOCKED: that too is the code box, not an error with a
// retry. With the code entered (an unlocked answer) there is no box.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { FruitList } from '../../src/bindings/FruitList';
import type { FruitListInput } from '../../src/bindings/FruitListInput';
import * as fruitApi from '../../src/features/fruit/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
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
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/fruit/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const HAJI = account(31, 'seller', '0012', 'حاجی نذیر', 'Haji Nazir');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeFruitPickers();
  fakeAccounts([HAJI]);
  fakeAccountSearch([HAJI]);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the fruit list and the partner code (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);
  const partnerA = language === 'ur' ? 'الف' : 'Partner A';

  /** Makes the list answer as the backend does: locked or not, and refusing a partner while locked. */
  const lockTheList = (locked: boolean) => {
    fakeFruit();
    const books = vi.mocked(fruitApi.fruitList).getMockImplementation();
    if (books === undefined) throw new Error('the double has no list');
    vi.mocked(fruitApi.fruitList).mockImplementation(async (input: FruitListInput) => {
      if (locked && input.partner_id !== null) {
        throw new CommandError(
          refusal([{ code: 'ERR_PARTNER_LOCKED', field: 'partner_id', values: {} }]),
        );
      }
      const list: FruitList = await books(input);
      return { ...list, partners_locked: locked };
    });
  };

  const open = async () => {
    const view = await renderApp('/rokar/fruit');
    await screen.findByRole('heading', { level: 1, name: named(m('fruit-title')) });
    return { ...view, user: userEvent.setup() };
  };
  const codeBox = () => screen.queryByText(visible(m('partners-code-box-text')));

  it('a locked list shows the code box with its button, and the rest of the screen works', async () => {
    lockTheList(true);
    await open();
    expect(await screen.findByText(visible(m('partners-code-box-text')))).toBeInTheDocument();
    expect(screen.getByRole('button', { name: named(m('partners-code-enter')) })).toBeEnabled();
    // The list itself is still there, with its filters.
    expect(
      screen.getByRole('combobox', { name: named(m('fruit-filter-for')) }),
    ).toBeInTheDocument();
  });

  it('an unlocked list shows no code box', async () => {
    lockTheList(false);
    await open();
    await waitFor(() => {
      expect(fruitApi.fruitList).toHaveBeenCalled();
    });
    await screen.findByText(visible(m('fruit-list-empty')));
    expect(codeBox()).toBeNull();
  });

  it("one partner's fruit refused with ERR_PARTNER_LOCKED is the code box, not an error with a retry", async () => {
    lockTheList(true);
    const { user } = await open();
    await user.click(screen.getByRole('combobox', { name: named(m('fruit-filter-for')) }));
    await user.click(await screen.findByRole('option', { name: named(partnerA) }));
    await waitFor(() => {
      expect(fruitApi.fruitList).toHaveBeenCalledWith(expect.objectContaining({ partner_id: 1 }));
    });
    expect(await screen.findByText(visible(m('partners-code-box-text')))).toBeInTheDocument();
    expect(screen.queryByText(visible(m('fruit-list-failed')))).toBeNull();
  });
});
