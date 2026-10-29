// The header of the sale book (plan 38, PRD 12.2): one row of the fields typed on every بکری (date,
// truck, زمیندار, اڈا), and the rest (bill date, total freight with "Split", store, book, driver,
// note) behind a button at its end. A value behind it is never hidden without a sign: the button
// says how many of those fields hold something, and a problem on one of them opens the part.
// Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { named, openBook } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

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

const moreButton = (language: Language) =>
  screen.getByRole('button', { name: named(message(language, 'bikri-header-more')) });
const lessButton = (language: Language) =>
  screen.getByRole('button', { name: named(message(language, 'bikri-header-less')) });
const freightBox = (language: Language) =>
  screen.queryByRole('textbox', { name: named(message(language, 'bikri-header-total-freight')) });

describe.each(LANGUAGES)('the header of the sale book (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('has the four daily fields in view and the others behind "More", reached by the keyboard', async () => {
    const { user } = await openBook(language);
    for (const id of [
      'bikri-header-date',
      'bikri-header-truck',
      'glossary-zamindar',
      'glossary-adda',
    ]) {
      expect(
        screen.getAllByText((text) => visible(text) === message(language, id)).length,
      ).toBeGreaterThan(0);
    }
    expect(freightBox(language)).not.toBeInTheDocument();
    expect(
      screen.queryByRole('textbox', { name: named(message(language, 'bikri-header-driver')) }),
    ).not.toBeInTheDocument();
    // The button is a real button: Tab reaches it, Enter or Space opens the part.
    moreButton(language).focus();
    await user.keyboard('{Enter}');
    expect(
      await screen.findByRole('textbox', {
        name: named(message(language, 'bikri-header-total-freight')),
      }),
    ).toBeInTheDocument();
    expect(lessButton(language)).toBeInTheDocument();
    await user.click(lessButton(language));
    await waitFor(() => {
      expect(freightBox(language)).not.toBeInTheDocument();
    });
  });

  it('says how many of the hidden fields hold a value, so none is hidden in secret', async () => {
    const { user } = await openBook(language);
    await user.click(moreButton(language));
    const freight = await screen.findByRole('textbox', {
      name: named(message(language, 'bikri-header-total-freight')),
    });
    await user.click(freight);
    await user.keyboard('4000');
    await user.click(lessButton(language));
    await waitFor(() => {
      expect(visible(moreButton(language).textContent)).toContain('1');
    });
    // Opened again, the value is still there.
    await user.click(moreButton(language));
    const again = await screen.findByRole('textbox', {
      name: named(message(language, 'bikri-header-total-freight')),
    });
    expect(again).toHaveDisplayValue(/^4,?000$/);
  });
});
