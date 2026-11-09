// Find a saved بکری by each filter, and correct a wrong buyer or seller in an edit (plan 67, PRD
// FR-12.13, FR-7.70, D8 Feature 7, E2E-68), in the Urdu and the English app, against a fake backend.
//
// Find: number, truck, seller, buyer (an account from the picker, or the name of a walk-in چکتا
// buyer typed as text), day and item, each sent to `bikri_find` as the backend reads it. Edit: the
// buyer cell of a line and the seller cell of a lot can be changed in an edit; the change is marked
// and told in words; the draft the backend previews and the draft that is saved carry the new
// party; the review of the backend (a canned one here, with the numbers of E2E-68) shows both
// balances. The lock of a lot's seller when a later sale rests on it stays, and the screen says why.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { EditPreview } from '../../src/bindings/EditPreview';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch } from '../helpers/accounts';
import {
  AKRAM,
  APPLE,
  ARSHAD,
  BANANA,
  CHUKTA_KHATA,
  KARIM,
  RAHIM,
  fakeBikri,
} from '../helpers/bikri';
import { editable, fakePlan16 } from '../helpers/bikri-plan16';
import { CommandError } from '../../src/shared/api';
import { refusal } from '../helpers/ui-error';
import { NASIR, SHAHID, listRow } from '../helpers/bikri-saved';
import { cellOf, lotsGrid, named, salesGrid, textOf, typeAndPick } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { EMPTY_PAIRS } from '../helpers/paged';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/edit/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

/** The buyer of the worked day's change (E2E-68): a new regular buyer, طارق, code 0022. */
const TARIQ = account(23, 'buyer', '0022', 'طارق', 'Tariq');

const pageText = () => visible(document.body.textContent);

