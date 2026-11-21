// A seller's money paid on the spot, in his summary on the بکری page (plan 91, PRD FR-12.23 v1.53, Umar 21-11-2026):
// beside the advance tick one field, "موقع پر ادائیگی", and the lines of his account (پچھلا بقایا, آج کی صافی بکری,
// موقع پر ادائیگی, پچھلے بنام سے کٹا, نیا بقایا) with their sides, all from the backend's preview. The amount typed is
// spread by the backend over his lots' نقد, and each lot's answer is written into the lot's own نقد field. When the
// amount is more than he may take, an amber line says the balance will grow, and the save asks "is this entry correct?"
// (the cash question's yes). In the Urdu and the English app. The screen works out no money: the figures are the
// fake backend's, set to the plan's worked numbers.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as urduApi from '../../src/shared/urdu/api';
import { previewFor, IDS } from '../helpers/bikri-data';
import { cellOf, lotsGrid, names, openBook, textOf, typeEvent1Lot } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';
import { sellerLines, spreadAnswer, withSellers } from '../helpers/seller-paid';
import { refusal } from '../helpers/ui-error';
import type { BikriDraft } from '../../src/bindings/BikriDraft';

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

const RAHIM = IDS.rahim;

describe.each(LANGUAGES)('a seller’s money paid on the spot (%s)', (language: Language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const block = () => {
    const found = document.querySelector<HTMLElement>(`[data-seller-paid="${String(RAHIM)}"]`);
    if (found === null) throw new Error('no seller block');
    return found;
  };
  const field = () =>
    within(block()).getByRole('textbox', {
      name: (name) => visible(name) === m('bikri-seller-paid-field'),
    });
  /** The lines of the block as "label: text" pairs, in order. */
  const lines = () =>
    Array.from(block().querySelectorAll('dl > div')).map((row) => ({
      label: visible(row.querySelector('dt')?.textContent),
      value: visible(row.querySelector('dd')?.textContent),
    }));
  const side = (id: 'dr' | 'cr') => m(id === 'dr' ? 'common-side-dr' : 'common-side-cr');

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** Opens the book with Event 1's lot typed and the plan's first worked example as the seller's lines. */
  async function openWith(options: Partial<Parameters<typeof sellerLines>[0]> = {}) {
    const book = await openBook(language, (draft) =>
      withSellers(previewFor(draft), [sellerLines({ sellerId: RAHIM, ...options })]),
    );
    await typeEvent1Lot(book.user, language);
    await waitFor(() => {
      expect(document.querySelector('[data-seller-paid]')).not.toBeNull();
    });
    await waitFor(() => {
      expect(lines().length).toBeGreaterThan(0);
    });
    return book;
  }

  it('shows the field and the lines of his account: previous, today’s net, paid, the rest credited to his account, new balance, with sides', async () => {
    await openWith();
    expect(visible(block().textContent)).toContain(
      m('bikri-seller-paid-title', { name: names(language).rahim }),
    );
    expect(lines()).toEqual([
      { label: m('bikri-seller-paid-previous'), value: `30,000${side('dr')}` },
      { label: m('bikri-seller-paid-net'), value: '20,000' },
      { label: m('bikri-seller-paid-paid'), value: '5,000' },
      { label: m('bikri-seller-paid-rest'), value: '15,000' },
      { label: m('bikri-seller-paid-new'), value: expect.stringContaining('15,000') as string },
    ]);
    expect(lines().at(-1)?.value).toContain(side('dr'));
    // The field shows the sum of his lots' نقد, as the backend worked it out.
    expect(field()).toHaveValue('5,000');
  });

  it('when the shop owed him the same line shows the rest of today’s sale, and a figure that is 0 has no line of its own', async () => {
    await openWith({
      previous: { rupees: 10_000, side: 'cr' },
      credit: 20_000,
      paid: 5_000,
      carry: 15_000,
      cut: 0,
      added: 15_000,
      new: { rupees: 25_000, side: 'cr' },
    });
    const labels = lines().map((line) => line.label);
    expect(labels).toContain(m('bikri-seller-paid-rest'));
    expect(lines().find((line) => line.label === m('bikri-seller-paid-rest'))?.value).toBe(
      '15,000',
    );
    expect(labels).not.toContain(m('bikri-seller-paid-over'));
    expect(lines()[0]?.value).toContain(side('cr'));
  });

  it('paid above today’s sale with the shop owing him enough: the line "paid above" shows and no warning', async () => {
    await openWith({
      previous: { rupees: 10_000, side: 'cr' },
      paid: 28_000,
      carry: 0,
      cut: 0,
      over: 8_000,
      new: { rupees: 2_000, side: 'cr' },
      limit: 30_000,
    });
    expect(lines().map((line) => line.label)).toContain(m('bikri-seller-paid-over'));
    expect(
      within(block()).queryByText((text) => visible(text) === m('bikri-seller-paid-above')),
    ).toBeNull();
  });

  it('typing an amount asks the backend to spread it, and each lot’s answer goes into the lot’s own نقد field', async () => {
    const { backend, user } = await openWith();
    vi.mocked(bikriApi.bikriSellerPaidSpread).mockImplementation((input) =>
      Promise.resolve(
        spreadAnswer(
          withSellers(previewFor(input.draft), [sellerLines({ sellerId: RAHIM, paid: 15_000 })]),
          input.draft.lots.map((lot) => ({ key: lot.key, rupees: 15_000 })),
        ),
      ),
    );
    const box = field();
    await user.click(box);
    await user.keyboard('{Control>}a{/Control}15000');
    await waitFor(() => {
      expect(bikriApi.bikriSellerPaidSpread).toHaveBeenCalled();
    });
    const sent = vi.mocked(bikriApi.bikriSellerPaidSpread).mock.calls.at(-1)?.[0];
    expect(sent?.seller_paid).toEqual([{ seller_account_id: RAHIM, amount: '15000' }]);
    // A new بکری: no posting is left out.
    expect(sent?.entry_id ?? null).toBeNull();
    // The answer is in the lot's own نقد cell, a value the app filled (it is the draft the screen sends now).
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'cash')).toBe('15,000');
    });
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.cash_paid).toBe('15000');
    });
  });

  it('emptying the field sends 0, so the lots’ نقد go to nothing', async () => {
    const { user } = await openWith();
    vi.mocked(bikriApi.bikriSellerPaidSpread).mockImplementation((input) =>
      Promise.resolve(
        spreadAnswer(
          withSellers(previewFor(input.draft), [sellerLines({ sellerId: RAHIM, paid: 0 })]),
          input.draft.lots.map((lot) => ({ key: lot.key, rupees: 0 })),
        ),
      ),
    );
    await user.click(field());
    await user.keyboard('{Control>}a{/Control}{Backspace}');
    await waitFor(() => {
      expect(bikriApi.bikriSellerPaidSpread).toHaveBeenCalled();
    });
    expect(vi.mocked(bikriApi.bikriSellerPaidSpread).mock.calls.at(-1)?.[0].seller_paid).toEqual([
      { seller_account_id: RAHIM, amount: '0' },
    ]);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'cash')).toBe('');
    });
  });

  it('what the backend refuses in the amount shows under the field', async () => {
    const { user } = await openWith();
    vi.mocked(bikriApi.bikriSellerPaidSpread).mockRejectedValue(
      new CommandError(
        refusal([{ code: 'ERR_DECIMALS_NOT_ALLOWED', field: 'seller_paid[0].amount', values: {} }]),
      ),
    );
    await user.click(field());
    await user.keyboard('{Control>}a{/Control}12.5');
    expect(
      await within(block()).findByText((text) => visible(text) === m('err-decimals-not-allowed')),
    ).toBeInTheDocument();
  });

  it('above his limit: the amber line, then F10 asks "is this entry correct?" naming him and his new balance; No saves nothing and goes back to the field, Yes saves with the yes', async () => {
    const { backend, user } = await openWith({
      paid: 25_000,
      carry: 0,
      cut: 0,
      over: 5_000,
      new: { rupees: 35_000, side: 'dr' },
      needsConfirm: true,
    });
    // Like the backend: refused until the user has said yes.
    vi.mocked(bikriApi.bikriSave).mockImplementation((draft, confirm) => {
      if (!confirm) {
        return Promise.reject(
          new CommandError(
            refusal([
              { code: 'ERR_SELLER_PAID_ABOVE_TODAY', field: 'sellers[0].paid', values: {} },
            ]),
          ),
        );
      }
      backend.saves.push({ draft, confirm });
      return Promise.resolve({
        bikri_id: 41,
        number: 1,
        entry_id: 7,
        preview: withSellers(previewFor(draft), []),
      });
    });
    expect(
      within(block()).getByText((text) => visible(text) === m('bikri-seller-paid-above')),
    ).toBeInTheDocument();
    press('F10');
    const box = await screen.findByRole('alertdialog');
    expect(visible(box.textContent)).toContain(m('bikri-seller-over-title'));
    expect(visible(box.textContent)).toContain(names(language).rahim);
    expect(visible(box.textContent)).toContain('35,000');
    // No: nothing was saved and the cursor is in his field.
    await user.click(
      within(box).getByRole('button', {
        name: (name) => visible(name).startsWith(m('common-cash-confirm-no')),
      }),
    );
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    expect(backend.saves).toHaveLength(0);
    await waitFor(() => {
      expect(field()).toHaveFocus();
    });
    // Yes: it saves, with the confirmation.
    press('F10');
    const again = await screen.findByRole('alertdialog');
    await user.click(
      within(again).getByRole('button', {
        name: (name) => visible(name).startsWith(m('common-cash-confirm-yes')),
      }),
    );
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    expect(backend.saves[0]?.confirm).toBe(true);
  });

  it('a lot’s own نقد typed by hand is what the lots hold; the amount follows the sum the backend gives', async () => {
    const { user } = await openWith({ paid: 7_000 });
    await user.click(cellOf(lotsGrid(language), 0, 'cash'));
    await user.keyboard('7000{Enter}');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'cash')).toBe('7,000');
    });
    await waitFor(() => {
      expect(field()).toHaveValue('7,000');
    });
  });

  it('the draft sends the lots’ نقد only: there is no field for the seller’s amount in it', async () => {
    const { backend } = await openWith();
    const draft: BikriDraft = backend.lastDraft();
    expect('seller_paid' in draft).toBe(false);
    expect(draft.lots[0]?.cash_paid).toBe('5000');
  });
});
