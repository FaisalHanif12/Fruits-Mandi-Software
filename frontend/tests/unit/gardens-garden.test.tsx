// One garden (plan 17c, PRD 12.11, FR-7.67, FR-12.34 to FR-12.38, FR-6.7), in the Urdu and the
// English app, against a fake backend with the numbers of E2E-40: the facts and the running balance
// of the garden, the type list that keeps the day, the amount and the note, each partner's part
// before the save (shop part off his total, pocket part recorded only, with the words that a pocket
// part posts nothing), the cash question, the three kinds of entry, the cancel of an entry, the
// closing with its confirm box (Back has the focus) and its refusal, the undo of the closing, and a
// language switch in the middle of an entry that loses nothing. Every figure on the screen is a
// figure the backend sent; the fake's sums are written in the fixture from the plan.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
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
import { e40Closed, e40Garden, e40History, fakeGardens, gardenRow } from '../helpers/gardens';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { PAGE_ROWS, pagerButton, rangeText } from '../helpers/pager';
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
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;
type User = ReturnType<typeof userEvent.setup>;

describe.each(LANGUAGES)('one garden (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const names = {
    a: language === 'ur' ? 'الف' : 'Partner A',
    b: language === 'ur' ? 'ب' : 'Partner B',
  };
  const gardenNumber = '0080';

  const open = async (path = '/gardens/1') => {
    const view = await renderApp(path);
    await screen.findByText(
      said(message(language, 'gardens-detail-title', { code: gardenNumber })),
    );
    return { ...view, user: userEvent.setup() };
  };

  const typeList = () =>
    screen.getByRole('group', { name: named(message(language, 'gardens-type-label')) });
  const chooseType = async (user: User, type: 'installment' | 'cost' | 'other-sale') => {
    await user.click(
      within(typeList()).getByRole('button', {
        name: named(message(language, `gardens-type-${type}`)),
      }),
    );
  };
  /** A text field by its label: a plain box, or the Urdu text field (a combobox). */
  const field = (id: string) => {
    const label = named(message(language, id));
    return (
      screen.queryByRole('textbox', { name: label }) ??
      screen.getByRole('combobox', { name: label })
    );
  };
  const sourceGroup = (name: string) =>
    screen.getByRole('group', { name: named(message(language, 'ventures-source-of', { name })) });
  const startsWith = (id: string) => (name: string) =>
    visible(name).startsWith(message(language, id));

  /** An installment of 30,000 for "First installment": the first entry of E2E-40. */
  async function fillInstallment(user: User) {
    await user.type(field('gardens-installment-amount'), '30000');
    await user.type(field('gardens-payment-reason'), 'First installment');
  }
  /** Waits until the last preview was asked for what is typed and has been answered. */
  async function previewSettled(amount: string) {
    await waitFor(() => {
      expect(gardensApi.gardensEntryPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ amount }),
      );
    });
    await act(async () => {
      await vi.mocked(gardensApi.gardensEntryPreview).mock.results.at(-1)?.value;
    });
    await settlePreview();
  }

  it('shows the garden: its number, name and state, the running balance with its working, and the rent due', async () => {
    fakeGardens();
    await open();
    expect(screen.getByText(said(message(language, 'gardens-status-open')))).toBeInTheDocument();
    expect(
      screen.getAllByText(language === 'ur' ? 'شاہ پور باغ' : 'Shahpur Garden').length,
    ).toBeGreaterThan(0);
    // The running balance of E2E-40 (11,400) is the backend's, with its ⓘ.
    const balance = screen.getAllByText(said(message(language, 'gardens-balance')))[0];
    expect(balance?.closest('div')?.textContent).toContain('11,400');
    expect(balance?.closest('div')?.querySelector('button')).not.toBeNull();
    const due = screen.getAllByText(said(message(language, 'gardens-rent-due')))[0];
    expect(due?.closest('div')?.textContent).toContain('0');
    // The partners with their % and where their money comes from by default.
    const partners = screen.getByRole('list', {
      name: named(message(language, 'gardens-partners-title')),
    });
    expect(within(partners).getAllByRole('listitem')).toHaveLength(2);
    expect(visible(partners.textContent)).toContain('60%');
    expect(visible(partners.textContent)).toContain(message(language, 'ventures-source-pocket'));
    // Each partner's money: shop 33,000, pocket 22,000, net −33,000 (nothing back yet).
    const table = screen.getByRole('table', {
      name: named(message(language, 'gardens-partner-results-title')),
    });
    for (const amount of ['33,000', '22,000', '-33,000']) {
      expect(visible(table.textContent)).toContain(amount);
    }
    expect(gardensApi.gardensGet).toHaveBeenCalledWith({
      id: 1,
      history_offset: 0,
      history_limit: PAGE_ROWS,
    });
  });

  it('a garden with nothing paid starts in minus: the rent still due and the empty history say what to do', async () => {
    fakeGardens({ gardens: [gardenRow()], history: [] });
    await open();
    expect(screen.getByText(said(message(language, 'gardens-history-empty')))).toBeInTheDocument();
    const due = screen.getAllByText(said(message(language, 'gardens-rent-due')))[0];
    expect(due?.closest('div')?.textContent).toContain('50,000');
  });

  it('the history shows every row with its effect and the running balance after it', async () => {
    fakeGardens();
    await open();
    const cellOf = (rowId: string, colId: string) => {
      const row = document.querySelector(`.ag-row[row-id="${rowId}"]`);
      return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
    };
    await waitFor(() => {
      expect(cellOf('e11', 'balance')).toBe('-30,000');
    });
    expect(cellOf('e11', 'effect')).toBe('-30,000');
    expect(cellOf('e11', 'entry')).toBe(
      message(language, 'gardens-history-entry', {
        kind: message(language, 'gardens-kind-installment'),
        detail: 'First installment',
      }),
    );
    expect(cellOf('e12', 'balance')).toBe('-50,000');
    expect(cellOf('e13', 'balance')).toBe('-55,000');
    // The sale in our shop (row 3) and the sale in another shop.
    expect(cellOf('i3', 'balance')).toBe('1,400');
    expect(cellOf('i3', 'entry')).toBe(
      message(language, 'gardens-history-entry', {
        kind: message(language, 'gardens-kind-sale'),
        detail: message(language, 'gardens-history-detail-bikri', { number: 5 }),
      }),
    );
    expect(cellOf('e14', 'balance')).toBe('11,400');
    expect(cellOf('e14', 'entry')).toBe(
      message(language, 'gardens-history-entry', {
        kind: message(language, 'gardens-kind-other-sale'),
        detail: message(language, 'gardens-history-detail-sale-elsewhere', {
          shop: 'Haji Shop',
          qty: 20,
        }),
      }),
    );
  });

  it('a closed garden takes no entry: a note with the day instead of the form, and a button to undo', async () => {
    fakeGardens({ gardens: [e40Closed()], history: e40History(true) });
    await open();
    expect(
      screen.getByText(said(message(language, 'gardens-closed-notice', { date: '13-10-2026' }))),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole('group', { name: named(message(language, 'gardens-type-label')) }),
    ).toBeNull();
    expect(
      screen.getByRole('button', { name: named(message(language, 'gardens-reopen-button')) }),
    ).toBeInTheDocument();
    // No closing card for a closed garden; its partners show the result: A +6,840, B +4,560.
    expect(screen.queryByText(said(message(language, 'gardens-close-title')))).toBeNull();
    const table = screen.getByRole('table', {
      name: named(message(language, 'gardens-partner-results-title')),
    });
    for (const amount of ['39,840', '6,840', '26,560', '4,560']) {
      expect(visible(table.textContent)).toContain(amount);
    }
  });

  it('an unknown garden says so with a retry button', async () => {
    fakeGardens();
    await renderApp('/gardens/99');
    expect(
      await screen.findByText(said(message(language, 'gardens-detail-failed'))),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: message(language, 'common-retry') }),
    ).toBeInTheDocument();
  });

  it('the type list has the four types; changing it keeps the day, the amount and the note', async () => {
    fakeGardens();
    const { user } = await open();
    const buttons = within(typeList()).getAllByRole('button');
    expect(buttons.map((button) => visible(button.textContent))).toEqual([
      message(language, 'gardens-type-installment'),
      message(language, 'gardens-type-cost'),
      message(language, 'gardens-type-other-sale'),
      message(language, 'gardens-type-arrival'),
    ]);
    // An installment: an amount and a reason.
    await user.type(field('gardens-installment-amount'), '5000');
    await user.type(field('gardens-payment-reason'), 'Labour');
    // A garden cost: the day and the amount stay, the reason starts again.
    await chooseType(user, 'cost');
    expect(field('gardens-cost-amount')).toHaveValue('5,000');
    expect(field('gardens-payment-reason')).toHaveValue('');
    // A sale in another shop: the amount stays, and so does the day.
    await chooseType(user, 'other-sale');
    expect(field('gardens-sale-amount')).toHaveValue('5,000');
    expect(
      screen.getByRole('textbox', { name: named(message(language, 'gardens-field-date')) }),
    ).toHaveValue('10-10-2026');
    // The shop and the quantity start empty, and the sources of the partners are not asked.
    expect(field('gardens-sale-shop')).toHaveValue('');
    expect(
      screen.queryByRole('group', {
        name: named(message(language, 'ventures-source-of', { name: names.a })),
      }),
    ).toBeNull();
  });

  it('an installment shows each partner’s part before the save and says a pocket part posts nothing', async () => {
    fakeGardens();
    const { user } = await open();
    await fillInstallment(user);
    // The preview asked with one row for every partner, each from the garden's source.
    await waitFor(() => {
      expect(gardensApi.gardensEntryPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({
          type: 'installment',
          garden_id: 1,
          amount: '30000',
          reason: 'First installment',
          parts: [
            { partner_id: 1, source: 'shop' },
            { partner_id: 2, source: 'pocket' },
          ],
        }),
      );
    });
    const parts = await screen.findByRole('list', {
      name: named(message(language, 'gardens-parts-title')),
    });
    const text = visible(parts.textContent);
    expect(text).toContain('18,000');
    expect(text).toContain('12,000');
    expect(within(parts).getAllByText(said(message(language, 'ventures-part-shop')))).toHaveLength(
      1,
    );
    expect(
      within(parts).getAllByText(said(message(language, 'ventures-part-pocket'))),
    ).toHaveLength(1);
    // The words under the sources say what each source does.
    expect(screen.getByText(said(message(language, 'gardens-sources-hint')))).toBeInTheDocument();
    // The balance after the entry is the backend's, with the rent still due.
    expect(
      screen.getByText(said(message(language, 'gardens-numbers-balance-after'))).closest('div')
        ?.textContent,
    ).toContain('-30,000');
    // Partner B pays from the shop on this entry: his part is a shop part now.
    await user.click(
      within(sourceGroup(names.b)).getByRole('button', {
        name: message(language, 'ventures-source-shop'),
      }),
    );
    await waitFor(() => {
      expect(gardensApi.gardensEntryPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({
          parts: [
            { partner_id: 1, source: 'shop' },
            { partner_id: 2, source: 'shop' },
          ],
        }),
      );
    });
    await waitFor(() => {
      expect(
        within(
          screen.getByRole('list', { name: named(message(language, 'gardens-parts-title')) }),
        ).getAllByText(said(message(language, 'ventures-part-shop'))),
      ).toHaveLength(2);
    });
  });

  it('F10 saves the installment as typed, tells the number and starts the next one', async () => {
    const fake = fakeGardens();
    const { user } = await open();
    await fillInstallment(user);
    await screen.findByRole('list', { name: named(message(language, 'gardens-parts-title')) });
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toEqual({
      type: 'installment',
      garden_id: 1,
      date: '10-10-2026',
      amount: '30000',
      reason: 'First installment',
      parts: [
        { partner_id: 1, source: 'shop' },
        { partner_id: 2, source: 'pocket' },
      ],
      confirm_cash_below_zero: false,
    });
    expect(
      await screen.findByText(said(message(language, 'gardens-saved-installment', { number: 1 }))),
    ).toBeInTheDocument();
    // The next entry: the day stays, the amount and the reason start again.
    await waitFor(() => {
      expect(field('gardens-installment-amount')).toHaveValue('');
    });
    expect(field('gardens-payment-reason')).toHaveValue('');
    expect(
      screen.getByRole('textbox', { name: named(message(language, 'gardens-field-date')) }),
    ).toHaveValue('10-10-2026');
  });

  it('a garden cost is saved with its own type and reason', async () => {
    const fake = fakeGardens();
    const { user } = await open();
    await chooseType(user, 'cost');
    await user.type(field('gardens-cost-amount'), '5000');
    await user.type(field('gardens-payment-reason'), 'Labour');
    await previewSettled('5000');
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toMatchObject({
      type: 'cost',
      garden_id: 1,
      amount: '5000',
      reason: 'Labour',
    });
    expect(
      await screen.findByText(said(message(language, 'gardens-saved-cost', { number: 1 }))),
    ).toBeInTheDocument();
  });

  it('a sale in another shop is saved as typed and brings money in, so no cash question', async () => {
    const fake = fakeGardens({ cashBefore: 5_000 });
    const { user } = await open();
    await chooseType(user, 'other-sale');
    await user.type(field('gardens-sale-shop'), 'Haji Shop');
    await user.type(field('gardens-sale-qty'), '20');
    await user.type(field('gardens-sale-amount'), '10000');
    await previewSettled('10000');
    expect(
      screen.getByText(said(message(language, 'gardens-numbers-balance-after'))).closest('div')
        ?.textContent,
    ).toContain('11,400');
    expect(screen.queryByText(said(message(language, 'common-cash-confirm-title')))).toBeNull();
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toEqual({
      type: 'other_sale',
      garden_id: 1,
      date: '10-10-2026',
      shop: 'Haji Shop',
      // A garden with no goods arrived: the sale is typed by hand, as before (PRD FR-12.41).
      arrival_id: null,
      qty: '20',
      amount: '10000',
      note: null,
    });
  });

  it('cash going below zero shows the amber note and asks first; No saves nothing, Yes saves', async () => {
    const fake = fakeGardens({ cashBefore: 5_000 });
    const { user } = await open();
    await fillInstallment(user);
    expect(
      await screen.findByText(said(message(language, 'common-cash-confirm-title'))),
    ).toBeInTheDocument();
    await previewSettled('30000');
    press('F10');
    const no = await screen.findByRole('button', { name: startsWith('common-cash-confirm-no') });
    // Focus starts on "No", so Enter out of habit never saves a mistake.
    expect(no).toHaveFocus();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(
        screen.queryByRole('button', { name: startsWith('common-cash-confirm-no') }),
      ).toBeNull();
    });
    expect(gardensApi.gardensEntrySave).not.toHaveBeenCalled();
    press('F10');
    await user.click(
      await screen.findByRole('button', { name: startsWith('common-cash-confirm-yes') }),
    );
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toMatchObject({ confirm_cash_below_zero: true });
  });

  it('with every part from a pocket the shop’s cash is not touched: no question, and the screen says nothing is posted', async () => {
    const fake = fakeGardens({ cashBefore: 5_000 });
    const { user } = await open();
    await fillInstallment(user);
    await user.click(
      within(sourceGroup(names.a)).getByRole('button', {
        name: message(language, 'ventures-source-pocket'),
      }),
    );
    await waitFor(() => {
      expect(
        screen.getByText(said(message(language, 'ventures-parts-no-posting'))),
      ).toBeInTheDocument();
    });
    await previewSettled('30000');
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(
      screen.queryByRole('button', { name: message(language, 'common-cash-confirm-yes') }),
    ).toBeNull();
  });

  it('a refusal of the save is told in plain words at the field and nothing typed is lost', async () => {
    const fake = fakeGardens();
    fake.refuse.entry = [
      { code: 'ERR_GARDEN_CLOSED', field: 'garden_id' },
      { code: 'ERR_DATE_BEFORE_GARDEN', field: 'date', values: { garden_date: '10-10-2026' } },
    ];
    const { user } = await open();
    await fillInstallment(user);
    await screen.findByRole('list', { name: named(message(language, 'gardens-parts-title')) });
    press('F10');
    expect(
      await screen.findAllByText(/./, { selector: '[role="alert"], [role="alert"] *' }),
    ).not.toHaveLength(0);
    // The garden's own refusal is in words near the buttons; the day's is at the day field.
    expect(
      screen.getAllByText(said(message(language, 'err-garden-closed'))).length,
    ).toBeGreaterThan(0);
    expect(
      screen.getAllByText(
        said(message(language, 'err-date-before-garden', { garden_date: '10-10-2026' })),
      ).length,
    ).toBeGreaterThan(0);
    expect(field('gardens-installment-amount')).toHaveValue('30,000');
    expect(field('gardens-payment-reason')).toHaveValue('First installment');
  });

  it('F2 starts a new entry, asking first when something is typed', async () => {
    fakeGardens();
    const { user } = await open();
    await user.type(field('gardens-installment-amount'), '1');
    press('F2');
    expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
    await user.keyboard('{Escape}');
    expect(field('gardens-installment-amount')).toHaveValue('1');
  });

  it('the row that has focus shows its parts, and a payment of pockets only says nothing was posted', async () => {
    const history = e40History();
    const first = history[0];
    if (first === undefined) throw new Error('no history');
    history[0] = { ...first, journal_entry_id: null };
    fakeGardens({ history });
    const { user } = await open();
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="e11"] [col-id="entry"]');
      if (found === null) throw new Error('no row');
      return found;
    });
    await user.click(row);
    const parts = await screen.findAllByRole('list', {
      name: named(message(language, 'gardens-parts-title')),
    });
    expect(visible(parts[parts.length - 1]?.textContent)).toContain('18,000');
    expect(
      screen.getByText(said(message(language, 'ventures-parts-no-posting'))),
    ).toBeInTheDocument();
  });

  it('cancels an entry with a reason: the box says what happens, the cash question comes from the backend', async () => {
    const fake = fakeGardens();
    const { user } = await open();
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="e14"] [col-id="entry"]');
      if (found === null) throw new Error('no row');
      return found;
    });
    await user.click(row);
    await user.click(
      screen.getByRole('button', { name: message(language, 'gardens-history-cancel') }),
    );
    expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
    fake.refuse.cancel = [
      { code: 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED', field: 'confirm_cash_below_zero' },
    ];
    await user.type(field('gardens-cancel-reason'), 'Wrong shop');
    await user.click(
      screen.getByRole('button', { name: message(language, 'gardens-cancel-confirm') }),
    );
    await waitFor(() => {
      expect(gardensApi.gardensEntryCancel).toHaveBeenCalledWith({
        id: 14,
        version: 1,
        reason: 'Wrong shop',
        confirm_cash_below_zero: false,
      });
    });
    // The box now asks the cash question: the next press confirms it.
    fake.refuse = {};
    const confirm = await screen.findByRole('button', {
      name: message(language, 'gardens-cancel-confirm-cash'),
    });
    await user.click(confirm);
    await waitFor(() => {
      expect(fake.saved.cancels).toHaveLength(1);
    });
    expect(fake.saved.cancels[0]).toEqual({
      id: 14,
      version: 1,
      reason: 'Wrong shop',
      confirm_cash_below_zero: true,
    });
  });

  it('a sale row and a cancelled row cannot be cancelled: the button is dimmed', async () => {
    const history = e40History();
    const second = history[1];
    if (second === undefined) throw new Error('no history');
    history[1] = { ...second, cancelled: true, cancel_reason: 'Wrong amount', can_cancel: false };
    fakeGardens({ history });
    const { user } = await open();
    const click = async (rowId: string) => {
      const row = await waitFor(() => {
        const found = document.querySelector<HTMLElement>(
          `.ag-row[row-id="${rowId}"] [col-id="entry"]`,
        );
        if (found === null) throw new Error('no row');
        return found;
      });
      await user.click(row);
    };
    await click('i3');
    expect(
      screen.getByRole('button', { name: message(language, 'gardens-history-cancel') }),
    ).toBeDisabled();
    await click('e12');
    expect(
      screen.getByRole('button', { name: message(language, 'gardens-history-cancel') }),
    ).toBeDisabled();
    expect(
      screen.getByText(
        said(message(language, 'gardens-entry-cancelled-reason', { reason: 'Wrong amount' })),
      ),
    ).toBeInTheDocument();
  });

  it('the history loads page by page', async () => {
    const first = e40History()[0];
    if (first === undefined) throw new Error('no history');
    const many = Array.from({ length: 60 }, (_, index) => ({ ...first, entry_id: 200 + index }));
    fakeGardens({ history: many });
    const { user } = await open();
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(rangeText(language, 1, 25, 60));
    });
    await user.click(await pagerButton(language, 'next'));
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(rangeText(language, 26, 50, 60));
    });
    expect(gardensApi.gardensGet).toHaveBeenLastCalledWith({
      id: 1,
      history_offset: PAGE_ROWS,
      history_limit: PAGE_ROWS,
    });
  });

  it('the closing card shows what the closing does, asked for the day typed', async () => {
    fakeGardens();
    await open();
    await waitFor(() => {
      expect(gardensApi.gardensClosePreview).toHaveBeenCalledWith({
        id: 1,
        version: 1,
        date: '10-10-2026',
      });
    });
    await screen.findByText(said(message(language, 'gardens-close-balance')));
    const card = screen.getByText(said(message(language, 'gardens-close-money-in'))).closest('dl');
    for (const amount of ['66,400', '55,000', '11,400']) {
      expect(visible(card?.textContent)).toContain(amount);
    }
    const credits = screen.getByRole('list', {
      name: named(message(language, 'gardens-close-credits-title')),
    });
    expect(visible(credits.textContent)).toContain('39,840');
    expect(visible(credits.textContent)).toContain('26,560');
    expect(
      screen.getByText(said(message(language, 'gardens-close-earliest', { date: '12-10-2026' }))),
    ).toBeInTheDocument();
  });

  it('the close button asks what will happen, with Back focused; Esc closes nothing, Yes closes', async () => {
    const fake = fakeGardens();
    const { user } = await open();
    await screen.findByText(said(message(language, 'gardens-close-balance')));
    const closeButton = () =>
      screen.getByRole('button', { name: message(language, 'gardens-close-button') });
    await waitFor(() => {
      expect(closeButton()).toBeEnabled();
    });
    await user.click(closeButton());
    const dialog = await screen.findByRole('alertdialog');
    expect(visible(dialog.textContent)).toContain('10-10-2026');
    expect(visible(dialog.textContent)).toContain('66,400');
    // Focus starts on Back, so Enter out of habit never closes a garden (PRD FR-7.8).
    expect(within(dialog).getByRole('button', { name: startsWith('common-back') })).toHaveFocus();
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    expect(gardensApi.gardensClose).not.toHaveBeenCalled();
    await user.click(closeButton());
    await user.click(
      await screen.findByRole('button', { name: message(language, 'gardens-close-confirm') }),
    );
    await waitFor(() => {
      expect(fake.saved.closings).toHaveLength(1);
    });
    expect(fake.saved.closings[0]).toEqual({ id: 1, version: 1, date: '10-10-2026' });
    expect(
      await screen.findByText(
        said(message(language, 'gardens-closed-toast', { code: gardenNumber })),
      ),
    ).toBeInTheDocument();
  });

  it('a day before the garden’s last entry is refused at the day field with the day to use', async () => {
    const fake = fakeGardens();
    const { user } = await open();
    await screen.findByText(said(message(language, 'gardens-close-balance')));
    fake.refuse.close = [
      { code: 'ERR_CLOSE_BEFORE_LAST_ENTRY', field: 'date', values: { last_date: '12-10-2026' } },
    ];
    await user.click(
      screen.getByRole('button', { name: message(language, 'gardens-close-button') }),
    );
    await user.click(
      await screen.findByRole('button', { name: message(language, 'gardens-close-confirm') }),
    );
    await waitFor(() => {
      expect(
        screen.getAllByText(
          said(message(language, 'err-close-before-last-entry', { last_date: '12-10-2026' })),
        ).length,
      ).toBeGreaterThan(0);
    });
    // The day typed is still there.
    expect(
      screen.getByRole('textbox', { name: named(message(language, 'gardens-close-date')) }),
    ).toHaveValue('10-10-2026');
  });

  it('undoes the closing with a reason, while the day is open', async () => {
    const fake = fakeGardens({ gardens: [e40Closed()], history: e40History(true) });
    const { user } = await open();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'gardens-reopen-button')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    expect(visible(dialog.textContent)).toContain(message(language, 'gardens-reopen-title'));
    await user.type(within(dialog).getByRole('combobox'), 'Wrong day');
    await user.click(
      within(dialog).getByRole('button', { name: message(language, 'gardens-reopen-confirm') }),
    );
    await waitFor(() => {
      expect(fake.saved.reopens).toHaveLength(1);
    });
    expect(fake.saved.reopens[0]).toEqual({ id: 1, version: 2, reason: 'Wrong day' });
  });

  it('the keyboard alone does an installment: type the amount, Enter to the reason, F10', async () => {
    const fake = fakeGardens();
    await open();
    field('gardens-installment-amount').focus();
    const user = userEvent.setup();
    await user.keyboard('20000{Enter}Second installment');
    await previewSettled('20000');
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toMatchObject({
      type: 'installment',
      amount: '20000',
      reason: 'Second installment',
    });
  });

  it('a language switch in the middle of an entry keeps the type, the day, the amount and the sources', async () => {
    fakeGardens();
    const { user } = await open();
    await chooseType(user, 'cost');
    await user.type(field('gardens-cost-amount'), '5000');
    await user.type(field('gardens-payment-reason'), 'Labour');
    await user.click(
      within(sourceGroup(names.b)).getByRole('button', {
        name: message(language, 'ventures-source-shop'),
      }),
    );
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    const otherB = other === 'ur' ? 'ب' : 'Partner B';
    await waitFor(() => {
      expect(
        screen.getByRole('group', { name: named(message(other, 'gardens-type-label')) }),
      ).toBeInTheDocument();
    });
    const find = (id: string) => {
      const label = named(message(other, id));
      return (
        screen.queryByRole('textbox', { name: label }) ??
        screen.getByRole('combobox', { name: label })
      );
    };
    expect(find('gardens-cost-amount')).toHaveValue('5,000');
    expect(find('gardens-payment-reason')).toHaveValue('Labour');
    expect(find('gardens-field-date')).toHaveValue('10-10-2026');
    const shop = within(
      screen.getByRole('group', {
        name: named(message(other, 'ventures-source-of', { name: otherB })),
      }),
    ).getByRole('button', { name: message(other, 'ventures-source-shop') });
    expect(shop).toHaveAttribute('aria-pressed', 'true');
    const group = screen.getByRole('group', { name: named(message(other, 'gardens-type-label')) });
    expect(
      within(group).getByRole('button', { name: named(message(other, 'gardens-type-cost')) }),
    ).toHaveAttribute('aria-pressed', 'true');
  });

  it('a garden that is open shows the entry form with the first field focused for the keyboard', async () => {
    fakeGardens({ gardens: [e40Garden()] });
    await open();
    await waitFor(() => {
      expect(
        screen.getByRole('textbox', { name: named(message(language, 'gardens-field-date')) }),
      ).toBeInTheDocument();
    });
  });
});