describe.each(LANGUAGES)('find and edit by party (%s)', (language) => {
  const m = (id: string, args: Record<string, string | number> = {}) => message(language, id, args);
  const nameOf = {
    akram: language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
    tariq: language === 'ur' ? 'طارق' : 'Tariq',
    karim: language === 'ur' ? 'کریم' : 'Karim',
    rahim: language === 'ur' ? 'رحیم' : 'Rahim',
  };

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    // The new buyer is in the lists as well.
    fakeAccountSearch([RAHIM, KARIM, AKRAM, ARSHAD, TARIQ, CHUKTA_KHATA], {}, [SHAHID, NASIR]);
  });

  describe('find (F3): every filter is sent as the backend reads it', () => {
    const openFind = async () => {
      const view = await renderApp('/bikri/book?find=true');
      const dialog = await screen.findByRole('dialog');
      return { ...view, dialog, user: userEvent.setup() };
    };
    const lastFind = () => vi.mocked(bikriApi.bikriFind).mock.lastCall?.[0];
    const NOTHING = {
      number: null,
      truck_no: null,
      seller_id: null,
      date: null,
      item_id: null,
      buyer_account_id: null,
      buyer_text: null,
    };

    beforeEach(() => {
      vi.mocked(bikriApi.bikriFind).mockResolvedValue([listRow({ id: 33, number: 1 })]);
    });

    it('a buyer picked from the list is sent as his account, and the matches show', async () => {
      const { dialog, user } = await openFind();
      await user.click(within(dialog).getByRole('combobox', { name: named(m('glossary-gahak')) }));
      await typeAndPick(user, 'akram');
      await waitFor(() => {
        expect(lastFind()).toEqual({ ...NOTHING, buyer_account_id: AKRAM.id });
      });
      // The hint is gone: a buyer alone is a filter.
      await waitFor(() => {
        expect(visible(dialog.textContent)).not.toContain(m('bikri-find-hint'));
      });
      expect(
        await within(dialog).findByRole('button', {
          name: (name) => visible(name).includes(m('bikri-hit-title', { number: 1 })),
        }),
      ).toBeInTheDocument();
    });

    it('the name of a walk-in چکتا buyer is sent as text, never as an account', async () => {
      const { dialog, user } = await openFind();
      await user.type(
        within(dialog).getByRole('textbox', { name: named(m('bikri-find-walk-in')) }),
        'shahid',
      );
      await waitFor(() => {
        expect(lastFind()).toEqual({ ...NOTHING, buyer_text: 'shahid' });
      });
    });

    it('the buyer, the walk-in name and the other filters go together', async () => {
      const { dialog, user } = await openFind();
      await user.click(
        within(dialog).getByRole('combobox', { name: named(m('glossary-beopari')) }),
      );
      await typeAndPick(user, 'rahim');
      await user.click(within(dialog).getByRole('combobox', { name: named(m('glossary-gahak')) }));
      await typeAndPick(user, 'akram');
      await user.type(
        within(dialog).getByRole('textbox', { name: named(m('bikri-header-date')) }),
        '10-10-2026',
      );
      await user.click(
        within(dialog).getByRole('combobox', { name: named(m('glossary-qism-maal')) }),
      );
      await typeAndPick(user, 'banana');
      await waitFor(() => {
        expect(lastFind()).toEqual({
          ...NOTHING,
          seller_id: RAHIM.id,
          buyer_account_id: AKRAM.id,
          date: '10-10-2026',
          item_id: BANANA.id,
        });
      });
      // An item that is not typed any more is sent as nothing again (the other item is not Banana).
      expect(APPLE.id).not.toBe(BANANA.id);
    });

    it('emptying the buyer box asks for no buyer again', async () => {
      const { dialog, user } = await openFind();
      await user.type(
        within(dialog).getByRole('textbox', { name: named(m('bikri-find-walk-in')) }),
        'sh',
      );
      await waitFor(() => {
        expect(lastFind()).toEqual({ ...NOTHING, buyer_text: 'sh' });
      });
      await user.clear(
        within(dialog).getByRole('textbox', { name: named(m('bikri-find-walk-in')) }),
      );
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain(m('bikri-find-hint'));
      });
    });
  });

  describe('edit: a wrong buyer or seller is corrected on the entry', () => {
    /** Posting 7 of بکری 41 open for edit, with طارق in the buyer list. */
    const openEdit = async (options: Parameters<typeof fakePlan16>[0] = {}) => {
      const plan = fakePlan16(options);
      const view = await renderApp('/bikri/book?id=41&edit=7');
      await screen.findByRole('heading', {
        level: 1,
        name: named(m('bikri-edit-title', { number: 1 })),
      });
      await waitFor(() => {
        expect(textOf(salesGrid(language), 0, 'buyer')).toBe(nameOf.akram);
      });
      return { ...view, plan, user: userEvent.setup() };
    };

    /** The review of the backend for line 1 moved from اکرم to طارق: the numbers of E2E-68. */
    const buyerReview = (): EditPreview => ({
      errors: [],
      review: {
        entry_date: '2026-10-10',
        changes: [
          {
            place: { kind: 'line', lot_no: 1, line_no: 1, label_ur: null, label_en: null },
            field: 'buyer',
            before: { type: 'names', ur: AKRAM.name_ur, en: AKRAM.name_en },
            after: { type: 'names', ur: TARIQ.name_ur, en: TARIQ.name_en },
          },
        ],
        effect: {
          parties: [
            {
              account: {
                id: AKRAM.id,
                code: AKRAM.code,
                name_ur: AKRAM.name_ur,
                name_en: AKRAM.name_en,
              },
              before: { amount_paisa: paisaForTest(288_000), side: 'dr' },
              after: { amount_paisa: paisaForTest(1_000_000), side: 'cr' },
            },
            {
              account: {
                id: TARIQ.id,
                code: TARIQ.code,
                name_ur: TARIQ.name_ur,
                name_en: TARIQ.name_en,
              },
              before: { amount_paisa: paisaForTest(0), side: 'zero' },
              after: { amount_paisa: paisaForTest(1_288_000), side: 'dr' },
            },
          ],
          cash: null,
          partners: [],
          partners_locked: false,
        },
        reason_required: false,
        needs_reopen: false,
      },
    });

    it('the buyer cell of a line can be changed: it is marked, told in words, and the new buyer is previewed and saved', async () => {
      const { plan, user } = await openEdit();
      vi.mocked(bikriApi.bikriEditReview).mockResolvedValue(buyerReview());
      await user.click(cellOf(salesGrid(language), 0, 'buyer'));
      await typeAndPick(user, 'tariq');
      await waitFor(() => {
        expect(textOf(salesGrid(language), 0, 'buyer')).toBe(nameOf.tariq);
      });
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'buyer')).toHaveClass('cell-changed');
      });
      // The cells of the other line and the other cells of this one are not marked.
      expect(cellOf(salesGrid(language), 1, 'buyer')).not.toHaveClass('cell-changed');
      expect(cellOf(salesGrid(language), 0, 'qty')).not.toHaveClass('cell-changed');
      // "What you changed": the old buyer next to the new one, in words.
      await waitFor(() => {
        expect(pageText()).toContain(
          m('edit-review-change', {
            where: `${m('edit-review-where-line', { lot: 1, line: 1, label: '' })} `,
            field: m('edit-field-buyer'),
            before: nameOf.akram,
            after: nameOf.tariq,
          }),
        );
      });
      // The backend is asked about the edited draft with the new buyer on the same line.
      await waitFor(() => {
        expect(plan.editPreviews.at(-1)?.draft.lots[0]?.sales[0]).toMatchObject({
          sale_id: 101,
          buyer_account_id: TARIQ.id,
        });
      });
      // The review shows both parties: اکرم now has 10,000 Credit left over, طارق owes 12,880.
      await waitFor(() => {
        expect(pageText()).toContain('10,000');
      });
      expect(pageText()).toContain('12,880');
      expect(pageText()).toContain(nameOf.tariq);
      // F10 saves the corrected buyer with the line's own id, and the other line as it was loaded.
      press('F10');
      await waitFor(() => {
        expect(plan.editSaves).toHaveLength(1);
      });
      const sales = plan.editSaves[0]?.draft.lots[0]?.sales ?? [];
      expect(sales[0]).toMatchObject({ sale_id: 101, buyer_account_id: TARIQ.id });
      expect(sales[1]).toMatchObject({ sale_id: 102, buyer_account_id: ARSHAD.id });
    });

    it('changing the buyer back is no change, and the save goes back to the first buyer', async () => {
      const { plan, user } = await openEdit();
      await user.click(cellOf(salesGrid(language), 0, 'buyer'));
      await typeAndPick(user, 'tariq');
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'buyer')).toHaveClass('cell-changed');
      });
      await user.click(cellOf(salesGrid(language), 0, 'buyer'));
      await typeAndPick(user, 'akram');
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'buyer')).not.toHaveClass('cell-changed');
      });
      await waitFor(() => {
        expect(plan.editPreviews.at(-1)?.draft.lots[0]?.sales[0]).toMatchObject({
          buyer_account_id: AKRAM.id,
        });
      });
    });

    it('the seller cell of a lot can be changed, and the new seller is previewed and saved', async () => {
      const { plan, user } = await openEdit();
      // The review of the backend lists the change of the lot's seller.
      vi.mocked(bikriApi.bikriEditReview).mockResolvedValue({
        errors: [],
        review: {
          entry_date: '2026-10-10',
          changes: [
            {
              place: { kind: 'lot', lot_no: 1, line_no: null, label_ur: null, label_en: null },
              field: 'seller',
              before: { type: 'names', ur: RAHIM.name_ur, en: RAHIM.name_en },
              after: { type: 'names', ur: KARIM.name_ur, en: KARIM.name_en },
            },
          ],
          effect: { parties: [], cash: null, partners: [], partners_locked: false },
          reason_required: false,
          needs_reopen: false,
        },
      });
      await user.click(cellOf(lotsGrid(language), 0, 'seller'));
      await typeAndPick(user, 'karim');
      await waitFor(() => {
        expect(textOf(lotsGrid(language), 0, 'seller')).toBe(nameOf.karim);
      });
      await waitFor(() => {
        expect(cellOf(lotsGrid(language), 0, 'seller')).toHaveClass('cell-changed');
      });
      // "What you changed" tells it in words: the field, the old seller and the new one.
      await waitFor(() => {
        const told = Array.from(document.querySelectorAll('li')).map((item) =>
          visible(item.textContent),
        );
        expect(
          told.some(
            (line) =>
              line.includes(m('edit-field-seller')) &&
              line.includes(nameOf.rahim) &&
              line.includes(nameOf.karim),
          ),
        ).toBe(true);
      });
      press('F10');
      await waitFor(() => {
        expect(plan.editSaves).toHaveLength(1);
      });
      expect(plan.editSaves[0]?.draft.lots[0]).toMatchObject({
        lot_id: 40,
        seller_account_id: KARIM.id,
      });
    });

    it('a seller that a later sale rests on cannot be changed: the screen says why, at the seller', async () => {
      const { user } = await openEdit();
      vi.mocked(bikriApi.bikriEditPreview).mockRejectedValue(
        new CommandError(
          refusal([
            {
              code: 'ERR_EDIT_NOT_ALLOWED',
              field: 'lots[0].seller_account_id',
              values: { rule: 'lot_seller_locked', entry: 7 },
            },
          ]),
        ),
      );
      await user.click(cellOf(lotsGrid(language), 0, 'seller'));
      await typeAndPick(user, 'karim');
      await waitFor(() => {
        expect(pageText()).toContain(m('err-edit-not-allowed', { rule: 'lot_seller_locked' }));
      });
    });

    it('a closed day cannot be saved with a new buyer either', async () => {
      const { plan, user } = await openEdit({ editable: editable({ editable: false }) });
      await user.click(cellOf(salesGrid(language), 0, 'buyer'));
      await typeAndPick(user, 'tariq');
      press('F10');
      expect(plan.editSaves).toEqual([]);
    });
  });
});
