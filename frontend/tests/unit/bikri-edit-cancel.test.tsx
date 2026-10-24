// Edit, cancel and undo of a saved بکری (plan 16, PRD 12.9, FR-12.18 to FR-12.22, FR-7.47,
// FR-6.7, E2E-13, E2E-15) in the Urdu and the English app, against a fake backend: the posting is
// loaded as a draft with the ids of its lines, what the user changes is marked and listed in words,
// unchanged lines go back with their ids, an earlier day asks for a reason, a closed day cannot be
// saved, the cancel names what it reverses and asks about cash, and the undo button of the message
// brings the values back in a new entry. The numbers are the backend's; the screen works out none.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { cancelPreview, cash, editable, editableLater, fakePlan16 } from '../helpers/bikri-plan16';
import {
  cellOf,
  focusedIn,
  lotsGrid,
  named,
  openBook,
  salesGrid,
  textOf,
  typeHeaderAndLot,
  typeTwoLines,
} from '../helpers/bikri-screen';
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
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
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

const said = (text: string) => (content: string) => visible(content) === text;
/** A button's name starts with its words; its key follows in a chip ("Save changes F10"). */
const starts = (text: string) => (name: string) => visible(name).startsWith(text);
const pageText = () => visible(document.body.textContent);

describe.each(LANGUAGES)('edit and cancel of a saved بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const names = {
    rahim: language === 'ur' ? 'رحیم' : 'Rahim',
    akram: language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
  };

  /** The saved بکری 41 on screen, read-only. */
  const openSaved = async () => {
    const view = await renderApp('/bikri/book?id=41');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-saved-title', { number: 1 })),
    });
    return { ...view, user: userEvent.setup() };
  };

  /** Posting 7 of بکری 41 open for edit. */
  const openEdit = async () => {
    const view = await renderApp('/bikri/book?id=41&edit=7');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-edit-title', { number: 1 })),
    });
    await waitFor(() => {
      expect(textOf(salesGrid(language), 0, 'buyer')).toBe(names.akram);
    });
    return { ...view, user: userEvent.setup() };
  };

  /** Types a new rate into the first line, the way the munshi does: click the cell, type, Enter. */
  const typeRate = async (user: ReturnType<typeof userEvent.setup>, rate: string) => {
    await user.click(cellOf(salesGrid(language), 0, 'rate'));
    await user.keyboard(`${rate}{Enter}`);
  };

  describe('the saved view', () => {
    it('has Edit and Cancel, and lists each entry with its state and an edit button', async () => {
      fakePlan16();
      await openSaved();
      expect(
        screen.getByRole('button', { name: said(message(language, 'bikri-edit')) }),
      ).toBeEnabled();
      expect(
        screen.getByRole('button', { name: starts(message(language, 'bikri-cancel-bikri')) }),
      ).toBeInTheDocument();
      expect(pageText()).toContain(message(language, 'bikri-posting-open'));
      expect(
        screen.getByRole('button', { name: said(message(language, 'bikri-posting-edit')) }),
      ).toBeInTheDocument();
    });

    it('a بکری undone at once shows the words of its reason, not the id the backend stores', async () => {
      fakePlan16();
      const { savedEvent1 } = await import('../helpers/bikri-saved');
      vi.mocked(bikriApi.bikriGet).mockResolvedValue({
        ...savedEvent1(),
        status: 'cancelled',
        cancel_reason: 'bikri-undo-reason',
      });
      await openSaved();
      expect(pageText()).toContain(
        message(language, 'bikri-saved-cancel-reason', {
          reason: message(language, 'bikri-undo-reason'),
        }),
      );
      expect(pageText()).not.toContain('bikri-undo-reason');
    });

    it('Edit opens the first entry that can be edited', async () => {
      fakePlan16();
      const { router, user } = await openSaved();
      await user.click(screen.getByRole('button', { name: said(message(language, 'bikri-edit')) }));
      await waitFor(() => {
        expect(router.state.location.search).toMatchObject({ id: 41, edit: 7 });
      });
    });

    it('an entry on a closed day cannot be edited, and the list says so in words', async () => {
      fakePlan16();
      vi.mocked(bikriApi.bikriGet).mockResolvedValue({
        ...(await import('../helpers/bikri-saved')).savedEvent1(),
        postings: [
          {
            entry_id: 7,
            posting_type: 'bikri_first',
            entry_date: '2026-10-10',
            total: (await import('../helpers/paisa')).paisaForTest(3_656_400),
            reversed: false,
            editable: false,
          },
        ],
      });
      await openSaved();
      expect(pageText()).toContain(message(language, 'bikri-posting-not-editable'));
      expect(
        screen.getByRole('button', { name: said(message(language, 'bikri-edit')) }),
      ).toBeDisabled();
    });
  });

  describe('editing the first entry', () => {
    it('loads the entry as a draft with the ids of its lot and lines, and asks the backend about it', async () => {
      const plan = fakePlan16();
      await openEdit();
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(names.rahim);
      expect(textOf(lotsGrid(language), 0, 'qty')).toBe('50');
      expect(textOf(salesGrid(language), 0, 'qty')).toBe('20');
      expect(textOf(salesGrid(language), 0, 'rate')).toBe('600');
      await waitFor(() => {
        expect(plan.editPreviews.length).toBeGreaterThan(0);
      });
      const asked = plan.editPreviews.at(-1);
      expect(asked?.entryId).toBe(7);
      expect(asked?.draft.lots[0]).toMatchObject({ key: 'lot:40', lot_id: 40 });
      expect(asked?.draft.lots[0]?.sales[0]).toMatchObject({
        key: 'sale:101',
        sale_id: 101,
        seller_rate: '600',
      });
      // Nothing of a new بکری is touched: no draft is kept for an edit.
      expect(bikriApi.bikriPreview).not.toHaveBeenCalled();
    });

    it('the day of the entry is kept: its date field cannot be changed', async () => {
      fakePlan16();
      await openEdit();
      const date = await screen.findByRole('textbox', {
        name: named(message(language, 'bikri-header-date')),
      });
      expect(date).toHaveAttribute('readonly');
    });

    it('a changed cell is marked and the change is told in words, old text next to the new', async () => {
      fakePlan16();
      const { user } = await openEdit();
      expect(
        screen.getByText(said(message(language, 'bikri-edit-no-changes'))),
      ).toBeInTheDocument();
      await typeRate(user, '610');
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'rate')).toHaveClass('cell-changed');
      });
      // The cell next to it is not marked.
      expect(cellOf(salesGrid(language), 0, 'qty')).not.toHaveClass('cell-changed');
      const rateLabel = message(language, 'bikri-col-rate');
      await waitFor(() => {
        expect(pageText()).toContain(
          message(language, 'bikri-edit-change', {
            where: message(language, 'bikri-edit-where-line', { lot: 1, line: 1 }),
            field: rateLabel,
            before: '600',
            after: '610',
          }),
        );
      });
      // The numbers of the lot before (stored) and after (preview) are the backend's, side by side.
      const card = screen
        .getByRole('heading', { name: named(message(language, 'bikri-edit-changes-title')) })
        .closest('section');
      const cardText = visible(card?.textContent);
      expect(cardText).toContain(message(language, 'bikri-edit-before'));
      expect(cardText).toContain(message(language, 'bikri-edit-after'));
      expect(cardText.split('19,316')).toHaveLength(3);
    });

    it('typing the old value back is no change', async () => {
      fakePlan16();
      const { user } = await openEdit();
      await typeRate(user, '610');
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'rate')).toHaveClass('cell-changed');
      });
      await typeRate(user, '600');
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'rate')).not.toHaveClass('cell-changed');
      });
    });

    it('F10 on a posting of today saves at once: the version, the ids and the unchanged lines go back', async () => {
      const plan = fakePlan16();
      const { router, user } = await openEdit();
      await typeRate(user, '610');
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'rate')).toHaveClass('cell-changed');
      });
      press('F10');
      await waitFor(() => {
        expect(plan.editSaves).toHaveLength(1);
      });
      const saved = plan.editSaves[0];
      expect(saved).toMatchObject({ entryId: 7, version: 3, reason: null, confirm: false });
      const sales = saved?.draft.lots[0]?.sales ?? [];
      // The changed line has its new rate; the unchanged line goes back as loaded, with its id.
      expect(sales[0]).toMatchObject({ sale_id: 101, seller_rate: '610' });
      expect(sales[1]).toMatchObject({ sale_id: 102, seller_rate: '580', qty: '30' });
      await waitFor(() => {
        expect(router.state.location.search).toMatchObject({ id: 41 });
      });
      expect(router.state.location.search).not.toHaveProperty('edit');
      expect(
        await screen.findByText(said(message(language, 'bikri-edit-saved-toast', { number: 1 }))),
      ).toBeInTheDocument();
    });

    it('keyboard only: Enter walks through the loaded cells, a new rate is typed, F10 saves', async () => {
      const plan = fakePlan16();
      const { user } = await openEdit();
      // The cursor starts on the truck. Four Enters leave the header (truck, زمیندار, اڈا, freight).
      await waitFor(() => {
        expect(document.activeElement?.getAttribute('role')).toBe('combobox');
      });
      await user.keyboard('{Enter}{Enter}{Enter}{Enter}');
      await waitFor(() => {
        expect(focusedIn(lotsGrid(language))).toBe('0:seller');
      });
      // The eight stops of the lot, then the first line's buyer, quantity and rate.
      await user.keyboard('{Enter}'.repeat(8));
      await waitFor(() => {
        expect(focusedIn(salesGrid(language))).toBe('0:buyer');
      });
      await user.keyboard('{Enter}{Enter}');
      await waitFor(() => {
        expect(focusedIn(salesGrid(language))).toBe('0:rate');
      });
      await user.keyboard('610{Enter}');
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'rate')).toHaveClass('cell-changed');
      });
      press('F10');
      await waitFor(() => {
        expect(plan.editSaves).toHaveLength(1);
      });
      expect(plan.editSaves[0]?.draft.lots[0]?.sales[0]).toMatchObject({ seller_rate: '610' });
      // The header was not filled from the truck's last sale: nothing else changed.
      expect(plan.editSaves[0]?.draft.header).toMatchObject({ adda_id: null });
    });

    it('an entry of an earlier day asks for a reason before it saves', async () => {
      const plan = fakePlan16({ editable: editable({ entry_date: '2026-10-09' }) });
      const { user } = await openEdit();
      await typeRate(user, '610');
      press('F10');
      const dialog = await screen.findByRole('alertdialog');
      // Nothing is sent without it.
      expect(plan.editSaves).toEqual([]);
      await user.type(
        within(dialog).getByRole('combobox', {
          name: named(message(language, 'bikri-edit-reason-label')),
        }),
        'wrong rate',
      );
      await user.click(
        within(dialog).getByRole('button', {
          name: starts(message(language, 'bikri-edit-reason-confirm')),
        }),
      );
      await waitFor(() => {
        expect(plan.editSaves).toHaveLength(1);
      });
      expect(plan.editSaves[0]?.reason).toBe('wrong rate');
    });

    it('a closed day shows why and cannot be saved', async () => {
      const plan = fakePlan16({ editable: editable({ editable: false }) });
      const { user } = await openEdit();
      expect(pageText()).toContain(message(language, 'bikri-closed-day', { date: '10-10-2026' }));
      expect(
        screen.getByRole('button', { name: starts(message(language, 'bikri-closed-day-open')) }),
      ).toBeInTheDocument();
      await typeRate(user, '610');
      press('F10');
      expect(plan.editSaves).toEqual([]);
      expect(
        screen.getByRole('button', { name: starts(message(language, 'bikri-edit-save')) }),
      ).toBeDisabled();
    });

    it('cash going below zero asks first; Yes saves with the confirmation', async () => {
      const plan = fakePlan16();
      plan.cashConfirmRequired = true;
      vi.mocked(bikriApi.bikriEditPreview).mockImplementation(async (entryId, draft) => {
        plan.editPreviews.push({ entryId, draft });
        const { previewFor } = await import('../helpers/bikri-data');
        return { ...previewFor(draft), cash: cash(1_000, -500, true) };
      });
      const { user } = await openEdit();
      await typeRate(user, '610');
      await screen.findByText(said(message(language, 'common-cash-confirm-title')));
      press('F10');
      await screen.findByRole('alertdialog');
      await user.click(
        screen.getByRole('button', { name: message(language, 'common-cash-confirm-yes') }),
      );
      await waitFor(() => {
        expect(plan.editSaves).toHaveLength(1);
      });
      expect(plan.editSaves[0]?.confirm).toBe(true);
    });

    it('the version having changed is told in words and keeps the edit on screen', async () => {
      fakePlan16();
      const { refused } = await import('../helpers/bikri-plan16');
      vi.mocked(bikriApi.bikriEditSave).mockRejectedValue(
        refused('ERR_VERSION_CHANGED', 'version'),
      );
      const { user } = await openEdit();
      await typeRate(user, '610');
      press('F10');
      expect(
        await screen.findByText(said(message(language, 'err-version-changed'))),
      ).toBeInTheDocument();
      expect(textOf(salesGrid(language), 0, 'rate')).toBe('610');
    });

    it('a language switch in the middle of an edit loses nothing and keeps the marks', async () => {
      fakePlan16();
      const { user } = await openEdit();
      await typeRate(user, '610');
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'rate')).toHaveClass('cell-changed');
      });
      press('KeyL', { altKey: true });
      const other = language === 'ur' ? 'en' : 'ur';
      await waitFor(() => {
        expect(pageText()).toContain(message(other, 'bikri-edit-changes-title'));
      });
      await waitFor(() => {
        expect(textOf(salesGrid(other), 0, 'rate')).toBe('610');
        expect(cellOf(salesGrid(other), 0, 'rate')).toHaveClass('cell-changed');
      });
      expect(pageText()).toContain(
        message(other, 'bikri-edit-change', {
          where: message(other, 'bikri-edit-where-line', { lot: 1, line: 1 }),
          field: message(other, 'bikri-col-rate'),
          before: '600',
          after: '610',
        }),
      );
    });

    it('a saved lot cannot be removed with Ctrl+Delete, a line can', async () => {
      fakePlan16();
      const { user } = await openEdit();
      await user.click(cellOf(lotsGrid(language), 0, 'qty'));
      press('Delete', { ctrlKey: true });
      expect(textOf(lotsGrid(language), 0, 'qty')).toBe('50');
      await user.click(cellOf(salesGrid(language), 1, 'qty'));
      press('Delete', { ctrlKey: true });
      await waitFor(() => {
        expect(pageText()).toContain(
          message(language, 'bikri-edit-removed-line', {
            where: message(language, 'bikri-edit-where-line', { lot: 1, line: 2 }),
            buyer: language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani',
          }),
        );
      });
    });
  });

  describe('editing a later sale', () => {
    const openLater = async () => {
      const view = await renderApp('/bikri/book?id=41&edit=9');
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'bikri-edit-title', { number: 1 })),
      });
      await waitFor(() => {
        expect(textOf(salesGrid(language), 0, 'buyer')).toBe(
          language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani',
        );
      });
      return { ...view, user: userEvent.setup() };
    };

    it('shows the lot and the day read-only and only the lines of that entry', async () => {
      fakePlan16();
      await openLater();
      expect(pageText()).toContain(
        message(language, 'bikri-edit-help-later', { date: '10-10-2026' }),
      );
      expect(pageText()).toContain(names.rahim);
      // One line, loaded as stored; the header and the lots of the sale book are not here.
      expect(textOf(salesGrid(language), 0, 'qty')).toBe('5');
      expect(textOf(salesGrid(language), 0, 'rate')).toBe('560');
      expect(
        screen.queryByRole('group', { name: named(message(language, 'bikri-lots-label')) }),
      ).toBeNull();
    });

    it('a changed rate is marked and listed; F10 saves entry 9 with the ids of its line', async () => {
      const plan = fakePlan16();
      const { user } = await openLater();
      await typeRate(user, '570');
      await waitFor(() => {
        expect(cellOf(salesGrid(language), 0, 'rate')).toHaveClass('cell-changed');
      });
      expect(pageText()).toContain(
        message(language, 'bikri-edit-change', {
          where: message(language, 'bikri-edit-where-line', { lot: 1, line: 1 }),
          field: message(language, 'bikri-col-rate'),
          before: '560',
          after: '570',
        }),
      );
      press('F10');
      await waitFor(() => {
        expect(plan.editSaves).toHaveLength(1);
      });
      expect(plan.editSaves[0]).toMatchObject({
        entryId: 9,
        version: 3,
        reason: null,
        confirm: false,
      });
      expect(plan.editSaves[0]?.draft.lots[0]).toMatchObject({ lot_id: 40 });
      expect(plan.editSaves[0]?.draft.lots[0]?.sales[0]).toMatchObject({
        sale_id: 105,
        seller_rate: '570',
      });
    });

    it('a later entry of an earlier day asks for a reason; a closed day cannot be saved', async () => {
      const plan = fakePlan16({
        editableLater: editableLater({ entry_date: '2026-10-09' }),
      });
      const { user } = await openLater();
      await typeRate(user, '570');
      press('F10');
      const dialog = await screen.findByRole('alertdialog');
      expect(plan.editSaves).toEqual([]);
      await user.type(
        within(dialog).getByRole('combobox', {
          name: named(message(language, 'bikri-edit-reason-label')),
        }),
        'wrong rate',
      );
      await user.click(
        within(dialog).getByRole('button', {
          name: starts(message(language, 'bikri-edit-reason-confirm')),
        }),
      );
      await waitFor(() => {
        expect(plan.editSaves[0]?.reason).toBe('wrong rate');
      });
    });
  });

  describe('cancelling the whole بکری', () => {
    const openCancel = async () => {
      const view = await openSaved();
      await view.user.click(
        screen.getByRole('button', { name: starts(message(language, 'bikri-cancel-bikri')) }),
      );
      const dialog = await screen.findByRole('alertdialog');
      return { ...view, dialog };
    };

    const reasonField = (dialog: HTMLElement) =>
      within(dialog).getByRole('combobox', {
        name: named(message(language, 'bikri-cancel-reason')),
      });

    it('keyboard only: the reason is typed and Enter cancels', async () => {
      const plan = fakePlan16();
      const { dialog, user } = await openCancel();
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain('36,564');
      });
      // Focus starts in the reason field.
      expect(reasonField(dialog)).toHaveFocus();
      await user.keyboard('entered twice{Enter}');
      await waitFor(() => {
        expect(plan.cancels).toHaveLength(1);
      });
      expect(plan.cancels[0]).toMatchObject({ reason: 'entered twice', confirm: false });
    });

    it('names the entries it reverses and needs a reason', async () => {
      const plan = fakePlan16();
      const { dialog, user } = await openCancel();
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain('36,564');
      });
      expect(visible(dialog.textContent)).toContain(message(language, 'bikri-cancel-reverses'));
      expect(visible(dialog.textContent)).toContain('10-10-2026');
      // An empty reason is told at the field and nothing is sent.
      await user.click(
        within(dialog).getByRole('button', {
          name: starts(message(language, 'bikri-cancel-confirm')),
        }),
      );
      expect(
        await within(dialog).findByText(said(message(language, 'err-required'))),
      ).toBeInTheDocument();
      expect(plan.cancels).toEqual([]);
      await user.type(reasonField(dialog), 'entered twice');
      await user.click(
        within(dialog).getByRole('button', {
          name: starts(message(language, 'bikri-cancel-confirm')),
        }),
      );
      await waitFor(() => {
        expect(plan.cancels).toEqual([
          { id: 41, version: 3, reason: 'entered twice', confirm: false },
        ]);
      });
      expect(
        await screen.findByText(said(message(language, 'bikri-cancelled-toast', { number: 1 }))),
      ).toBeInTheDocument();
    });

    it('shows the cash line when cash changes', async () => {
      fakePlan16({ cancelPreview: cancelPreview({ cash: cash(58_484, 45_684) }) });
      const { dialog } = await openCancel();
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain('58,484');
      });
      expect(visible(dialog.textContent)).toContain('45,684');
    });

    it('a closed day is told, the button is dimmed and nothing can be sent', async () => {
      const plan = fakePlan16({
        cancelPreview: cancelPreview({ closed_dates: ['2026-10-10'], can_cancel: false }),
      });
      const { dialog } = await openCancel();
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain(
          message(language, 'bikri-closed-day', { date: '10-10-2026' }),
        );
      });
      expect(
        within(dialog).getByRole('button', {
          name: starts(message(language, 'bikri-cancel-confirm')),
        }),
      ).toBeDisabled();
      expect(plan.cancels).toEqual([]);
    });

    it('cash going below zero asks after the reason; No comes back to the reason, Yes cancels', async () => {
      const plan = fakePlan16({
        cancelPreview: cancelPreview({ cash: cash(1_000, -500, true) }),
      });
      const { dialog, user } = await openCancel();
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain('36,564');
      });
      await user.type(reasonField(dialog), 'entered twice');
      await user.click(
        within(dialog).getByRole('button', {
          name: starts(message(language, 'bikri-cancel-confirm')),
        }),
      );
      const cashBox = await screen.findByRole('alertdialog', {
        name: said(message(language, 'common-cash-confirm-title')),
      });
      expect(visible(cashBox.textContent)).toContain('-500');
      // No: back to the reason, with what was typed, and nothing is cancelled.
      await user.click(
        within(cashBox).getByRole('button', {
          name: starts(message(language, 'common-cash-confirm-no')),
        }),
      );
      const again = await screen.findByRole('alertdialog', {
        name: said(message(language, 'bikri-cancel-title')),
      });
      expect(reasonField(again)).toHaveValue('entered twice');
      expect(plan.cancels).toEqual([]);
      await user.click(
        within(again).getByRole('button', {
          name: starts(message(language, 'bikri-cancel-confirm')),
        }),
      );
      const second = await screen.findByRole('alertdialog', {
        name: said(message(language, 'common-cash-confirm-title')),
      });
      await user.click(
        within(second).getByRole('button', {
          name: starts(message(language, 'common-cash-confirm-yes')),
        }),
      );
      await waitFor(() => {
        expect(plan.cancels).toEqual([
          { id: 41, version: 3, reason: 'entered twice', confirm: true },
        ]);
      });
    });
  });

  describe('undo straight after a save', () => {
    /** Event 1 typed and saved with F10, so the message with the undo button is on screen. */
    const saveEvent1 = async () => {
      const plan = fakePlan16();
      const { user } = await openBook(language);
      await typeHeaderAndLot(user, language);
      await typeTwoLines(user, language);
      await waitFor(() => {
        expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
      });
      press('F10');
      await waitFor(() => {
        expect(pageText()).toContain(message(language, 'bikri-saved-toast', { number: 1 }));
      });
      return { plan, user };
    };

    it('the message has an Undo button with its key, and Alt+Z brings the values back in a new entry', async () => {
      const { plan } = await saveEvent1();
      const button = await screen.findByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'bikri-undo')),
      });
      expect(visible(button.textContent)).toContain('Alt+Z');
      press('KeyZ', { altKey: true });
      await waitFor(() => {
        expect(plan.undos).toEqual([{ id: 41, confirm: false }]);
      });
      // A new unsaved entry holds the same values, to correct and save again.
      await waitFor(() => {
        expect(textOf(lotsGrid(language), 0, 'seller')).toBe(names.rahim);
      });
      expect(textOf(lotsGrid(language), 0, 'qty')).toBe('50');
      expect(textOf(salesGrid(language), 0, 'buyer')).toBe(names.akram);
      expect(textOf(salesGrid(language), 0, 'rate')).toBe('600');
      expect(pageText()).toContain(message(language, 'bikri-undo-done', { number: 1 }));
    });

    it('the button does the same as the key', async () => {
      const { plan, user } = await saveEvent1();
      const button = await screen.findByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'bikri-undo')),
      });
      await user.click(button);
      await waitFor(() => {
        expect(plan.undos).toHaveLength(1);
      });
    });

    it('cash going below zero asks first', async () => {
      const { plan, user } = await saveEvent1();
      vi.mocked(bikriApi.bikriCancelPreview).mockResolvedValue(
        cancelPreview({ cash: cash(1_000, -500, true) }),
      );
      plan.cashConfirmRequired = true;
      press('KeyZ', { altKey: true });
      const box = await screen.findByRole('alertdialog');
      expect(plan.undos).toEqual([]);
      await user.click(
        within(box).getByRole('button', {
          name: starts(message(language, 'common-cash-confirm-yes')),
        }),
      );
      await waitFor(() => {
        expect(plan.undos).toEqual([{ id: 41, confirm: true }]);
      });
    });

    it('after leaving the sale book the cash question still opens, on the screen he is on', async () => {
      const { router, plan, user } = await (async () => {
        const plan = fakePlan16();
        const { router, user } = await openBook(language);
        await typeHeaderAndLot(user, language);
        await typeTwoLines(user, language);
        await waitFor(() => {
          expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
        });
        press('F10');
        await waitFor(() => {
          expect(pageText()).toContain(message(language, 'bikri-saved-toast', { number: 1 }));
        });
        return { router, plan, user };
      })();
      vi.mocked(bikriApi.bikriCancelPreview).mockResolvedValue(
        cancelPreview({ cash: cash(1_000, -500, true) }),
      );
      plan.cashConfirmRequired = true;
      // He goes to the dashboard (nothing typed, so nothing asks) and presses Alt+Z there.
      await router.navigate({ to: '/' });
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/');
      });
      press('KeyZ', { altKey: true });
      const box = await screen.findByRole('alertdialog');
      expect(visible(box.textContent)).toContain('-500');
      expect(plan.undos).toEqual([]);
      await user.click(
        within(box).getByRole('button', {
          name: said(message(language, 'common-cash-confirm-yes')),
        }),
      );
      await waitFor(() => {
        expect(plan.undos).toEqual([{ id: 41, confirm: true }]);
      });
      // The values come back in a new entry: the sale book opens on them.
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/book');
      });
      await waitFor(() => {
        expect(textOf(salesGrid(language), 0, 'rate')).toBe('600');
      });
    });

    it('a refusal of the undo is told in the backend words, never in silence', async () => {
      const { plan } = await saveEvent1();
      const { refused } = await import('../helpers/bikri-plan16');
      vi.mocked(bikriApi.bikriUndo).mockRejectedValue(
        refused('ERR_DATE_CLOSED', 'date', { closed_up_to: '10-10-2026' }),
      );
      press('KeyZ', { altKey: true });
      expect(
        await screen.findByText(
          said(message(language, 'err-date-closed', { closed_up_to: '10-10-2026' })),
        ),
      ).toBeInTheDocument();
      expect(plan.undos).toEqual([]);
    });

    /** Event 1 saved, then he goes to the dashboard: the sale book is no longer on screen. */
    const saveThenLeave = async () => {
      const plan = fakePlan16();
      const { router, user } = await openBook(language);
      await typeHeaderAndLot(user, language);
      await typeTwoLines(user, language);
      await waitFor(() => {
        expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
      });
      press('F10');
      await waitFor(() => {
        expect(pageText()).toContain(message(language, 'bikri-saved-toast', { number: 1 }));
      });
      await router.navigate({ to: '/' });
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/');
      });
      return { plan, router, user };
    };

    it('after leaving the sale book, if the cash changed since the question, it is asked again with the new cash', async () => {
      const { plan, user } = await saveThenLeave();
      // The first answer said no question was needed; the undo is then refused for cash; the
      // second answer has the cash as it is now.
      vi.mocked(bikriApi.bikriCancelPreview)
        .mockResolvedValueOnce(cancelPreview({ cash: cash(1_000, 900) }))
        .mockResolvedValue(cancelPreview({ cash: cash(1_000, -500, true) }));
      plan.cashConfirmRequired = true;
      press('KeyZ', { altKey: true });
      const box = await screen.findByRole('alertdialog');
      expect(visible(box.textContent)).toContain('-500');
      expect(plan.undos).toEqual([]);
      await user.click(
        within(box).getByRole('button', {
          name: said(message(language, 'common-cash-confirm-yes')),
        }),
      );
      await waitFor(() => {
        expect(plan.undos).toEqual([{ id: 41, confirm: true }]);
      });
    });

    it('after leaving the sale book, a cash refusal that cannot be asked again is told in words with the cash after', async () => {
      const { plan } = await saveThenLeave();
      const { refused } = await import('../helpers/bikri-plan16');
      vi.mocked(bikriApi.bikriCancelPreview).mockResolvedValue(
        cancelPreview({ cash: cash(1_000, 900) }),
      );
      vi.mocked(bikriApi.bikriUndo).mockRejectedValue(
        refused('ERR_CASH_BELOW_ZERO_NOT_CONFIRMED', null, { cash_after_paisa: -50_000 }),
      );
      press('KeyZ', { altKey: true });
      expect(
        await screen.findByText(
          said(message(language, 'err-cash-below-zero-not-confirmed', { cash_after: '-500' })),
        ),
      ).toBeInTheDocument();
      expect(plan.undos).toEqual([]);
    });

    it('if the cancel preview cannot be read, he is told and nothing is undone', async () => {
      const { plan } = await saveEvent1();
      const { refused } = await import('../helpers/bikri-plan16');
      vi.mocked(bikriApi.bikriCancelPreview).mockRejectedValue(
        refused('ERR_BIKRI_CANCELLED', 'id'),
      );
      press('KeyZ', { altKey: true });
      expect(
        await screen.findByText(said(message(language, 'err-bikri-cancelled'))),
      ).toBeInTheDocument();
      expect(plan.undos).toEqual([]);
    });

    it('another entry already started is never replaced: he is told, nothing is undone', async () => {
      const { plan, user } = await saveEvent1();
      const truck = await screen.findByRole('combobox', {
        name: named(message(language, 'bikri-header-truck')),
      });
      await user.type(truck, 'LHR-77');
      press('KeyZ', { altKey: true });
      expect(
        await screen.findByText(said(message(language, 'bikri-undo-busy'))),
      ).toBeInTheDocument();
      expect(plan.undos).toEqual([]);
      expect(truck).toHaveValue('LHR-77');
    });
  });
});
