// A seller's account in his summary on the بکری page (plan 91, plan 95 item 8, PRD FR-12.23 v1.53, FR-12.52, Umar
// 23-11-2026): the tick and five lines (پچھلا بقایا, آج کی صافی بکری, موقع پر ادائیگی, بکری کی باقی رقم، کھاتے میں جمع, نیا
// بقایا) with their sides, all from the backend's preview. There is NO box to type an amount in the summary: cash to the
// seller is typed in the نقد ادائیگی of his lots, and موقع پر ادائیگی is the sum of those, worked out by the backend. When it
// is more than he may take, an amber line says the balance will grow, and the save asks "is this entry correct?" (the cash
// question's yes), after which a No puts the cursor on the نقد ادائیگی cell. In the Urdu and the English app. The screen
// works out no money: the figures are the fake backend's, set to the plan's worked numbers.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as urduApi from '../../src/shared/urdu/api';
import { previewFor, IDS } from '../helpers/bikri-data';
import {
  cellOf,
  focusedIn,
  lotsGrid,
  names,
  openBook,
  textOf,
  typeEvent1Lot,
} from '../helpers/bikri-screen';
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
import { sellerLines, withSellers } from '../helpers/seller-paid';
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

describe.each(LANGUAGES)(
  'a seller’s summary and the نقد ادائیگی of his lots (%s)',
  (language: Language) => {
    const m = (id: string, vars: Record<string, string | number> = {}) =>
      message(language, id, vars);
    const block = () => {
      const found = document.querySelector<HTMLElement>(`[data-seller-paid="${String(RAHIM)}"]`);
      if (found === null) throw new Error('no seller block');
      return found;
    };
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
      // The title is the seller's name only (plan 97 item 3): no "paid on the spot" words in it.
      expect(visible(block().querySelector('h3')?.textContent)).toBe(names(language).rahim);
      expect(lines()).toEqual([
        { label: m('bikri-seller-paid-previous'), value: `30,000${side('dr')}` },
        { label: m('bikri-seller-paid-net'), value: '20,000' },
        { label: m('bikri-seller-paid-paid'), value: '5,000' },
        { label: m('bikri-seller-paid-rest'), value: '15,000' },
        { label: m('bikri-seller-paid-new'), value: expect.stringContaining('15,000') as string },
      ]);
      expect(lines().at(-1)?.value).toContain(side('dr'));
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

    it('has no box for an amount: the summary only shows the lines, and the cash is typed in the lot’s نقد ادائیگی', async () => {
      await openWith();
      expect(within(block()).queryAllByRole('textbox')).toHaveLength(0);
      // The lot column and the summary line are named نقد ادائیگی.
      expect(
        visible(lotsGrid(language).querySelector('[col-id="cash"].ag-header-cell')?.textContent),
      ).toContain(m('glossary-naqd-adaigi'));
      // The cards stand in a grid that has 2 or 3 columns when the summary is wide and one when it is narrow (item 10).
      expect(block().parentElement?.className).toContain('grid-cols-1');
      expect(block().parentElement?.className).toContain('@form3:grid-cols-2');
      expect(block().parentElement?.className).toContain('@form4:grid-cols-3');
      expect(bikriApi.bikriPreview).toHaveBeenCalled();
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
      // The cursor is on the نقد ادائیگی cell of his lot, where the amount is typed (plan 95 item 8).
      await waitFor(() => {
        expect(focusedIn(lotsGrid(language))).toBe('0:cash');
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

    it('typing the نقد ادائیگی of his lot moves موقع پر ادائیگی and the lines: they follow the sum the backend gives', async () => {
      // A fake backend that takes the paid amount from what is typed in the lots, as the real one does.
      const book = await openBook(language, (draft) => {
        const paid = draft.lots.reduce((sum, lot) => sum + Number(lot.cash_paid ?? '0'), 0);
        return withSellers(previewFor(draft), [
          sellerLines({
            sellerId: RAHIM,
            paid,
            carry: 20_000 - paid,
            cut: 20_000 - paid,
            new: { rupees: 30_000 - 20_000 + paid, side: 'dr' },
          }),
        ]);
      });
      await typeEvent1Lot(book.user, language);
      await waitFor(() => {
        expect(lines().find((line) => line.label === m('bikri-seller-paid-paid'))?.value).toBe(
          '5,000',
        );
      });
      await book.user.click(cellOf(lotsGrid(language), 0, 'cash'));
      await book.user.keyboard('7000{Enter}');
      await waitFor(() => {
        expect(textOf(lotsGrid(language), 0, 'cash')).toBe('7,000');
      });
      await waitFor(() => {
        expect(lines().find((line) => line.label === m('bikri-seller-paid-paid'))?.value).toBe(
          '7,000',
        );
      });
      expect(lines().find((line) => line.label === m('bikri-seller-paid-rest'))?.value).toBe(
        '13,000',
      );
    });

    it('the draft sends the lots’ نقد only: there is no field for the seller’s amount in it', async () => {
      const { backend } = await openWith();
      const draft: BikriDraft = backend.lastDraft();
      expect('seller_paid' in draft).toBe(false);
      expect(draft.lots[0]?.cash_paid).toBe('5000');
    });
  },
);
