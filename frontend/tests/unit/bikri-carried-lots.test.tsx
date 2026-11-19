// Lots carried over from earlier بکری and new lots on one screen (plan 76 point 1, PRD FR-12.17,
// FR-12.48, E2E-16b) in the Urdu and the English app, against a fake backend. The old address of a later
// sale opens the sale book with the lot already on it; the lot's cells are shown, not typed; the new
// lines are typed with the keyboard only; the preview and the save go to the mixed commands with the
// text typed (and the plain commands are not used); new lots can be added beside the carried one and
// go in the same save; the cash box asks once for all of it; a refusal on a carried line lands on that
// line; one undo takes the whole save back; a language switch in the middle loses nothing (F39). The
// numbers are the backend's; the screen works out none.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { forgetMixedSaves, mixedSaveOf } from '../../src/features/bikri/print/mixed-save-ids';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { settlePreview } from '../helpers/preview-settled';
import { cash, fakePlan16, mixedAnswer, refused, stockRow } from '../helpers/bikri-plan16';
import {
  cellOf,
  focusedIn,
  lotsGrid,
  named,
  salesGrid,
  textOf,
  typeCodeAndEnter,
  waitForLineFill,
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
import { IDS } from '../helpers/bikri-data';
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

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  forgetMixedSaves();
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;

describe.each(LANGUAGES)('lots carried over onto a new بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const names = {
    rahim: language === 'ur' ? 'رحیم' : 'Rahim',
    akram: language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
    karim: language === 'ur' ? 'کریم' : 'Karim',
  };

  /** The old address of a later sale: it opens the sale book with lot 40 on it. */
  const open = async () => {
    const view = await renderApp('/bikri/later?lot=40');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(names.rahim);
    });
    return { ...view, user: userEvent.setup() };
  };

  /** Types the one new line of E2E-16b: Akram, the 5 left (filled by the app), at 560. */
  const typeLine = async (user: ReturnType<typeof userEvent.setup>) => {
    await waitForLineFill(language);
    // The cursor is already on the buyer of the lot's first line: the lot was put on the screen with it
    // there. A short code (Akram is the first buyer, running number 0), then qty, rate, paid on the spot.
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:buyer');
    });
    await typeCodeAndEnter(user, '0');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
    });
    // Quantity (the app filled it), rate 560, then the paid on the spot cell, left empty.
    await user.keyboard('{Enter}560{Enter}{Enter}');
  };

  it('the old address of a later sale opens the sale book with the lot on it, and says so', async () => {
    const plan = fakePlan16();
    const { router } = await open();
    expect(router.state.location.pathname).toBe('/bikri/book');
    // The request is taken out of the address once the lot is on the screen.
    await waitFor(() => {
      expect(router.state.location.search).not.toHaveProperty('carry');
    });
    expect(visible(document.body.textContent)).toContain(
      message(language, 'bikri-carried-note', { count: 1 }),
    );
    // The lot is read by its id alone, and the screen asks the mixed preview, never the plain one.
    expect(vi.mocked(bikriApi.bikriStockList).mock.calls[0]?.[0]).toMatchObject({ lot_id: 40 });
    await waitFor(() => {
      expect(plan.mixedPreviews.length).toBeGreaterThan(0);
    });
    expect(plan.mixedPreviews[0]).toMatchObject({
      date: '10-10-2026',
      carried: [{ lot_id: 40 }],
      new_lots: [],
    });
    expect(bikriApi.bikriPreview).not.toHaveBeenCalled();
  });

  it('shows the lot and does not let its cells be typed in; the new line starts with what is left', async () => {
    fakePlan16();
    const { user } = await open();
    await waitForLineFill(language);
    expect(textOf(salesGrid(language), 0, 'qty')).toBe('5');
    expect(textOf(salesGrid(language), 0, 'rate')).toBe('');
    // The quantity received is shown (50) and a key does not change it.
    expect(textOf(lotsGrid(language), 0, 'qty')).toBe('50');
    await user.click(cellOf(lotsGrid(language), 0, 'qty'));
    await user.keyboard('9');
    expect(textOf(lotsGrid(language), 0, 'qty')).toBe('50');
  });

  it('keyboard only: buyer, qty, rate, then F10 saves the carried lot with the text as typed', async () => {
    const plan = fakePlan16();
    const { user } = await open();
    await typeLine(user);
    await waitFor(() => {
      expect(plan.mixedPreviews.at(-1)?.carried[0]?.sales).toEqual([
        expect.objectContaining({ buyer_account_id: IDS.akram, qty: '5', seller_rate: '560' }),
        expect.anything(),
      ]);
    });
    await settlePreview();
    press('F10');
    await waitFor(() => {
      expect(plan.mixedSaves).toHaveLength(1);
    });
    const saved = plan.mixedSaves[0];
    expect(saved?.confirm).toBe(false);
    expect(saved?.input).toMatchObject({ date: '10-10-2026', new_lots: [] });
    expect(saved?.input.carried[0]).toMatchObject({ lot_id: 40 });
    expect(saved?.input.carried[0]?.sales[0]).toMatchObject({
      buyer_account_id: IDS.akram,
      qty: '5',
      seller_rate: '560',
    });
    // The plain save was never used, and the message names the old بکری (no new one was made).
    expect(bikriApi.bikriSave).not.toHaveBeenCalled();
    expect(
      await screen.findByText(said(message(language, 'bikri-mixed-saved-toast', { number: 1 }))),
    ).toBeInTheDocument();
  });

  it('a new lot can be added beside the carried one, and both go in one save', async () => {
    const plan = fakePlan16();
    const { user } = await open();
    await typeLine(user);
    // The empty lot row under the carried lot is the place for a new lot: its cursor starts on the seller.
    // Karim by his short code, then the lot.
    await user.click(cellOf(lotsGrid(language), 1, 'seller'));
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).toBe('1:seller');
    });
    await typeCodeAndEnter(user, '1');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 1, 'seller')).toBe(names.karim);
    });
    // His item and packing come from his last lot (filled a moment after he is chosen).
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 1, 'packing')).not.toBe('');
    });
    await user.keyboard('{Enter}{Enter}10{Enter}1000{Enter}');
    await settlePreview();
    press('F10');
    await waitFor(() => {
      expect(plan.mixedSaves).toHaveLength(1);
    });
    const input = plan.mixedSaves[0]?.input;
    // The carried lot first, the new lot after it, as the backend lists them.
    expect(input?.carried).toHaveLength(1);
    expect(input?.new_lots).toHaveLength(1);
    expect(input?.new_lots[0]).toMatchObject({ seller_account_id: IDS.karim, qty_received: '10' });
    expect(
      await screen.findByText(said(message(language, 'bikri-mixed-saved-toast', { number: 2 }))),
    ).toBeInTheDocument();
    // The print box can then print the whole save: the new بکری 42 and the old بکری 41 of the carried lot, as
    // the backend listed them in the answer of the save (`print_bikri_ids`).
    expect(mixedSaveOf(42)).toEqual({ ids: [42, 41], numbers: [2, 1] });
  });

  it('cash going below zero asks once for the whole save: No saves nothing, Yes saves with the confirmation', async () => {
    const plan = fakePlan16();
    plan.cashConfirmRequired = true;
    vi.mocked(bikriApi.bikriMixedPreview).mockImplementation((input) => {
      plan.mixedPreviews.push(structuredClone(input));
      const base = mixedAnswer(input, 5);
      return Promise.resolve({
        ...base,
        preview: { ...base.preview, cash: cash(1_000, -500, true) },
      });
    });
    const { user } = await open();
    await typeLine(user);
    await screen.findByText(said(message(language, 'common-cash-confirm-title')));
    press('F10');
    const box = await screen.findByRole('alertdialog');
    expect(visible(box.textContent)).toContain('-500');
    // The focus starts on "No": Enter out of habit saves nothing.
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    expect(plan.mixedSaves).toEqual([]);
    press('F10');
    await screen.findByRole('alertdialog');
    await user.click(
      screen.getByRole('button', { name: message(language, 'common-cash-confirm-yes') }),
    );
    await waitFor(() => {
      expect(plan.mixedSaves).toHaveLength(1);
    });
    expect(plan.mixedSaves[0]?.confirm).toBe(true);
  });

  it('a refusal on a carried line is told in words, lands on that line and keeps what was typed', async () => {
    fakePlan16();
    vi.mocked(bikriApi.bikriMixedSave).mockRejectedValue(
      refused('ERR_QTY_EXCEEDS_LOT', 'carried[0].sales[0].qty', { available: 3 }),
    );
    const { user } = await open();
    await typeLine(user);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'qty')).not.toBe('');
    });
    await settlePreview();
    press('F10');
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'err-qty-exceeds-lot', { available: 3 }),
      );
    });
    expect(textOf(salesGrid(language), 0, 'rate')).toBe('560');
  });

  it('one undo takes the whole save back and brings the carried lot and its line back on a new entry', async () => {
    const plan = fakePlan16();
    const { user } = await open();
    await typeLine(user);
    await settlePreview();
    press('F10');
    await waitFor(() => {
      expect(plan.mixedSaves).toHaveLength(1);
    });
    const button = await screen.findByRole('button', {
      name: (label) => visible(label).startsWith(message(language, 'bikri-undo')),
    });
    await user.click(button);
    await waitFor(() => {
      expect(plan.mixedUndos).toEqual([
        { later_entry_ids: [50], new_bikri_id: null, confirm_cash_below_zero: false },
      ]);
    });
    // The lot is on the screen again with the line that was typed, to correct and save once more.
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(names.rahim);
    });
    expect(textOf(salesGrid(language), 0, 'buyer')).toBe(names.akram);
    expect(textOf(salesGrid(language), 0, 'rate')).toBe('560');
    expect(bikriApi.bikriUndo).not.toHaveBeenCalled();
  });

  // The undo of a mixed save asks the cash question inside the backend: it answers ERR_CASH_BELOW_ZERO_NOT_CONFIRMED
  // with the cash after, and the box "is this entry correct?" opens with that figure; the yes sends the undo
  // again with the confirmation (PRD FR-6.7, FR-7.47; plan 80 R2-1). Cash after the undo here: -2,000.
  describe('an undo of the whole save that would take cash below zero', () => {
    /** Saves the mixed entry, then makes the undo refuse until it is confirmed. Returns the plan and the user. */
    const saveThenRefuseUndo = async (values: Record<string, string | number>) => {
      const plan = fakePlan16();
      const confirmed = vi.mocked(bikriApi.bikriMixedUndo).getMockImplementation();
      if (confirmed === undefined) throw new Error('the double has no undo');
      vi.mocked(bikriApi.bikriMixedUndo).mockImplementation((input) =>
        input.confirm_cash_below_zero
          ? confirmed(input)
          : Promise.reject(refused('ERR_CASH_BELOW_ZERO_NOT_CONFIRMED', null, values)),
      );
      const { user } = await open();
      await typeLine(user);
      await settlePreview();
      press('F10');
      await waitFor(() => {
        expect(plan.mixedSaves).toHaveLength(1);
      });
      const button = await screen.findByRole('button', {
        name: (label) => visible(label).startsWith(message(language, 'bikri-undo')),
      });
      await user.click(button);
      return { plan, user };
    };

    it('opens the cash box with the cash after from the refusal, and the yes sends the undo again confirmed', async () => {
      const { plan, user } = await saveThenRefuseUndo({ cash_after_paisa: -200_000 });
      const box = await screen.findByRole('alertdialog');
      expect(visible(box.textContent)).toContain(message(language, 'common-cash-confirm-title'));
      expect(visible(box.textContent)).toContain('-2,000');
      // Nothing is undone yet: the first call was the refused one.
      expect(plan.mixedUndos).toEqual([]);
      expect(vi.mocked(bikriApi.bikriMixedUndo)).toHaveBeenCalledTimes(1);
      await user.click(
        screen.getByRole('button', { name: message(language, 'common-cash-confirm-yes') }),
      );
      await waitFor(() => {
        expect(plan.mixedUndos).toEqual([
          { later_entry_ids: [50], new_bikri_id: null, confirm_cash_below_zero: true },
        ]);
      });
      // The lot is on the screen again with the line that was typed, as after any undo.
      await waitFor(() => {
        expect(textOf(salesGrid(language), 0, 'rate')).toBe('560');
      });
    });

    it('the focus starts on "No": Enter out of habit undoes nothing and sends nothing again', async () => {
      const { plan, user } = await saveThenRefuseUndo({ cash_after_paisa: -200_000 });
      await screen.findByRole('alertdialog');
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      });
      expect(plan.mixedUndos).toEqual([]);
      expect(vi.mocked(bikriApi.bikriMixedUndo)).toHaveBeenCalledTimes(1);
    });
  });

  it('a language switch in the middle loses nothing and keeps the lot carried', async () => {
    const plan = fakePlan16();
    const { user } = await open();
    await typeLine(user);
    const rateBefore = textOf(salesGrid(language), 0, 'rate');
    press('KeyL', { altKey: true });
    const other = language === 'ur' ? 'Akram Toll Tax' : 'اکرم ٹول ٹیکس';
    await waitFor(() => {
      expect(visible(salesGrid(language === 'ur' ? 'en' : 'ur').textContent)).toContain(other);
    });
    expect(textOf(salesGrid(language === 'ur' ? 'en' : 'ur'), 0, 'rate')).toBe(rateBefore);
    expect(textOf(lotsGrid(language === 'ur' ? 'en' : 'ur'), 0, 'qty')).toBe('50');
    expect(plan.mixedSaves).toEqual([]);
  });

  it('a lot with nothing left is not put on the screen, and he is told', async () => {
    fakePlan16({ rows: [stockRow({ balance: 0, qty_sold: 50 })] });
    await renderApp('/bikri/later?lot=40');
    expect(
      await screen.findByText(said(message(language, 'bikri-carried-gone'))),
    ).toBeInTheDocument();
    expect(textOf(lotsGrid(language), 0, 'seller')).toBe('');
  });

  it('"From unsold stock" picks a lot of the goods left and puts it on the screen', async () => {
    fakePlan16();
    const view = await renderApp('/bikri/book');
    const user = userEvent.setup();
    await screen.findByRole('button', { name: named(message(language, 'bikri-carried-add')) });
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'bikri-carried-add')) }),
    );
    const dialog = await screen.findByRole('dialog');
    // The list of the goods left, as the backend sent it, in the dialog; a click picks the row.
    const row = await screen.findByText(names.rahim, {
      selector: '[role="gridcell"], [role="gridcell"] *',
    });
    await user.click(row);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(dialog).toBeDefined();
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(names.rahim);
    });
    expect(view.router.state.location.pathname).toBe('/bikri/book');
  });
});
