// Event 2 (PRD 4.5) by keyboard, to the save (plan 13, plan 64): a lot from Karim, sold to a چکتا
// buyer who pays the whole bill on the spot. The draft opens with the lot (typing a whole lot into
// two grids is the Windows spec's job, 13-bikri-save.e2e.mjs). The munshi takes the چکتا buyer from
// the buyer box, either a person the list holds (Shahid, marked چکتا) or a new one made with the
// second add row and its short form (PRD FR-7.69, FR-13.3). The line has no name or phone cells of
// its own and no extra Enter stop: buyer, quantity, rate, and the paid cell is skipped. He presses
// F10, and the draft that goes to the backend sends the account چکتا کھاتہ with his names and phone.
// In the Urdu and the English app. The backend's own numbers for Event 2 (E2E-5) are proved in the
// services tests; this proves the screen sends the right text.

import { act, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { IDS } from '../helpers/bikri-data';
import { seedBikriDraft } from '../helpers/bikri-seed';
import {
  cellOf,
  expectNewEmptyBook,
  focusedIn,
  named,
  openBook,
  salesGrid,
} from '../helpers/bikri-screen';
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
  account_id: IDS.chuktaKhata,
  name_ur: 'طارق',
  name_en: 'Tariq',
  phone: '03002222222',
  last_date: null,
};

type View = Awaited<ReturnType<typeof openBook>>;

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

  /** Opens the book on Event 2's lot, with the cursor in the buyer box of the first line. */
  async function openOnBuyer() {
    seedBikriDraft({ lot: 'karim' });
    const view = await openBook(language);
    await waitFor(() => {
      expect(cellOf(salesGrid(language), 0, 'buyer')).toBeInTheDocument();
    }, SLOW_MS);
    await view.user.click(cellOf(salesGrid(language), 0, 'buyer'));
    return view;
  }

  /** What is left after the buyer is chosen: 10 at 1,200 and F10, then the draft that was saved. */
  async function typeLineAndSave({ backend, user }: View) {
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
    }, SLOW_MS);
    // 10 at 1,200. A چکتا buyer pays the whole bill: the paid cell cannot be typed in, so Enter after
    // the rate goes on (NFR-USE-6); the payment type is the backend's, so the screen sends none.
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
    return backend.saves[0]?.draft;
  }

  it('takes Shahid, marked چکتا, from the list: his names and phone go with the line, 10 at 1,200, saved', async () => {
    const view = await openOnBuyer();
    const { user } = view;
    await user.keyboard('shahid');
    // The list holds him as a چکتا buyer: his name, the mark and his phone.
    const option = await screen.findByRole(
      'option',
      {
        name: (name) => visible(name).includes(language === 'ur' ? 'شاہد' : 'Shahid'),
      },
      SLOW_MS,
    );
    expect(within(option).getByText(message(language, 'accounts-type-chukta'))).toBeInTheDocument();
    expect(visible(option.textContent)).toContain('03000000000');
    await user.keyboard('{Enter}');
    const draft = await typeLineAndSave(view);
    // Karim's lot, as seeded.
    expect(draft?.lots[0]).toMatchObject({
      seller_account_id: IDS.karim,
      item_id: IDS.apple,
      qty_received: '10',
      freight: '1000',
      labour: { method: 'manual', value: '50' },
      munshiana: { method: 'manual', value: '50' },
    });
    // The line goes to the account چکتا کھاتہ, with the person's names and phone (PRD FR-13.5).
    const sale = draft?.lots[0]?.sales[0];
    expect(sale).toMatchObject({
      buyer_account_id: IDS.chuktaKhata,
      qty: '10',
      seller_rate: '1200',
      chukta: { name_ur: 'شاہد', name_en: 'Shahid', phone: '03000000000' },
    });
    // Nothing typed in the paid cell: sent as nothing, and the backend makes it the whole bill (FR-12.44).
    expect(sale?.paid).toBeNull();
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'bikri-saved-toast', { number: 1 }),
      );
    });
    await expectNewEmptyBook(language);
  });

  it('adds a new چکتا buyer with the second add row: the short form saves him and the line takes him', async () => {
    vi.mocked(accountApi.chuktaAdd).mockResolvedValue(TARIQ);
    const view = await openOnBuyer();
    const { user } = view;
    await user.keyboard('tariq');
    // Nobody matches, so the list ends with the two add rows: a new buyer, then a new چکتا buyer.
    await waitFor(() => {
      expect(screen.queryAllByRole('option')).toHaveLength(0);
    }, SLOW_MS);
    await user.keyboard('{ArrowDown}{ArrowDown}{Enter}');
    const dialog = await screen.findByRole(
      'dialog',
      {
        name: named(message(language, 'chukta-add-title')),
      },
      SLOW_MS,
    );
    // The typed name is in his name; the other name fills by itself (PRD FR-7.24).
    const urdu = within(dialog).getByLabelText<HTMLInputElement>(
      named(message(language, 'accounts-field-name-ur')),
    );
    const english = within(dialog).getByLabelText<HTMLInputElement>(
      named(message(language, 'accounts-field-name-en')),
    );
    await waitFor(() => {
      expect(urdu.value).toBe('طارق');
    }, SLOW_MS);
    await waitFor(() => {
      expect(english.value.toLowerCase()).toBe('tariq');
    }, SLOW_MS);
    // The phone is the last box of the form. The cursor goes there with the keyboard (focus, no click:
    // a click would leave the grid cell, and the cell ends its edit when focus is pressed elsewhere).
    act(() => {
      within(dialog)
        .getByRole('textbox', { name: named(message(language, 'accounts-field-phone1')) })
        .focus();
    });
    await user.keyboard('0300-2222222');
    press('F10');
    await waitFor(() => {
      expect(accountApi.chuktaAdd).toHaveBeenCalledTimes(1);
    }, SLOW_MS);
    const added = vi.mocked(accountApi.chuktaAdd).mock.calls[0]?.[0];
    expect(added).toMatchObject({ name_ur: 'طارق', phone: '0300-2222222' });
    await waitFor(() => {
      expect(
        screen.queryByRole('dialog', { name: named(message(language, 'chukta-add-title')) }),
      ).toBeNull();
    }, SLOW_MS);
    // He is chosen at once: his name is in the box, with the cursor still there, and the screen
    // under the form kept everything it had. Enter carries on to the quantity (PRD FR-7.69).
    await waitFor(() => {
      expect(
        document.querySelector<HTMLInputElement>('.ag-custom-component-popup input')?.value,
      ).toBe(language === 'ur' ? 'طارق' : 'Tariq');
    }, SLOW_MS);
    await user.keyboard('{Enter}');
    const draft = await typeLineAndSave(view);
    expect(draft?.lots[0]?.sales[0]).toMatchObject({
      buyer_account_id: IDS.chuktaKhata,
      chukta: { name_ur: 'طارق', name_en: 'Tariq', phone: '03002222222' },
    });
    expect(draft?.lots[0]).toMatchObject({ seller_account_id: IDS.karim, qty_received: '10' });
  });
});
