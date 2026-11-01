// The پختہ کھاتہ hint on the new account form (plan 45, PRD 13, FR-13.5, FR-7.29, FR-7.1): a short
// grey note under the type field of the NEW form, telling the user that a one-time buyer or seller
// has no account and is typed on the sale line under پختہ کھاتہ. The edit form does not show it. The
// type list still offers only seller and buyer (plan 32, R6). Urdu and English.

import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, paragraph, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const NASIR = account(23, 'buyer', '0022', 'ناصر', 'Nasir');
const named = (text: string) => (name: string) => visible(name) === text;

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the پختہ کھاتہ hint on the account form (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('the new form shows the note under the type field, with پختہ کھاتہ in it', async () => {
    fakeAccounts([]);
    await renderApp('/index/accounts/new');
    const note = await screen.findByText(paragraph(message(language, 'accounts-field-type-hint')));
    // The glossary word of the language is in the sentence (PRD Section 3).
    expect(note.textContent).toContain(message(language, 'glossary-pukhta-khata'));
    // It is the type field's own note: the type control is described by it.
    const type = await screen.findByRole('combobox', {
      name: named(message(language, 'accounts-field-type')),
    });
    expect(type.getAttribute('aria-describedby')).toBe(note.id);
  });

  it('the edit form does not show the note', async () => {
    fakeAccounts([NASIR]);
    await renderApp('/index/accounts/23');
    await screen.findByRole('heading', { name: /ناصر|Nasir/ });
    expect(
      screen.queryByText(paragraph(message(language, 'accounts-field-type-hint'))),
    ).not.toBeInTheDocument();
  });

  it('the type list on the new form still offers only seller and buyer', async () => {
    fakeAccounts([]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'accounts-field-type')),
      }),
    );
    const shown = (await screen.findAllByRole('option')).map((option) =>
      visible(option.textContent),
    );
    expect(shown).toEqual([
      message(language, 'accounts-type-seller'),
      message(language, 'accounts-type-buyer'),
    ]);
  });
});
