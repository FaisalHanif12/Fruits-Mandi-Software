// The garden screens show what the REAL backend answers (plan 17c, T2). The backend wrote each
// answer in its own test (tests/fixtures/gardens_*.json, copied, never edited). The screens are
// driven by those answers and by the refusals the backend really sent: the list, an open garden of
// E2E-40 with its history and each partner's money, an installment, a cost, the cash question, a
// closing with its numbers, a closing in loss, the undo, and the refusals in words. In the Urdu and
// in the English app. Every figure on screen is a figure the backend sent.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as urduApi from '../../src/shared/urdu/api';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeGardenItemPicker, fakeGardens } from '../helpers/gardens';
import * as real from '../helpers/gardens-real';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { settlePreview } from '../helpers/preview-settled';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeGardenItemPicker();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;
const pageText = () => visible(document.body.textContent);
const refused = (error: typeof real.GARDENS_ENTRY_SAVE_GARDEN_CLOSED) => new CommandError(error);

describe.each(LANGUAGES)('the garden screens with the backend’s real answers (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeGardens();
  });

  const gardenNumber = message(language, 'gardens-number', { number: 1 });
  const open = async (path = '/gardens/1') => {
    const view = await renderApp(path);
    await screen.findByText(
      said(message(language, 'gardens-detail-title', { number: gardenNumber })),
    );
    return { ...view, user: userEvent.setup() };
  };
  const field = (id: string) => {
    const label = named(message(language, id));
    return (
      screen.queryByRole('textbox', { name: label }) ??
      screen.getByRole('combobox', { name: label })
    );
  };
  const rowCell = (rowId: string, colId: string) => {
    const row = document.querySelector(`.ag-row[row-id="${rowId}"]`);
    return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
  };
  const startsWith = (id: string) => (name: string) =>
    visible(name).startsWith(message(language, id));

  it('the list shows the real garden and the totals the backend worked out', async () => {
    vi.mocked(gardensApi.gardensList).mockResolvedValue(real.GARDENS_LIST_ALL);
    await renderApp('/gardens');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'gardens-title')),
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="1"]')).not.toBeNull();
    });
    expect(rowCell('1', 'number')).toBe(gardenNumber);
    expect(rowCell('1', 'status')).toBe(message(language, 'gardens-status-open'));
    // Money in 66,400 less 55,000 paid: the running balance 11,400, and no rent still due.
    expect(rowCell('1', 'balance')).toBe('11,400');
    const pinned = document.querySelector('.ag-row-pinned');
    expect(visible(pinned?.querySelector('[col-id="balance"]')?.textContent)).toBe('11,400');
  });

  it('an open garden (E2E-40): the figures, the partners and what each one paid', async () => {
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_OPEN);
    await open();
    expect(screen.getByText(said(message(language, 'gardens-status-open')))).toBeInTheDocument();
    const balance = screen.getAllByText(said(message(language, 'gardens-balance')))[0];
    expect(balance?.closest('div')?.textContent).toContain('11,400');
    const partners = screen.getByRole('list', {
      name: named(message(language, 'gardens-partners-title')),
    });
    expect(within(partners).getAllByRole('listitem')).toHaveLength(2);
    expect(visible(partners.textContent)).toContain('60%');
    expect(visible(partners.textContent)).toContain('40%');
    // A paid 33,000 from the shop, B 22,000 from his pocket; nothing is back yet (net −33,000).
    const table = screen.getByRole('table', {
      name: named(message(language, 'gardens-partner-results-title')),
    });
    for (const figure of ['33,000', '22,000', '-33,000', '-22,000']) {
      expect(visible(table.textContent)).toContain(figure);
    }
    // The figures card: 66,400 in, of it 10,000 from the other shop.
    expect(pageText()).toContain('66,400');
    expect(pageText()).toContain('10,000');
  });

  it('the history shows the running balance after each real row', async () => {
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_OPEN);
    await open();
    await waitFor(() => {
      expect(rowCell('e1', 'balance')).toBe('-30,000');
    });
    expect(rowCell('e2', 'balance')).toBe('-50,000');
    expect(rowCell('e3', 'balance')).toBe('-55,000');
    // The sale in our shop has no entry of its own: its row has a place in the list.
    expect(rowCell('i3', 'balance')).toBe('1,400');
    expect(rowCell('i3', 'effect')).toBe('56,400');
    expect(rowCell('e4', 'balance')).toBe('11,400');
    expect(rowCell('e4', 'entry')).toContain(message(language, 'gardens-kind-other-sale'));
  });

  it('an installment: the preview shows the real parts, and the save sends what was typed', async () => {
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_OPEN);
    vi.mocked(gardensApi.gardensEntryPreview).mockResolvedValue(
      real.GARDENS_ENTRY_PREVIEW_INSTALLMENT,
    );
    vi.mocked(gardensApi.gardensEntrySave).mockResolvedValue(real.GARDENS_ENTRY_SAVE_INSTALLMENT);
    const { user } = await open();
    const day = screen.getByRole('textbox', {
      name: named(message(language, 'gardens-field-date')),
    });
    await user.clear(day);
    await user.type(day, '11-10-2026');
    await user.type(field('gardens-installment-amount'), '20000');
    await user.type(field('gardens-payment-reason'), 'second installment');
    const parts = await screen.findByRole('list', {
      name: named(message(language, 'gardens-parts-title')),
    });
    // 20,000 at 60 / 40: 12,000 from the shop and 8,000 from his pocket; balance −30,000 to −50,000.
    expect(visible(parts.textContent)).toContain('12,000');
    expect(visible(parts.textContent)).toContain('8,000');
    expect(
      screen.getByText(said(message(language, 'gardens-numbers-balance-after'))).closest('div')
        ?.textContent,
    ).toContain('-50,000');
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensEntrySave).toHaveBeenCalledTimes(1);
    });
    expect(gardensApi.gardensEntrySave).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'installment',
        garden_id: 1,
        date: '11-10-2026',
        amount: '20000',
        reason: 'second installment',
      }),
    );
    expect(
      await screen.findByText(
        said(
          message(language, 'gardens-saved-installment', {
            number: real.GARDENS_ENTRY_SAVE_INSTALLMENT.number,
          }),
        ),
      ),
    ).toBeInTheDocument();
  });

  it('a payment that takes cash below zero: the real note, the box with the real cash, then the save', async () => {
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_OPEN);
    vi.mocked(gardensApi.gardensEntryPreview).mockResolvedValue(
      real.GARDENS_ENTRY_PREVIEW_CASH_BELOW_ZERO,
    );
    vi.mocked(gardensApi.gardensEntrySave).mockResolvedValue(real.GARDENS_ENTRY_SAVE_INSTALLMENT);
    const { user } = await open();
    await user.type(field('gardens-installment-amount'), '90000');
    await user.type(field('gardens-payment-reason'), 'too much');
    expect(
      await screen.findByText(said(message(language, 'common-cash-confirm-title'))),
    ).toBeInTheDocument();
    // The answer to the last key must have landed (the whole reason typed), so that F10 asks first.
    await waitFor(() => {
      expect(gardensApi.gardensEntryPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ amount: '90000', reason: 'too much' }),
      );
    });
    await act(async () => {
      await vi.mocked(gardensApi.gardensEntryPreview).mock.results.at(-1)?.value;
    });
    await settlePreview();
    press('F10');
    const yes = await screen.findByRole(
      'button',
      { name: startsWith('common-cash-confirm-yes') },
      { timeout: 5000 },
    );
    // The box names the real cash after the entry: −37,000.
    expect(visible(screen.getByRole('alertdialog').textContent)).toContain('37,000');
    await user.click(yes);
    await waitFor(() => {
      expect(gardensApi.gardensEntrySave).toHaveBeenCalledWith(
        expect.objectContaining({ confirm_cash_below_zero: true, amount: '90000' }),
      );
    });
  });

  it('the real refusals of a save read as words at their fields', async () => {
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_OPEN);
    vi.mocked(gardensApi.gardensEntryPreview).mockResolvedValue(
      real.GARDENS_ENTRY_PREVIEW_INSTALLMENT,
    );
    vi.mocked(gardensApi.gardensEntrySave).mockRejectedValueOnce(
      refused(real.GARDENS_ENTRY_SAVE_DATE_BEFORE_GARDEN),
    );
    const { user } = await open();
    await user.type(field('gardens-installment-amount'), '20000');
    await user.type(field('gardens-payment-reason'), 'second installment');
    await screen.findByRole('list', { name: named(message(language, 'gardens-parts-title')) });
    press('F10');
    // The real refusal says the garden's day: written 10-10-2026, never 2026-10-10.
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'err-date-before-garden', { garden_date: '10-10-2026' }),
      );
    });
    expect(pageText()).not.toContain('2026-10-10');
    expect(field('gardens-installment-amount')).toHaveValue('20,000');
    // The garden closed meanwhile: its own words, near the buttons.
    vi.mocked(gardensApi.gardensEntrySave).mockRejectedValueOnce(
      refused(real.GARDENS_ENTRY_SAVE_GARDEN_CLOSED),
    );
    press('F10');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-garden-closed'));
    });
  });

  it('cancelling the cost sends the id and the version the backend gave, with the reason', async () => {
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_OPEN);
    vi.mocked(gardensApi.gardensEntryCancel).mockResolvedValue(real.GARDENS_ENTRY_CANCEL_COST);
    const { user } = await open();
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="e3"] [col-id="entry"]');
      if (found === null) throw new Error('no cost row');
      return found;
    });
    await user.click(row);
    await user.click(
      screen.getByRole('button', { name: message(language, 'gardens-history-cancel') }),
    );
    await user.type(field('gardens-cancel-reason'), 'wrong garden');
    await user.click(
      screen.getByRole('button', { name: message(language, 'gardens-cancel-confirm') }),
    );
    await waitFor(() => {
      expect(gardensApi.gardensEntryCancel).toHaveBeenCalledWith(
        real.GARDENS_ENTRY_CANCEL_COST_REQUEST,
      );
    });
  });

  it('the closing card shows the real numbers and sends the real request', async () => {
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_OPEN);
    vi.mocked(gardensApi.gardensClosePreview).mockResolvedValue(real.GARDENS_CLOSE_PREVIEW_E2E40);
    vi.mocked(gardensApi.gardensClose).mockResolvedValue(real.GARDENS_CLOSE_E2E40);
    const { user } = await open();
    const day = screen.getByRole('textbox', {
      name: named(message(language, 'gardens-close-date')),
    });
    await user.clear(day);
    await user.type(day, '13-10-2026');
    await screen.findByText(said(message(language, 'gardens-close-balance')));
    // 66,400 goes to A 39,840 and B 26,560; the result is 11,400 on a cost of 55,000.
    const credits = await screen.findByRole('list', {
      name: named(message(language, 'gardens-close-credits-title')),
    });
    expect(visible(credits.textContent)).toContain('39,840');
    expect(visible(credits.textContent)).toContain('26,560');
    const card = screen.getByText(said(message(language, 'gardens-close-money-in'))).closest('dl');
    for (const figure of ['66,400', '55,000', '11,400']) {
      expect(visible(card?.textContent)).toContain(figure);
    }
    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: message(language, 'gardens-close-button') }),
      ).toBeEnabled();
    });
    await user.click(
      screen.getByRole('button', { name: message(language, 'gardens-close-button') }),
    );
    await user.click(
      await screen.findByRole('button', { name: message(language, 'gardens-close-confirm') }),
    );
    await waitFor(() => {
      expect(gardensApi.gardensClose).toHaveBeenCalledWith(real.GARDENS_CLOSE_E2E40_REQUEST);
    });
  });

  it('a day before the last entry: the real preview refuses at the day field and the button waits', async () => {
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_OPEN);
    vi.mocked(gardensApi.gardensClosePreview).mockResolvedValue(
      real.GARDENS_CLOSE_PREVIEW_BEFORE_LAST_ENTRY,
    );
    await open();
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'err-close-before-last-entry', { last_date: '12-10-2026' }),
      );
    });
    expect(
      screen.getByRole('button', { name: message(language, 'gardens-close-button') }),
    ).toBeDisabled();
  });

  it('a garden closed in loss (E2E-40b) shows its credits and each partner’s result', async () => {
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_CLOSED_LOSS);
    await open();
    expect(
      screen.getByText(said(message(language, 'gardens-closed-notice', { date: '13-10-2026' }))),
    ).toBeInTheDocument();
    // Closing 10,000 at 60 / 40 gives 6,000 and 4,000; the result is minus 45,000.
    const table = screen.getByRole('table', {
      name: named(message(language, 'gardens-partner-results-title')),
    });
    for (const figure of ['6,000', '4,000', '-27,000']) {
      expect(visible(table.textContent)).toContain(figure);
    }
    // A closed garden has no form and no closing card.
    expect(
      screen.queryByRole('group', { name: named(message(language, 'gardens-type-label')) }),
    ).toBeNull();
    expect(screen.queryByText(said(message(language, 'gardens-close-title')))).toBeNull();
  });

  it('the undo sends the real request: the garden, its version and the reason', async () => {
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_CLOSED);
    vi.mocked(gardensApi.gardensReopen).mockResolvedValue(real.GARDENS_REOPEN_E2E40);
    const { user } = await open();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'gardens-reopen-button')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.type(within(dialog).getByRole('combobox'), 'wrong day');
    await user.click(
      within(dialog).getByRole('button', { name: message(language, 'gardens-reopen-confirm') }),
    );
    await waitFor(() => {
      expect(gardensApi.gardensReopen).toHaveBeenCalledWith(real.GARDENS_REOPEN_E2E40_REQUEST);
    });
  });

  it('the garden after the undo shows its cancelled closing row in the history', async () => {
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_REOPENED);
    await open();
    expect(screen.getByText(said(message(language, 'gardens-status-open')))).toBeInTheDocument();
    await waitFor(() => {
      expect(rowCell('e5', 'state')).toBe(message(language, 'gardens-entry-cancelled'));
    });
    expect(
      screen.getByRole('group', { name: named(message(language, 'gardens-type-label')) }),
    ).toBeInTheDocument();
  });

  it('a new garden: the real preview shows the first installment’s parts, the save opens the garden', async () => {
    vi.mocked(gardensApi.gardensDefaults).mockResolvedValue(real.GARDENS_DEFAULTS_FIRST);
    vi.mocked(gardensApi.gardensCreatePreview).mockResolvedValue(
      real.GARDENS_CREATE_PREVIEW_FIRST_INSTALLMENT,
    );
    vi.mocked(gardensApi.gardensCreate).mockResolvedValue(real.GARDENS_CREATE_E2E40);
    vi.mocked(gardensApi.gardensGet).mockResolvedValue(real.GARDENS_GET_OPEN);
    const view = await renderApp('/gardens/new');
    await screen.findByText(said(message(language, 'gardens-new-from-defaults')));
    const user = userEvent.setup();
    await user.type(field('gardens-field-rent'), '50000');
    await user.type(field('gardens-field-paid-now'), '30000');
    const parts = await screen.findByRole('list', {
      name: named(message(language, 'gardens-first-parts-title')),
    });
    // 30,000 at 60 / 40: 18,000 from the shop's cash, 12,000 from a pocket (the real preview sends
    // the partners as the backend's test chose them, so only the figures are checked).
    expect(visible(parts.textContent)).toContain('18,000');
    expect(visible(parts.textContent)).toContain('12,000');
    await waitFor(() => {
      expect(gardensApi.gardensCreatePreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ rent: '50000', paid_now: '30000' }),
      );
    });
    // The reason of the first installment is the standard words, filled in for the Owner.
    expect(field('gardens-field-paid-now-reason')).toHaveValue(
      message(language, 'gardens-first-installment-reason'),
    );
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensCreate).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(view.router.state.location.pathname).toBe('/gardens/1');
    });
  });

  it('the shares of a new garden are refused with the real words when they do not total 100%', async () => {
    vi.mocked(gardensApi.gardensDefaults).mockResolvedValue(real.GARDENS_DEFAULTS_FIRST);
    vi.mocked(gardensApi.gardensCreatePreview).mockResolvedValue(
      real.GARDENS_CREATE_PREVIEW_FIRST_INSTALLMENT,
    );
    vi.mocked(gardensApi.gardensCreate).mockRejectedValueOnce(
      refused(real.GARDENS_CREATE_SHARES_NOT_100),
    );
    await renderApp('/gardens/new');
    await screen.findByText(said(message(language, 'gardens-new-from-defaults')));
    const user = userEvent.setup();
    await user.type(field('gardens-field-rent'), '50000');
    await screen.findByText(said(message(language, 'partners-total-ok')));
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensCreate).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-shares-not-100', { total: '90' }));
    });
  });
});
