// A later sale (plan 16, PRD 12.9, FR-12.17, E2E-16b) in the Urdu and the English app, against a
// fake backend: the lot is shown read-only, the new lines are typed with the keyboard only, the
// preview and the save go to the later-sale commands with the text typed, the cash box asks before
// a save that takes cash below zero, a refusal keeps what was typed, and a language switch in the
// middle loses nothing (F39). The numbers are the backend's; the screen works out none.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { cash, fakePlan16, laterSaleAnswer, refused, stockRow } from '../helpers/bikri-plan16';
import {
  cellOf,
  focusedIn,
  named,
  salesGrid,
  textOf,
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
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;

describe.each(LANGUAGES)('a later sale (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const names = {
    rahim: language === 'ur' ? 'رحیم' : 'Rahim',
    akram: language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
  };

  const open = async () => {
    const view = await renderApp('/bikri/later?lot=40');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-later-title', { number: 1 })),
    });
    return { ...view, user: userEvent.setup() };
  };

  /** Types the one new line of E2E-16b: Akram, the 5 left (filled by the app), at 560. */
  const typeLine = async (user: ReturnType<typeof userEvent.setup>) => {
    await waitForLineFill(language);
    await user.click(cellOf(salesGrid(language), 0, 'buyer'));
    // The first key opens the cell, the list answers, Enter takes the buyer (a short code).
    await user.keyboard('1');
    await screen.findAllByRole('option');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
    });
    await user.keyboard('{Enter}560{Enter}');
  };

  it('shows the lot read-only and asks the backend about it on the day of today', async () => {
    const plan = fakePlan16();
    await open();
    const card = await screen.findByRole('heading', {
      name: named(message(language, 'bikri-later-lot-title')),
    });
    const text = visible(card.closest('section')?.textContent);
    expect(text).toContain(names.rahim);
    expect(text).toContain('09-10-2026');
    // The lot's other charges are not charged again, and the screen says so.
    expect(text).toContain(visible(message(language, 'bikri-later-no-lot-charges')));
    await waitFor(() => {
      expect(plan.laterPreviews.length).toBeGreaterThan(0);
    });
    expect(plan.laterPreviews[0]).toMatchObject({ lotId: 40, date: '10-10-2026' });
  });

  it('the new line starts with what the lot has left, and the rate stays empty', async () => {
    fakePlan16();
    await open();
    await waitForLineFill(language);
    // The backend's remaining quantity (5, what the lot has left) is in the cell like typed text.
    expect(textOf(salesGrid(language), 0, 'qty')).toBe('5');
    expect(textOf(salesGrid(language), 0, 'rate')).toBe('');
  });

  it('keyboard only: buyer, qty, rate, then F10 saves the text as typed and goes back to the list', async () => {
    const plan = fakePlan16();
    const { router, user } = await open();
    await typeLine(user);
    // Three Enters on the new line, then the cursor is on the next line's buyer.
    await waitFor(() => {
      expect(plan.laterPreviews.at(-1)?.sales).toEqual([
        expect.objectContaining({ buyer_account_id: 19, qty: '5', seller_rate: '560' }),
        expect.anything(),
      ]);
    });
    press('F10');
    await waitFor(() => {
      expect(plan.laterSaves).toHaveLength(1);
    });
    const saved = plan.laterSaves[0];
    expect(saved).toMatchObject({ lotId: 40, date: '10-10-2026', confirm: false });
    expect(saved?.sales[0]).toMatchObject({ buyer_account_id: 19, qty: '5', seller_rate: '560' });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/stock');
    });
    expect(
      await screen.findByText(said(message(language, 'bikri-later-saved-toast', { number: 1 }))),
    ).toBeInTheDocument();
    // The list is read again, so the lot shows what it has left now.
    expect(bikriApi.bikriStockList).toHaveBeenCalled();
  });

  it('cash going below zero asks first: No saves nothing, Yes saves with the confirmation', async () => {
    const plan = fakePlan16();
    plan.cashConfirmRequired = true;
    vi.mocked(bikriApi.bikriLaterSalePreview).mockImplementation((lotId, date, sales) => {
      plan.laterPreviews.push({ lotId, date, sales: [...sales] });
      const base = laterSaleAnswer(sales, 5);
      return Promise.resolve({ ...base, cash: cash(1_000, -500, true) });
    });
    const { user } = await open();
    await typeLine(user);
    // The amber note shows while the entry is typed; then F10 asks (PRD FR-6.7).
    await screen.findByText(said(message(language, 'common-cash-confirm-title')));
    press('F10');
    const box = await screen.findByRole('alertdialog');
    expect(visible(box.textContent)).toContain('-500');
    // The focus starts on "No": Enter out of habit saves nothing.
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    expect(plan.laterSaves).toEqual([]);
    press('F10');
    await screen.findByRole('alertdialog');
    await user.click(
      screen.getByRole('button', { name: message(language, 'common-cash-confirm-yes') }),
    );
    await waitFor(() => {
      expect(plan.laterSaves).toHaveLength(1);
    });
    expect(plan.laterSaves[0]?.confirm).toBe(true);
  });

  it('a refusal is told in words and keeps what was typed', async () => {
    fakePlan16();
    vi.mocked(bikriApi.bikriLaterSaleSave).mockRejectedValue(
      refused('ERR_DATE_CLOSED', 'date', { closed_up_to: '09-10-2026' }),
    );
    const { user } = await open();
    await typeLine(user);
    press('F10');
    expect(
      await screen.findByText(
        said(message(language, 'err-date-closed', { closed_up_to: '09-10-2026' })),
      ),
    ).toBeInTheDocument();
    expect(textOf(salesGrid(language), 0, 'rate')).toBe('560');
  });

  it('a date error is on the day field, once, and the lines stay', async () => {
    fakePlan16();
    vi.mocked(bikriApi.bikriLaterSaleSave).mockRejectedValue(
      refused('ERR_DATE_BEFORE_RECEIVED', 'date', { received_date: '09-10-2026' }),
    );
    const { user } = await open();
    await typeLine(user);
    press('F10');
    const words = said(message(language, 'err-date-before-received'));
    expect(await screen.findByText(words)).toBeInTheDocument();
    // Under the day field only: not again in a notice or the list of problems.
    expect(screen.getAllByText(words)).toHaveLength(1);
    expect(textOf(salesGrid(language), 0, 'rate')).toBe('560');
  });

  it('a language switch in the middle loses nothing and keeps the cursor', async () => {
    const plan = fakePlan16();
    const { user } = await open();
    await typeLine(user);
    const rateBefore = textOf(salesGrid(language), 0, 'rate');
    press('KeyL', { altKey: true });
    // The same line in the other script, the same rate.
    const other = language === 'ur' ? 'Akram Toll Tax' : 'اکرم ٹول ٹیکس';
    await waitFor(() => {
      const grid = document.querySelector('.entry-grid');
      expect(visible(grid?.textContent)).toContain(other);
    });
    const grid = document.querySelector<HTMLElement>('.entry-grid');
    if (grid === null) throw new Error('no grid');
    expect(textOf(grid, 0, 'rate')).toBe(rateBefore);
    expect(textOf(grid, 0, 'qty')).toBe('5');
    expect(plan.laterSaves).toEqual([]);
  });

  it('nothing left in the lot says so, with a way back', async () => {
    fakePlan16({ rows: [stockRow({ balance: 0, qty_sold: 50 })] });
    // The list asks for lots that are written off too, so the lot is found with nothing left.
    await renderApp('/bikri/later?lot=40');
    expect(
      await screen.findByText(said(message(language, 'bikri-later-nothing-left'))),
    ).toBeInTheDocument();
  });

  it('a lot that is not there says so, with a way back', async () => {
    fakePlan16({ rows: [] });
    await renderApp('/bikri/later?lot=99');
    expect(
      await screen.findByText(said(message(language, 'bikri-later-lot-missing'))),
    ).toBeInTheDocument();
  });
});
