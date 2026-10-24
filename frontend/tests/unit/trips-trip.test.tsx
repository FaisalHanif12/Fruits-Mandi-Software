// One trip (plan 16b, PRD 12.10, FR-7.67, FR-12.24 to FR-12.33, FR-6.7), in the Urdu and the English
// app, against a fake backend with the numbers of E2E-34: the facts and the result of the trip, the
// type list that keeps the day, the amount and the note, each partner's part before the save (shop
// part off his total, pocket part recorded only), the cash question, the three kinds of entry, the
// write-off of spoiled goods and its box, the cancel of a cost, a refusal, and a language switch in
// the middle of an entry that loses nothing. Every figure on the screen is a figure the backend
// sent; the fake's sums are written in the fixture from the plan.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as purchasesApi from '../../src/features/purchases/api';
import * as tripsApi from '../../src/features/trips/api';
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
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import {
  costEntry,
  e34Purchase,
  e34Trip,
  fakeItemPickers,
  fakeTrips,
  purchaseEntry,
} from '../helpers/trips';
import { EMPTY_PAIRS } from '../helpers/paged';
import { settlePreview } from '../helpers/preview-settled';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/purchases/api');
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
  fakeItemPickers();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;
type User = ReturnType<typeof userEvent.setup>;

describe.each(LANGUAGES)('one trip (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const names = {
    a: language === 'ur' ? 'الف' : 'Partner A',
    b: language === 'ur' ? 'ب' : 'Partner B',
  };
  const tripNumber = message(language, 'trips-number', { number: 1 });
  const purchaseNumber = message(language, 'purchases-number', { number: 1 });

  const open = async (path = '/trips/1') => {
    const view = await renderApp(path);
    await screen.findByText(said(message(language, 'trips-detail-title', { number: tripNumber })));
    return { ...view, user: userEvent.setup() };
  };

  const typeList = () =>
    screen.getByRole('group', { name: named(message(language, 'trips-type-label')) });
  const chooseType = async (user: User, type: 'purchase' | 'cost' | 'other-sale') => {
    await user.click(
      within(typeList()).getByRole('button', {
        name: named(message(language, `trips-type-${type}`)),
      }),
    );
  };
  /** A text field by its label: a plain box, or the Urdu text field (a combobox). */
  const field = (id: string) => {
    const name = named(message(language, id));
    return screen.queryByRole('textbox', { name }) ?? screen.getByRole('combobox', { name });
  };
  const combo = field;
  const sourceGroup = (name: string) =>
    screen.getByRole('group', { name: named(message(language, 'ventures-source-of', { name })) });

  /** Picks the one orange or the one box from a master picker. */
  async function pick(user: User, id: string, text: string) {
    const box = combo(id);
    await user.click(box);
    await user.type(box, text);
    await user.click(await screen.findByRole('option'));
  }

  /** Fills the purchase of E2E-34: 20 oranges at 500, freight 400. */
  async function fillPurchase(user: User) {
    await pick(user, 'glossary-qism-maal', language === 'ur' ? 'م' : 'Ora');
    await pick(user, 'glossary-packing', language === 'ur' ? 'پ' : 'Box');
    await user.type(field('purchases-field-qty'), '20');
    await user.type(field('purchases-field-rate'), '500');
    await user.type(field('glossary-kiraya'), '400');
  }

  it('shows the trip: its number, the state, the partners with their %, and the result with its working', async () => {
    fakeTrips();
    await open();
    expect(screen.getByText(said(message(language, 'trips-status-open')))).toBeInTheDocument();
    const partners = screen.getByRole('list', {
      name: named(message(language, 'trips-partners-title')),
    });
    expect(within(partners).getAllByRole('listitem')).toHaveLength(3);
    expect(visible(partners.textContent)).toContain('40%');
    // The result of E2E-34 (660) is the backend's, with its ⓘ.
    const profit = screen.getAllByText(said(message(language, 'glossary-tijarati-munafa')))[0];
    expect(profit?.closest('div')?.textContent).toContain('660');
    expect(profit?.closest('div')?.querySelector('button')).not.toBeNull();
    // Each partner's money: shop 4,160, back 3,384, net −776.
    const table = screen.getByRole('table', {
      name: named(message(language, 'trips-partner-results-title')),
    });
    expect(visible(table.textContent)).toContain('4,160');
    expect(visible(table.textContent)).toContain('3,384');
    expect(visible(table.textContent)).toContain('-776');
    expect(tripsApi.tripsGet).toHaveBeenCalledWith({
      id: 1,
      entries_offset: 0,
      entries_limit: 100,
    });
  });

  it('a closed trip takes no entry: a note instead of the form', async () => {
    fakeTrips({
      trips: [e34Trip({ status: 'closed', closed_date: '2026-10-11', qty_left: 0 })],
    });
    await open();
    expect(screen.getByText(said(message(language, 'trips-closed-notice')))).toBeInTheDocument();
    expect(
      screen.queryByRole('group', { name: named(message(language, 'trips-type-label')) }),
    ).toBeNull();
  });

  it('an unknown trip says so with a retry button', async () => {
    fakeTrips();
    await renderApp('/trips/99');
    expect(
      await screen.findByText(said(message(language, 'trips-detail-failed'))),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: message(language, 'common-retry') }),
    ).toBeInTheDocument();
  });

  it('the type list has the three types; changing it keeps the day, the amount and the note', async () => {
    fakeTrips();
    const { user } = await open();
    const buttons = within(typeList()).getAllByRole('button');
    expect(buttons.map((button) => visible(button.textContent))).toEqual([
      message(language, 'trips-type-purchase'),
      message(language, 'trips-type-cost'),
      message(language, 'trips-type-other-sale'),
    ]);
    // A purchase: a note and a quantity.
    await user.type(field('trips-field-note'), 'Nasir');
    await user.type(field('purchases-field-qty'), '20');
    // A trip cost: the day stays, the quantity is gone, the amount is typed here.
    await chooseType(user, 'cost');
    expect(
      screen.queryByRole('textbox', { name: named(message(language, 'purchases-field-qty')) }),
    ).toBeNull();
    await user.type(field('trips-cost-amount'), '4000');
    // A sale in another shop: the amount stays, the note comes back as typed, the cost reason goes.
    await chooseType(user, 'other-sale');
    expect(field('trips-sale-amount')).toHaveValue('4,000');
    expect(field('trips-field-note')).toHaveValue('Nasir');
    expect(
      screen.getByRole('textbox', { name: named(message(language, 'trips-field-date')) }),
    ).toHaveValue('10-10-2026');
    // Back on the purchase the quantity starts again.
    await chooseType(user, 'purchase');
    expect(field('purchases-field-qty')).toHaveValue('');
  });

  it('a purchase shows each partner’s part before the save, from the shop or from a pocket', async () => {
    fakeTrips();
    const { user } = await open();
    await fillPurchase(user);
    // The preview asked with one row for every partner, each from the trip’s source.
    await waitFor(() => {
      expect(purchasesApi.purchasesPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({
          trip_id: 1,
          qty: '20',
          rate: '500',
          freight: '400',
          item_id: 61,
          packing_id: 71,
          parts: [
            { partner_id: 1, source: 'shop' },
            { partner_id: 2, source: 'shop' },
            { partner_id: 3, source: 'shop' },
          ],
        }),
      );
    });
    const parts = await screen.findByRole('list', {
      name: named(message(language, 'trips-parts-title')),
    });
    const text = visible(parts.textContent);
    for (const amount of ['4,160', '3,120']) expect(text).toContain(amount);
    expect(within(parts).getAllByText(said(message(language, 'ventures-part-shop')))).toHaveLength(
      3,
    );
    // The cost lines are the backend's: value 10,000, paid now 10,400.
    expect(screen.getByText('10,000')).toBeInTheDocument();
    // Partner B pays from his own pocket: his part is recorded only.
    await user.click(
      within(sourceGroup(names.b)).getByRole('button', {
        name: message(language, 'ventures-source-pocket'),
      }),
    );
    await waitFor(() => {
      expect(purchasesApi.purchasesPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({
          parts: [
            { partner_id: 1, source: 'shop' },
            { partner_id: 2, source: 'pocket' },
            { partner_id: 3, source: 'shop' },
          ],
        }),
      );
    });
    await waitFor(() => {
      expect(
        within(
          screen.getByRole('list', { name: named(message(language, 'trips-parts-title')) }),
        ).getAllByText(said(message(language, 'ventures-part-pocket'))),
      ).toHaveLength(1);
    });
  });

  it('F10 saves the purchase as typed, tells the number and starts the next one', async () => {
    const fake = fakeTrips();
    const { user } = await open();
    await user.type(field('purchases-field-supplier'), 'Haji Nazir');
    await fillPurchase(user);
    await screen.findByRole('list', { name: named(message(language, 'trips-parts-title')) });
    press('F10');
    await waitFor(() => {
      expect(fake.saved.purchases).toHaveLength(1);
    });
    expect(fake.saved.purchases[0]).toEqual({
      trip_id: 1,
      date: '10-10-2026',
      supplier: 'Haji Nazir',
      item_id: 61,
      packing_id: 71,
      qty: '20',
      weight: null,
      rate: '500',
      freight: '400',
      labour: null,
      other_cost: null,
      note: null,
      parts: [
        { partner_id: 1, source: 'shop' },
        { partner_id: 2, source: 'shop' },
        { partner_id: 3, source: 'shop' },
      ],
      confirm_cash_below_zero: false,
    });
    expect(
      await screen.findByText(
        said(
          message(language, 'trips-saved-purchase', {
            number: message(language, 'purchases-number', { number: 2 }),
          }),
        ),
      ),
    ).toBeInTheDocument();
    // The next purchase: the supplier stays, the rest is empty.
    await waitFor(() => {
      expect(field('purchases-field-qty')).toHaveValue('');
    });
    expect(field('purchases-field-supplier')).toHaveValue('Haji Nazir');
  });

  it('cash going below zero shows the amber note and asks first; No saves nothing, Yes saves', async () => {
    const fake = fakeTrips({ cashBefore: 5_000 });
    const { user } = await open();
    await fillPurchase(user);
    expect(
      await screen.findByText(said(message(language, 'common-cash-confirm-title'))),
    ).toBeInTheDocument();
    // F10 asks first only when the answer on screen is for what is typed now (a newer preview in
    // flight makes the save go without the question, and the backend's refusal opens the box). So
    // wait until the answer to the last key has landed.
    await waitFor(() => {
      expect(purchasesApi.purchasesPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ freight: '400' }),
      );
    });
    await act(async () => {
      await vi.mocked(purchasesApi.purchasesPreview).mock.results.at(-1)?.value;
    });
    await settlePreview();
    press('F10');
    const startsWith = (id: string) => (name: string) =>
      visible(name).startsWith(message(language, id));
    const no = await screen.findByRole('button', { name: startsWith('common-cash-confirm-no') });
    // Focus starts on "No", so Enter out of habit never saves a mistake.
    expect(no).toHaveFocus();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(
        screen.queryByRole('button', { name: startsWith('common-cash-confirm-no') }),
      ).toBeNull();
    });
    expect(purchasesApi.purchasesSave).not.toHaveBeenCalled();
    press('F10');
    await user.click(
      await screen.findByRole('button', { name: startsWith('common-cash-confirm-yes') }),
    );
    await waitFor(() => {
      expect(fake.saved.purchases).toHaveLength(1);
    });
    expect(fake.saved.purchases[0]?.confirm_cash_below_zero).toBe(true);
  });

  it('with every part from a pocket the shop’s cash is not touched: no question', async () => {
    const fake = fakeTrips({ cashBefore: 5_000 });
    const { user } = await open();
    await fillPurchase(user);
    for (const name of [names.a, names.b, language === 'ur' ? 'ج' : 'Partner C']) {
      await user.click(
        within(sourceGroup(name)).getByRole('button', {
          name: message(language, 'ventures-source-pocket'),
        }),
      );
    }
    await waitFor(() => {
      expect(
        screen.getByText(said(message(language, 'ventures-parts-no-posting'))),
      ).toBeInTheDocument();
    });
    press('F10');
    await waitFor(() => {
      expect(fake.saved.purchases).toHaveLength(1);
    });
    expect(
      screen.queryByRole('button', { name: message(language, 'common-cash-confirm-yes') }),
    ).toBeNull();
  });

  it('a refusal of the save is told in plain words and nothing typed is lost', async () => {
    const fake = fakeTrips();
    fake.refuse.purchase = [{ code: 'ERR_TRIP_CLOSED', field: 'trip_id' }];
    const { user } = await open();
    await fillPurchase(user);
    press('F10');
    expect(await screen.findByText(said(message(language, 'err-trip-closed')))).toBeInTheDocument();
    expect(field('purchases-field-qty')).toHaveValue('20');
    expect(field('purchases-field-rate')).toHaveValue('500');
  });

  it('a trip cost shows how it is spread over the purchases and each partner’s part, then saves', async () => {
    const fake = fakeTrips();
    const { user } = await open();
    await chooseType(user, 'cost');
    await user.type(field('trips-cost-amount'), '4000');
    await user.type(combo('trips-cost-reason'), 'Truck');
    const spread = await screen.findByRole('list', {
      name: named(message(language, 'trips-cost-spread-title')),
    });
    expect(visible(spread.textContent)).toContain(purchaseNumber);
    expect(visible(spread.textContent)).toContain('4,000');
    expect(
      await screen.findByRole('list', { name: named(message(language, 'trips-parts-title')) }),
    ).toBeInTheDocument();
    press('F10');
    await waitFor(() => {
      expect(fake.saved.costs).toHaveLength(1);
    });
    expect(fake.saved.costs[0]).toEqual({
      trip_id: 1,
      date: '10-10-2026',
      amount: '4000',
      reason: 'Truck',
      parts: [
        { partner_id: 1, source: 'shop' },
        { partner_id: 2, source: 'shop' },
        { partner_id: 3, source: 'shop' },
      ],
      confirm_cash_below_zero: false,
    });
  });

  it('a sale in another shop shows the cost, the result and each partner’s part, then saves', async () => {
    const fake = fakeTrips();
    const { user } = await open();
    await chooseType(user, 'other-sale');
    await user.click(
      screen.getByRole('combobox', { name: named(message(language, 'trips-sale-purchase')) }),
    );
    await user.click(await screen.findByRole('option'));
    await user.type(combo('trips-sale-shop'), 'Rehmat Fruit');
    await user.type(field('trips-sale-qty'), '15');
    await user.type(field('trips-sale-amount'), '8460');
    const card = (
      await screen.findByText(said(message(language, 'trips-sale-credits-title')))
    ).closest('section');
    expect(card).not.toBeNull();
    const text = visible(card?.textContent);
    // E2E-34: the money 8,460 → 3,384 / 2,538 / 2,538, cost 7,800, result 660.
    for (const figure of ['3,384', '2,538', '7,800', '660', '8,460'])
      expect(text).toContain(figure);
    // A sale in another shop only brings money in, so the cash note never shows.
    expect(screen.queryByText(said(message(language, 'common-cash-confirm-title')))).toBeNull();
    press('F10');
    await waitFor(() => {
      expect(fake.saved.sales).toHaveLength(1);
    });
    expect(fake.saved.sales[0]).toEqual({
      trip_id: 1,
      date: '10-10-2026',
      shop: 'Rehmat Fruit',
      purchase_id: 7,
      qty: '15',
      amount: '8460',
      note: null,
    });
  });

  it('the purchase card has the write-off: the quantity and a reason go to the backend', async () => {
    fakeTrips();
    const { user } = await open();
    const target = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="7"] [col-id="number"]');
      if (found === null) throw new Error('no purchase row');
      return found;
    });
    await user.click(target);
    await screen.findByText(
      said(
        message(language, 'purchases-detail-title', {
          number: purchaseNumber,
          item: language === 'ur' ? 'مالٹا' : 'Orange',
        }),
      ),
    );
    await user.click(
      await screen.findByRole('button', { name: message(language, 'purchases-write-off') }),
    );
    const qty = await screen.findByRole('textbox', {
      name: named(message(language, 'purchases-write-off-qty')),
    });
    await user.type(qty, '5');
    await user.type(combo('purchases-write-off-reason'), 'Spoiled');
    await user.click(
      screen.getByRole('button', { name: message(language, 'purchases-write-off-confirm') }),
    );
    await waitFor(() => {
      expect(purchasesApi.purchasesWriteOff).toHaveBeenCalledWith(7, '5', 'Spoiled');
    });
  });

  it('the history lists the entries; a trip cost can be cancelled with a reason', async () => {
    fakeTrips({ entries: [costEntry(), purchaseEntry()] });
    const { user } = await open();
    const costKey = `cost-12`;
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>(
        `.ag-row[row-id="${costKey}"] [col-id="entry"]`,
      );
      if (found === null) throw new Error('no cost row');
      return found;
    });
    expect(visible(row.textContent)).toBe(
      message(language, 'trips-entry-label', {
        kind: message(language, 'trips-type-cost'),
        number: message(language, 'trips-cost-number', { number: 1 }),
      }),
    );
    await user.click(row);
    await user.click(
      screen.getByRole('button', { name: message(language, 'trips-entries-cancel') }),
    );
    await user.type(combo('trips-cancel-reason'), 'Wrong amount');
    await user.click(
      screen.getByRole('button', { name: message(language, 'trips-cancel-confirm') }),
    );
    await waitFor(() => {
      expect(tripsApi.tripsCostCancel).toHaveBeenCalledWith({
        id: 12,
        version: 1,
        reason: 'Wrong amount',
        confirm_cash_below_zero: false,
      });
    });
  });

  it('a purchase that sold goods cannot be cancelled: the button is dimmed', async () => {
    fakeTrips({ entries: [purchaseEntry({ can_cancel: false })] });
    const { user } = await open();
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>(
        '.ag-row[row-id="purchase-11"] [col-id="entry"]',
      );
      if (found === null) throw new Error('no purchase row');
      return found;
    });
    await user.click(row);
    expect(
      screen.getByRole('button', { name: message(language, 'trips-entries-cancel') }),
    ).toBeDisabled();
  });

  it('the history loads page by page', async () => {
    const many = Array.from({ length: 130 }, (_, index) =>
      costEntry({ id: 200 + index, number: index + 1 }),
    );
    fakeTrips({ entries: many });
    const { user } = await open();
    await waitFor(() => {
      expect(
        screen.getByText(
          said(message(language, 'trips-entries-count', { shown: 100, total: 130 })),
        ),
      ).toBeInTheDocument();
    });
    await user.click(screen.getByRole('button', { name: message(language, 'trips-entries-more') }));
    await waitFor(() => {
      expect(
        screen.getByText(
          said(message(language, 'trips-entries-count', { shown: 130, total: 130 })),
        ),
      ).toBeInTheDocument();
    });
    expect(tripsApi.tripsGet).toHaveBeenLastCalledWith({
      id: 1,
      entries_offset: 100,
      entries_limit: 100,
    });
  });

  it('a trip with nothing bought says so and tells what to do', async () => {
    fakeTrips({ trips: [e34Trip({ purchase_count: 0 })], purchases: [], entries: [], results: [] });
    await open();
    expect(screen.getByText(said(message(language, 'trips-purchases-empty')))).toBeInTheDocument();
    expect(screen.getByText(said(message(language, 'trips-entries-empty')))).toBeInTheDocument();
    expect(
      screen.getByText(said(message(language, 'trips-partner-results-empty'))),
    ).toBeInTheDocument();
  });

  it('a language switch in the middle of an entry keeps every value and every choice', async () => {
    fakeTrips();
    const { user } = await open();
    await user.type(field('purchases-field-supplier'), 'Haji Nazir');
    await fillPurchase(user);
    await user.click(
      within(sourceGroup(names.b)).getByRole('button', {
        name: message(language, 'ventures-source-pocket'),
      }),
    );
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    const otherNames = { b: other === 'ur' ? 'ب' : 'Partner B' };
    await waitFor(() => {
      expect(
        screen.getByRole('textbox', { name: named(message(other, 'purchases-field-qty')) }),
      ).toHaveValue('20');
    });
    expect(
      screen.getByRole('textbox', { name: named(message(other, 'purchases-field-rate')) }),
    ).toHaveValue('500');
    expect(
      screen.getByRole('combobox', { name: named(message(other, 'purchases-field-supplier')) }),
    ).toHaveValue('Haji Nazir');
    const pocket = within(
      screen.getByRole('group', {
        name: named(message(other, 'ventures-source-of', { name: otherNames.b })),
      }),
    ).getByRole('button', { name: message(other, 'ventures-source-pocket') });
    expect(pocket).toHaveAttribute('aria-pressed', 'true');
    // The item and the packing are still chosen (their names come in the other language).
    expect(
      screen.getByRole('combobox', { name: named(message(other, 'glossary-qism-maal')) }),
    ).toHaveValue(other === 'ur' ? 'مالٹا' : 'Orange');
  });

  it('the keyboard alone does a trip cost: type the amount and the reason, F10', async () => {
    const fake = fakeTrips();
    const { user } = await open();
    await chooseType(user, 'cost');
    field('trips-cost-amount').focus();
    await user.keyboard('4000{Enter}Truck');
    press('F10');
    await waitFor(() => {
      expect(fake.saved.costs).toHaveLength(1);
    });
    expect(fake.saved.costs[0]?.amount).toBe('4000');
    expect(fake.saved.costs[0]?.reason).toBe('Truck');
  });
});

describe('purchases of other trips', () => {
  it('the purchase fake lists only goods that are left for the sale book picker', async () => {
    fakeTrips({ purchases: [e34Purchase(), e34Purchase({ id: 9, qty_left: 0 })] });
    const list = await purchasesApi.purchasesList({
      trip_id: null,
      from: null,
      to: null,
      item_id: null,
      state: 'in_stock',
      offset: 0,
      limit: 0,
    });
    expect(list.rows.map((row) => row.id)).toEqual([7]);
  });
});
