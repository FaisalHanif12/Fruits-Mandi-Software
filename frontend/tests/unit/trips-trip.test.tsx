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
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { lineCell, lineCellText, linesSection, typePurchaseLine } from '../helpers/goods-lines';
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
  const tripNumber = '0070';
  const purchaseNumber = message(language, 'purchases-number', { number: 1 });

  const open = async (path = '/trips/1') => {
    const view = await renderApp(path);
    await screen.findByText(said(message(language, 'trips-detail-title', { code: tripNumber })));
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

  /** Fills the purchase of E2E-34 as one line of the grid: 20 oranges at 500, freight 400. */
  async function fillPurchase(user: User) {
    await typePurchaseLine(user, 0, {
      item: language === 'ur' ? 'م' : 'Ora',
      packing: language === 'ur' ? 'پ' : 'Box',
      qty: '20',
      rate: '500',
      freight: '400',
    });
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
    // A purchase: a note and a quantity on a line of the grid.
    await user.type(field('trips-field-note'), 'Nasir');
    await user.click(lineCell(0, 'qty'));
    await user.keyboard('20{Enter}');
    expect(lineCellText(0, 'qty')).toBe('20');
    // A trip cost: the day stays, the lines are gone, the amount is typed here.
    await chooseType(user, 'cost');
    expect(linesSection()).toBeNull();
    await user.type(field('trips-cost-amount'), '4000');
    // A sale in another shop: the amount stays, the note comes back as typed, the cost reason goes.
    await chooseType(user, 'other-sale');
    expect(field('trips-sale-amount')).toHaveValue('4,000');
    expect(field('trips-field-note')).toHaveValue('Nasir');
    expect(
      screen.getByRole('textbox', { name: named(message(language, 'trips-field-date')) }),
    ).toHaveValue('10-10-2026');
    // Back on the purchase the lines start again.
    await chooseType(user, 'purchase');
    expect(lineCellText(0, 'qty')).toBe('');
  });

  it('a purchase shows each partner’s part before the save, from the shop or from a pocket', async () => {
    fakeTrips();
    const { user } = await open();
    await fillPurchase(user);
    // The preview asked with one row for every partner, each from the trip’s source.
    await waitFor(() => {
      expect(purchasesApi.purchasesEntryPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({
          trip_id: 1,
          lines: [
            expect.objectContaining({
              qty: '20',
              rate: '500',
              freight: '400',
              item_id: 61,
              packing_id: 71,
            }),
          ],
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
    // The money of the entry is the backend's: paid now 10,400 for the line and for the whole entry.
    expect((await screen.findAllByText('10,400')).length).toBeGreaterThan(0);
    // Partner B pays from his own pocket: his part is recorded only.
    await user.click(
      within(sourceGroup(names.b)).getByRole('button', {
        name: message(language, 'ventures-source-pocket'),
      }),
    );
    await waitFor(() => {
      expect(purchasesApi.purchasesEntryPreview).toHaveBeenLastCalledWith(
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
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toEqual({
      trip_id: 1,
      date: '10-10-2026',
      // Not touched: the date to sell is the day of the purchase, so nothing is sent (FR-12.39).
      sell_date: null,
      supplier: 'Haji Nazir',
      lines: [
        {
          key: expect.any(String) as string,
          item_id: 61,
          packing_id: 71,
          qty: '20',
          weight: null,
          rate: '500',
          freight: '400',
          labour: null,
          other_cost: null,
          note: null,
          // Nothing for another shop: all of it is for our shop (PRD FR-12.41).
          other_qty: null,
          shop_name: null,
        },
      ],
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
    // The next purchase: the supplier stays, the lines are empty.
    await waitFor(() => {
      expect(lineCellText(0, 'qty')).toBe('');
    });
    expect(field('purchases-field-supplier')).toHaveValue('Haji Nazir');
  });

  const oranges = () => ({
    item: language === 'ur' ? 'م' : 'Ora',
    packing: language === 'ur' ? 'پ' : 'Box',
  });

  it('R1, R3: a purchase of three lines is typed with the keyboard only and one F10 saves all three, with the split of each line', async () => {
    // The cash in hand covers the 152,400, so there is no cash question.
    const fake = fakeTrips({ cashBefore: 500_000 });
    const { user } = await open();
    await typePurchaseLine(user, 0, {
      ...oranges(),
      qty: '80',
      rate: '1200',
      freight: '400',
      otherQty: '30',
      shop: 'Haji Shop',
    });
    await typePurchaseLine(user, 1, { ...oranges(), qty: '40', rate: '900' });
    await typePurchaseLine(user, 2, { ...oranges(), qty: '10', rate: '2000', otherQty: '4' });
    // Each line's value is its own: 80 × 1,200, 40 × 900 and 10 × 2,000, and the money of the whole
    // entry is the backend's: 96,400 (with the freight) + 36,000 + 20,000.
    await waitFor(() => {
      expect(lineCellText(2, 'value')).toBe('20,000');
    });
    expect(lineCellText(0, 'value')).toBe('96,000');
    expect(lineCellText(1, 'value')).toBe('36,000');
    // What is left for our shop, line by line: 80 − 30, 40, 10 − 4.
    await waitFor(() => {
      expect([0, 1, 2].map((row) => lineCellText(row, 'ourQty'))).toEqual(['50', '40', '6']);
    });
    expect((await screen.findAllByText('152,400')).length).toBeGreaterThan(0);
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    const sent = fake.saved.entries[0];
    // One call for all the lines, in the order of the grid, and the empty row at the end is not sent.
    expect(sent?.lines.map((line) => line.qty)).toEqual(['80', '40', '10']);
    expect(sent?.lines.map((line) => line.rate)).toEqual(['1200', '900', '2000']);
    expect(sent?.lines.map((line) => line.other_qty)).toEqual(['30', null, '4']);
    expect(sent?.lines.map((line) => line.shop_name)).toEqual(['Haji Shop', null, null]);
    // The partners' sources are chosen once for the whole entry.
    expect(sent?.parts).toHaveLength(3);
    const number = (value: number) => message(language, 'purchases-number', { number: value });
    expect(
      await screen.findByText(
        said(
          message(language, 'trips-saved-purchases', {
            count: 3,
            first: number(2),
            last: number(4),
          }),
        ),
      ),
    ).toBeInTheDocument();
    // The next entry starts with one empty line.
    await waitFor(() => {
      expect(lineCellText(0, 'qty')).toBe('');
    });
  });

  it('D3: Enter stops at the weight of a line whose item is sold by weight, which the backend tells as soon as the item is chosen', async () => {
    fakeTrips();
    const original = vi.mocked(purchasesApi.purchasesEntryPreview).getMockImplementation();
    if (original === undefined) throw new Error('no preview fake');
    // The backend asks for the weight of every line that has none: the item is sold by weight.
    vi.mocked(purchasesApi.purchasesEntryPreview).mockImplementation(async (input) => {
      const answer = await original(input);
      return {
        ...answer,
        errors: [
          ...answer.errors,
          ...input.lines.flatMap((line, index) =>
            line.weight === null
              ? [
                  {
                    code: 'ERR_WEIGHT_REQUIRED' as const,
                    field: `lines[${String(index)}].weight`,
                    values: {},
                  },
                ]
              : [],
          ),
        ],
      };
    });
    const { user } = await open();
    await waitFor(() => {
      lineCell(0, 'item');
    });
    await user.click(lineCell(0, 'item'));
    await user.keyboard(oranges().item);
    await user.keyboard('{Enter}');
    await user.keyboard(oranges().packing);
    await user.keyboard('{Enter}');
    // The answer about the weight must be here before the quantity is finished.
    await waitFor(() => {
      expect(purchasesApi.purchasesEntryPreview).toHaveBeenCalled();
    });
    await act(async () => {
      await vi.mocked(purchasesApi.purchasesEntryPreview).mock.results.at(-1)?.value;
    });
    await settlePreview();
    await user.keyboard('80{Enter}');
    await waitFor(() => {
      expect(document.activeElement?.closest('[col-id]')?.getAttribute('col-id')).toBe('weight');
    });
    await user.keyboard('1234{Enter}');
    await waitFor(() => {
      expect(document.activeElement?.closest('[col-id]')?.getAttribute('col-id')).toBe('rate');
    });
  });

  it('a mistake of one line is on the cell of that line and in words under the form, and every line stays', async () => {
    const fake = fakeTrips();
    fake.refuse.purchase = [
      { code: 'ERR_SPLIT_ABOVE_QTY', field: 'lines[1].other_qty', values: { qty: 40 } },
    ];
    const { user } = await open();
    await typePurchaseLine(user, 0, { ...oranges(), qty: '80', rate: '1200' });
    await typePurchaseLine(user, 1, { ...oranges(), qty: '40', rate: '900', otherQty: '90' });
    press('F10');
    expect(
      await screen.findByText(said(message(language, 'err-split-above-qty', { qty: 40 }))),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(lineCell(1, 'otherQty').className).toContain('cell-error');
    });
    expect(lineCell(0, 'otherQty').className).not.toContain('cell-error');
    expect(lineCell(1, 'qty').className).not.toContain('cell-error');
    expect(lineCellText(0, 'qty')).toBe('80');
    expect(lineCellText(1, 'qty')).toBe('40');
    expect(lineCellText(1, 'otherQty')).toBe('90');
  });

  it('R6: the list of purchases shows what is left for our shop and for another shop', async () => {
    fakeTrips({
      purchases: [e34Purchase({ qty_other: 3, qty_other_left: 3, qty_our_left: 2, qty_left: 5 })],
    });
    await open();
    const row = await waitFor(() => {
      const found = document.querySelector('.ag-row[row-id="7"]');
      if (found === null) throw new Error('no purchase row');
      return found;
    });
    expect(visible(row.querySelector('[col-id="qty_our_left"]')?.textContent)).toBe('2');
    expect(visible(row.querySelector('[col-id="qty_other_left"]')?.textContent)).toBe('3');
  });

  it('R3: change the split on the purchase card: the box starts with the part now, and the new part goes with the version', async () => {
    const fake = fakeTrips({
      purchases: [e34Purchase({ qty_other: 3, qty_other_left: 3, qty_our_left: 2, qty_left: 5 })],
    });
    const { user } = await open();
    const target = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="7"] [col-id="number"]');
      if (found === null) throw new Error('no purchase row');
      return found;
    });
    await user.click(target);
    await user.click(
      await screen.findByRole('button', { name: said(message(language, 'split-change')) }),
    );
    const dialog = await screen.findByRole('dialog');
    const qty = within(dialog).getByRole<HTMLInputElement>('textbox', {
      name: named(message(language, 'split-other-qty')),
    });
    // The box starts with the part now, visible like a typed value (NFR-USE-27).
    expect(qty.value).toBe('3');
    await user.click(qty);
    await user.keyboard('{Control>}a{/Control}4');
    await user.type(
      within(dialog).getByRole('combobox', { name: named(message(language, 'split-shop')) }),
      'Haji Shop',
    );
    await user.click(
      within(dialog).getByRole('button', { name: said(message(language, 'common-save')) }),
    );
    await waitFor(() => {
      expect(fake.saved.splits).toHaveLength(1);
    });
    expect(fake.saved.splits[0]).toEqual({
      id: 7,
      version: 1,
      other_qty: '4',
      shop_name: 'Haji Shop',
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
  });

  it('R3: a part above the quantity is refused in the box, at the field, and what was typed stays', async () => {
    fakeTrips();
    const { user } = await open();
    const target = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="7"] [col-id="number"]');
      if (found === null) throw new Error('no purchase row');
      return found;
    });
    await user.click(target);
    await user.click(
      await screen.findByRole('button', { name: said(message(language, 'split-change')) }),
    );
    const dialog = await screen.findByRole('dialog');
    const qty = within(dialog).getByRole<HTMLInputElement>('textbox', {
      name: named(message(language, 'split-other-qty')),
    });
    await user.click(qty);
    await user.keyboard('{Control>}a{/Control}90');
    await user.click(
      within(dialog).getByRole('button', { name: said(message(language, 'common-save')) }),
    );
    expect(
      await within(dialog).findByText(said(message(language, 'err-split-above-qty', { qty: 20 }))),
    ).toBeInTheDocument();
    expect(qty.value).toBe('90');
  });

  it('FR-12.39: the date to sell shows the day of the purchase, follows it, and a day typed over it goes with the purchase', async () => {
    const fake = fakeTrips();
    const { user } = await open();
    const sellDate = () => field('glossary-sell-date');
    // The box is filled with the day of the purchase, like a typed value (NFR-USE-27).
    expect(sellDate()).toHaveValue('10-10-2026');
    // It follows the purchase day while nobody has typed in it.
    const day = field('trips-field-date');
    await user.click(day);
    await user.keyboard('{Control>}a{/Control}11-10-2026');
    await waitFor(() => {
      expect(sellDate()).toHaveValue('11-10-2026');
    });
    await user.click(day);
    await user.keyboard('{Control>}a{/Control}10-10-2026');
    await fillPurchase(user);
    // A later day typed over it stays, and is sent as typed.
    await user.click(sellDate());
    await user.keyboard('{Control>}a{/Control}12-10-2026');
    await waitFor(() => {
      expect(sellDate()).toHaveValue('12-10-2026');
    });
    await screen.findByRole('list', { name: named(message(language, 'trips-parts-title')) });
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toMatchObject({ date: '10-10-2026', sell_date: '12-10-2026' });
    // The next purchase starts with the box on its own day again.
    await waitFor(() => {
      expect(sellDate()).toHaveValue('10-10-2026');
    });
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
      expect(purchasesApi.purchasesEntryPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ lines: [expect.objectContaining({ freight: '400' })] }),
      );
    });
    await act(async () => {
      await vi.mocked(purchasesApi.purchasesEntryPreview).mock.results.at(-1)?.value;
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
    expect(purchasesApi.purchasesEntrySave).not.toHaveBeenCalled();
    press('F10');
    await user.click(
      await screen.findByRole('button', { name: startsWith('common-cash-confirm-yes') }),
    );
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]?.confirm_cash_below_zero).toBe(true);
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
      expect(fake.saved.entries).toHaveLength(1);
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
    expect(lineCellText(0, 'qty')).toBe('20');
    expect(lineCellText(0, 'rate')).toBe('500');
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
    // The 15 are set for another shop, so the purchase is offered (PRD FR-12.41).
    const fake = fakeTrips({
      purchases: [
        e34Purchase({
          qty_other: 15,
          qty_our_sold: 0,
          qty_our_left: 0,
          qty_other_left: 15,
          qty_left: 15,
        }),
      ],
    });
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

  it('FR-12.39: the purchase card shows the date to sell, and "change" asks for a day and sends it with the version', async () => {
    fakeTrips();
    vi.mocked(purchasesApi.purchasesSetSellDate).mockImplementation((input) =>
      Promise.resolve({ ...input, version: input.version + 1 }),
    );
    const { user } = await open();
    const target = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="7"] [col-id="number"]');
      if (found === null) throw new Error('no purchase row');
      return found;
    });
    await user.click(target);
    const card = (
      await screen.findByText(
        said(
          message(language, 'purchases-detail-title', {
            number: purchaseNumber,
            item: language === 'ur' ? 'مالٹا' : 'Orange',
          }),
        ),
      )
    ).closest('section');
    if (card === null) throw new Error('no purchase card');
    // The day to sell is a fact of the card: it starts as the day of the purchase.
    expect(visible(card.textContent)).toContain('10-10-2026');
    await user.click(
      await screen.findByRole('button', {
        name: said(message(language, 'purchases-change-sell-date')),
      }),
    );
    const dialog = await screen.findByRole('dialog');
    const box = within(dialog).getByRole<HTMLInputElement>('textbox', {
      name: named(message(language, 'glossary-sell-date')),
    });
    // The box starts with the day it has now, visible like a typed value.
    expect(box.value).toBe('10-10-2026');
    await user.click(box);
    await user.keyboard('{Control>}a{/Control}13-10-2026');
    await user.click(
      within(dialog).getByRole('button', { name: said(message(language, 'common-save')) }),
    );
    await waitFor(() => {
      expect(purchasesApi.purchasesSetSellDate).toHaveBeenCalledWith({
        id: 7,
        version: 1,
        sell_date: '13-10-2026',
      });
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
  });

  it('FR-12.39: a refusal of the day stays in the box with the words at the field, and the typed day is kept', async () => {
    fakeTrips();
    vi.mocked(purchasesApi.purchasesSetSellDate).mockRejectedValue(
      fail({
        code: 'ERR_SELL_DATE_BEFORE_ENTRY',
        field: 'sell_date',
        values: { entry_date: '10-10-2026' },
      }),
    );
    const { user } = await open();
    const target = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="7"] [col-id="number"]');
      if (found === null) throw new Error('no purchase row');
      return found;
    });
    await user.click(target);
    await user.click(
      await screen.findByRole('button', {
        name: said(message(language, 'purchases-change-sell-date')),
      }),
    );
    const dialog = await screen.findByRole('dialog');
    const box = within(dialog).getByRole<HTMLInputElement>('textbox', {
      name: named(message(language, 'glossary-sell-date')),
    });
    await user.click(box);
    await user.keyboard('{Control>}a{/Control}09-10-2026');
    await user.click(
      within(dialog).getByRole('button', { name: said(message(language, 'common-save')) }),
    );
    expect(
      await within(dialog).findByText(
        said(message(language, 'err-sell-date-before-entry', { entry_date: '10-10-2026' })),
      ),
    ).toBeInTheDocument();
    expect(box.value).toBe('09-10-2026');
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
    // The grid is made again for the other direction, from the lines in the store.
    await waitFor(() => {
      expect(lineCellText(0, 'qty')).toBe('20');
    });
    expect(lineCellText(0, 'rate')).toBe('500');
    expect(lineCellText(0, 'freight')).toBe('400');
    expect(lineCellText(0, 'item')).toBe(other === 'ur' ? 'مالٹا' : 'Orange');
    expect(
      screen.getByRole('combobox', { name: named(message(other, 'purchases-field-supplier')) }),
    ).toHaveValue('Haji Nazir');
    const pocket = within(
      screen.getByRole('group', {
        name: named(message(other, 'ventures-source-of', { name: otherNames.b })),
      }),
    ).getByRole('button', { name: message(other, 'ventures-source-pocket') });
    expect(pocket).toHaveAttribute('aria-pressed', 'true');
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
