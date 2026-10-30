// The header of the sale book (plan 38, PRD 12.2): one row of the fields typed on every بکری (date,
// truck, زمیندار, اڈا), and the rest (bill date, total freight with "Split", store, book, driver,
// note) behind a button at its end. A value behind it is never hidden without a sign: the button
// says how many of those fields hold something, and a problem on one of them opens the part.
// Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { previewFor } from '../helpers/bikri-data';
import {
  cellOf,
  focusedIn,
  lotsGrid,
  named,
  names,
  openBook,
  typeHeaderAndLot,
  type User,
} from '../helpers/bikri-screen';
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

// In a short window the header folds to one summary line while the user types in the lots or the sale
// lines (tech lead, plan 38). The test window has no height, so the rule `short:` does not apply: what
// is tested is the screen's own part of it. The summary is in the page, the real fields stay in the
// page and in the Tab order, focus opens it at once, and a problem or the same-truck notice keeps it open.
describe.each(LANGUAGES)(
  'the header folds to one line while the lines are typed (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    const summary = () =>
      document.querySelector<HTMLElement>('button[aria-hidden="true"][tabindex="-1"]');
    const dateField = () =>
      screen.getByRole('textbox', { name: named(message(language, 'bikri-header-date')) });

    /** The header and the lot typed, the same-truck notice answered, the cursor in the lots. */
    async function inTheLots(user: User) {
      await typeHeaderAndLot(user, language);
      await user.click(
        await screen.findByRole('button', {
          name: named(message(language, 'bikri-truck-continue')),
        }),
      );
      await user.click(cellOf(lotsGrid(language), 0, 'seller'));
    }

    it('is open at the start and folds when the cursor goes to the lots; the fields stay in the page', async () => {
      const { user } = await openBook(language);
      expect(summary()).toBeNull();
      await inTheLots(user);
      await waitFor(() => {
        expect(summary()).not.toBeNull();
      });
      // The summary says the four daily values as text.
      const words = visible(summary()?.textContent);
      expect(words).toContain('LES-1234');
      expect(words).toContain(names(language).rahim);
      expect(words).toContain(names(language).swat);
      // The real fields are still in the page, in the same order: the date is the first one.
      expect(dateField()).toBeInTheDocument();
    });

    it('stays open while the same-truck notice asks a question', async () => {
      const { user } = await openBook(language);
      await typeHeaderAndLot(user, language);
      expect(
        await screen.findByRole('button', {
          name: named(message(language, 'bikri-truck-continue')),
        }),
      ).toBeInTheDocument();
      expect(summary()).toBeNull();
    });

    it('opens at once when the cursor comes back into a header field, and folds again when it leaves', async () => {
      const { user } = await openBook(language);
      await inTheLots(user);
      await waitFor(() => {
        expect(summary()).not.toBeNull();
      });
      await user.click(dateField());
      await waitFor(() => {
        expect(summary()).toBeNull();
      });
      expect(document.activeElement).toBe(dateField());
      // Enter walks on through the header to the lots as before (date, truck, زمیندار, اڈا) and it folds.
      await user.keyboard('{Enter}{Enter}{Enter}{Enter}');
      await waitFor(() => {
        expect(focusedIn(lotsGrid(language))).not.toBeNull();
      });
      await waitFor(() => {
        expect(summary()).not.toBeNull();
      });
    });

    it('a click on the summary line puts the cursor in the date field and opens the header', async () => {
      const { user } = await openBook(language);
      await inTheLots(user);
      await waitFor(() => {
        expect(summary()).not.toBeNull();
      });
      const line = summary();
      if (line === null) throw new Error('no summary');
      await user.click(line);
      await waitFor(() => {
        expect(summary()).toBeNull();
      });
      expect(document.activeElement).toBe(dateField());
    });

    it('stays open while the backend points at a header field', async () => {
      const { user } = await openBook(language, (draft) => ({
        ...previewFor(draft),
        errors: [{ code: 'ERR_DATE_INVALID', field: 'header.entry_date', values: {} }],
      }));
      await inTheLots(user);
      expect(
        await screen.findByText((text) => visible(text) === message(language, 'err-date-invalid')),
      ).toBeInTheDocument();
      expect(summary()).toBeNull();
    });
  },
);
