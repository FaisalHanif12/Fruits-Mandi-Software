// The short forms of "add it here" from the buyer box of a بکری line, used with the MOUSE (plan 64,
// PRD FR-7.69, FR-13.3, NFR-USE-8). The form is drawn outside the grid cell, and AG Grid ends a
// cell's edit when a click lands outside it, which closed the form on the first click into one of its
// boxes. The form carries the class AG Grid leaves alone (FormDialog), so clicks work in it as keys
// do. Checked in a real Chrome and WebKit page too (the dev page /dev/bikri); this is the guard.
// The ordinary new-buyer form and the new-چکتا-buyer form, in the English app.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { seedBikriDraft } from '../helpers/bikri-seed';
import { cellOf, named, openBook, salesGrid } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const NAMES = {
  toUrdu: { shahid: ['شاہد'], tariq: ['طارق'] },
  toEnglish: { شاہد: 'Shahid', طارق: 'Tariq' },
};

/** A new چکتا buyer, as the backend answers when the short form saves him (the fixture of `chukta_add`). */
const TARIQ = {
  id: 3,
  account_id: 9,
  name_ur: 'طارق',
  name_en: 'Tariq',
  phone: '03002222222',
  last_date: null,
};

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
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

const NEW_ORDINARY = /^new account$/i;

describe.each(['ordinary', 'chukta'] as const)('the %s short form, by mouse', (which) => {
  beforeEach(() => {
    resetClientState('en');
    signedIn(sessionUser({ language: 'en' }));
    vi.mocked(accountApi.chuktaAdd).mockResolvedValue(TARIQ);
  });

  /** Opens the form from a new line's buyer box with a click on its add row. */
  async function openByClick() {
    seedBikriDraft({ lot: 'karim' });
    const view = await openBook('en');
    await waitFor(() => {
      expect(cellOf(salesGrid('en'), 0, 'buyer')).toBeInTheDocument();
    }, SLOW_MS);
    await view.user.click(cellOf(salesGrid('en'), 0, 'buyer'));
    await view.user.keyboard('tariq');
    await waitFor(() => {
      expect(screen.queryAllByRole('option')).toHaveLength(0);
    }, SLOW_MS);
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>(
        which === 'chukta' ? '[id$="-add-extra"]' : '[id$="-add"]',
      );
      if (found === null) throw new Error('no add row yet');
      return found;
    }, SLOW_MS);
    await view.user.click(row);
    const dialog = await screen.findByRole(
      'dialog',
      { name: which === 'chukta' ? named(message('en', 'chukta-add-title')) : NEW_ORDINARY },
      SLOW_MS,
    );
    return { ...view, dialog };
  }

  it('stays open when a click lands in one of its boxes, and the cell keeps its edit', async () => {
    const { user, dialog } = await openByClick();
    const boxes = within(dialog).getAllByRole('textbox');
    const second = boxes[1];
    const last = boxes.at(-1);
    if (second === undefined || last === undefined) throw new Error('the form has too few boxes');
    await user.click(second);
    await user.click(last);
    await new Promise((resolve) => setTimeout(resolve, 300));
    // Still the same form, still open, and the picker of the cell behind it is still there.
    expect(dialog).toBeInTheDocument();
    expect(dialog.getAttribute('data-state')).toBe('open');
    expect(document.querySelectorAll('.ag-popup-editor').length).toBe(1);
  });

  it('closes with Back by mouse and leaves the typed text in the box', async () => {
    const { user, dialog } = await openByClick();
    await user.click(within(dialog).getByRole('button', { name: /back/i }));
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: /new/i })).toBeNull();
    }, SLOW_MS);
    expect(document.querySelector<HTMLInputElement>('.ag-popup-editor input')?.value).toBe('tariq');
  });
});

describe('the new-چکتا-buyer form, saved by mouse', () => {
  beforeEach(() => {
    resetClientState('en');
    signedIn(sessionUser({ language: 'en' }));
    vi.mocked(accountApi.chuktaAdd).mockResolvedValue(TARIQ);
  });

  it('click the add row, click into the phone, type, click Save: he is in the box', async () => {
    seedBikriDraft({ lot: 'karim' });
    const { user } = await openBook('en');
    await waitFor(() => {
      expect(cellOf(salesGrid('en'), 0, 'buyer')).toBeInTheDocument();
    }, SLOW_MS);
    await user.click(cellOf(salesGrid('en'), 0, 'buyer'));
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(screen.queryAllByRole('option')).toHaveLength(0);
    }, SLOW_MS);
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('[id$="-add-extra"]');
      if (found === null) throw new Error('no second add row yet');
      return found;
    }, SLOW_MS);
    await user.click(row);
    const dialog = await screen.findByRole(
      'dialog',
      { name: named(message('en', 'chukta-add-title')) },
      SLOW_MS,
    );
    await waitFor(() => {
      expect(
        within(dialog).getByLabelText<HTMLInputElement>(
          named(message('en', 'accounts-field-name-ur')),
        ).value,
      ).toBe('طارق');
    }, SLOW_MS);
    await user.type(
      within(dialog).getByRole('textbox', { name: named(message('en', 'accounts-field-phone1')) }),
      '0300-2222222',
    );
    await user.click(within(dialog).getByRole('button', { name: /^save/i }));
    await waitFor(() => {
      expect(accountApi.chuktaAdd).toHaveBeenCalledTimes(1);
    }, SLOW_MS);
    expect(vi.mocked(accountApi.chuktaAdd).mock.calls[0]?.[0]).toMatchObject({
      name_ur: 'طارق',
      phone: '0300-2222222',
    });
    await waitFor(() => {
      expect(document.querySelector<HTMLInputElement>('.ag-popup-editor input')?.value).toBe(
        'Tariq',
      );
    }, SLOW_MS);
    expect(
      screen.queryByRole('dialog', { name: named(message('en', 'chukta-add-title')) }),
    ).toBeNull();
  });
});
