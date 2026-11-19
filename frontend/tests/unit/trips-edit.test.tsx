// A saved trip cost or sale in another shop open for edit (plan 44, PRD FR-7.70, FR-7.71, FR-21.4),
// in the Urdu and the English app, against a fake backend: the Edit button of the history opens the
// entry in its form, filled as saved, under the amber band; the review comes from the backend; the
// reason is asked when the entry is not of today; the cash box when the review says so; and a closed
// day reopens first.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as purchasesApi from '../../src/features/purchases/api';
import * as tripsApi from '../../src/features/trips/api';
import * as editApi from '../../src/shared/edit/api';
import { LANGUAGES } from '../../src/i18n';
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
import { noChangeReview } from '../helpers/edit-real';
import { backendReopens, confirmFixDay } from '../helpers/fix-day';
import { CLOSED_DAY_ENTRY } from '../helpers/entry-flags';
import { EMPTY_PAIRS } from '../helpers/paged';
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
vi.mock('../../src/shared/edit/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeItemPickers();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(editApi.auditEditStamp).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;
const startsWith = (text: string) => (name: string) => visible(name).startsWith(text);

/** The answer of a saved edit. */
const EDITED = {
  id: 12,
  number: 1,
  version: 2,
  entry_id: 40,
  reversal_entry_id: 41,
  review: noChangeReview().review,
  edited: {
    at: '',
    local_date: '2026-10-10',
    local_time: '16:35',
    by: 'Owner',
    count: 1,
    reason: '',
  },
} as never;

describe.each(LANGUAGES)('a trip entry open for edit (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    vi.mocked(tripsApi.tripsCostEditPreview).mockResolvedValue(noChangeReview());
    vi.mocked(tripsApi.tripsOtherSaleEditPreview).mockResolvedValue(noChangeReview());
  });

  const open = async () => {
    const view = await renderApp('/trips/1');
    await screen.findByText(said(message(language, 'trips-detail-title', { code: '0070' })));
    return { ...view, user: userEvent.setup() };
  };

  /** Selects the history row of an entry. */
  async function selectRow(user: ReturnType<typeof userEvent.setup>, key: string) {
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>(
        `.ag-row[row-id="${key}"] [col-id="entry"]`,
      );
      if (found === null) throw new Error(`no row ${key}`);
      return found;
    });
    await user.click(row);
  }
  const edit = () =>
    screen.getByRole('button', { name: startsWith(message(language, 'edit-action-edit')) });
  const field = (id: string) => {
    const name = named(message(language, id));
    return screen.queryByRole('textbox', { name }) ?? screen.getByRole('combobox', { name });
  };

  it('Edit is off until a row has focus; then it opens the cost as saved, with the band', async () => {
    fakeTrips({ entries: [costEntry(), purchaseEntry()] });
    const { user } = await open();
    expect(edit()).toBeDisabled();
    await selectRow(user, 'cost-12');
    await user.click(edit());
    expect(
      await screen.findByText(
        startsWith(
          message(language, 'edit-band', {
            what: message(language, 'glossary-safar'),
            number: '1',
            date: '10-10-2026',
          }),
        ),
      ),
    ).toBeInTheDocument();
    expect((field('trips-cost-amount') as HTMLInputElement).value).toBe('4,000');
    // The cursor starts at the first box he can change, not at the day that is read only.
    expect(field('trips-cost-amount')).toHaveFocus();
    expect((field('trips-cost-reason') as HTMLInputElement).value).toBe('Truck');
    // The type list of a new entry gave way to the edit form.
    expect(
      screen.queryByRole('group', { name: named(message(language, 'trips-type-label')) }),
    ).toBeNull();
  });

  it('saves the cost with trips_cost_edit_save: id, version, the cost as typed, no reason for today', async () => {
    fakeTrips({ entries: [costEntry()] });
    vi.mocked(tripsApi.tripsCostEditSave).mockResolvedValue(EDITED);
    const { user } = await open();
    await selectRow(user, 'cost-12');
    await user.click(edit());
    const amount = await screen.findByRole('textbox', {
      name: named(message(language, 'trips-cost-amount')),
    });
    await user.clear(amount);
    await user.type(amount, '4500');
    await waitFor(() => {
      expect(tripsApi.tripsCostEditPreview).toHaveBeenCalled();
    });
    press('F10');
    await waitFor(() => {
      expect(tripsApi.tripsCostEditSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(tripsApi.tripsCostEditSave).mock.calls[0]?.[0]).toMatchObject({
      id: 12,
      version: 1,
      reason: '',
      cost: { trip_id: 1, amount: '4500', reason: 'Truck', confirm_cash_below_zero: false },
    });
    // The page goes back to the form of a new entry.
    expect(
      await screen.findByRole('group', { name: named(message(language, 'trips-type-label')) }),
    ).toBeInTheDocument();
  });

  it('an entry of an earlier day asks why, then saves with the reason', async () => {
    fakeTrips({ entries: [costEntry({ entry_date: '2026-10-09' })] });
    vi.mocked(tripsApi.tripsCostEditPreview).mockResolvedValue(noChangeReview('2026-10-09', true));
    vi.mocked(tripsApi.tripsCostEditSave).mockResolvedValue(EDITED);
    const { user } = await open();
    await selectRow(user, 'cost-12');
    await user.click(edit());
    const amount = await screen.findByRole('textbox', {
      name: named(message(language, 'trips-cost-amount')),
    });
    await user.clear(amount);
    await user.type(amount, '4500');
    await waitFor(() => {
      expect(tripsApi.tripsCostEditPreview).toHaveBeenCalled();
    });
    press('F10');
    const reason = await screen.findByRole('combobox', {
      name: named(message(language, 'edit-reason-label')),
    });
    expect(tripsApi.tripsCostEditSave).not.toHaveBeenCalled();
    await user.type(reason, 'wrong bill{Enter}');
    await waitFor(() => {
      expect(tripsApi.tripsCostEditSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(tripsApi.tripsCostEditSave).mock.calls[0]?.[0]).toMatchObject({
      id: 12,
      reason: 'wrong bill',
    });
  });

  it('the review says the edit takes cash below zero: the box asks first, Yes saves with the answer', async () => {
    fakeTrips({ entries: [costEntry()] });
    const review = noChangeReview();
    vi.mocked(tripsApi.tripsCostEditPreview).mockResolvedValue({
      ...review,
      review: {
        ...review.review,
        effect: {
          ...review.review.effect,
          cash: { before_paisa: 100_000, after_paisa: -50_000, needs_confirm: true },
        },
      },
    } as never);
    vi.mocked(tripsApi.tripsCostEditSave).mockResolvedValue(EDITED);
    const { user } = await open();
    await selectRow(user, 'cost-12');
    await user.click(edit());
    await screen.findByRole('textbox', { name: named(message(language, 'trips-cost-amount')) });
    await waitFor(() => {
      expect(tripsApi.tripsCostEditPreview).toHaveBeenCalled();
    });
    press('F10');
    const dialog = await screen.findByRole('alertdialog');
    expect(tripsApi.tripsCostEditSave).not.toHaveBeenCalled();
    // The box starts on "no": Enter by habit saves nothing.
    await user.click(
      within(dialog).getByRole('button', {
        name: named(message(language, 'common-cash-confirm-yes')),
      }),
    );
    await waitFor(() => {
      expect(tripsApi.tripsCostEditSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(tripsApi.tripsCostEditSave).mock.calls[0]?.[0]).toMatchObject({
      cost: { confirm_cash_below_zero: true },
    });
  });

  it('a sale in another shop opens with its shop, purchase, qty, money and note', async () => {
    fakeTrips({
      entries: [
        costEntry({
          id: 13,
          kind: 'other_sale',
          number: 2,
          amount_paisa: costEntry().amount_paisa,
          reason: 'cash sale',
          shop_name: 'Haji shop',
          qty: 5,
          purchase_id: 7,
          purchase_number: 1,
          parts: [],
          spread: [],
        }),
      ],
    });
    vi.mocked(tripsApi.tripsOtherSaleEditSave).mockResolvedValue(EDITED);
    const { user } = await open();
    await selectRow(user, 'other_sale-13');
    await user.click(edit());
    expect((field('trips-sale-shop') as HTMLInputElement).value).toBe('Haji shop');
    expect((field('trips-field-note') as HTMLInputElement).value).toBe('cash sale');
    await waitFor(() => {
      expect(tripsApi.tripsOtherSaleEditPreview).toHaveBeenCalled();
    });
    press('F10');
    await waitFor(() => {
      expect(tripsApi.tripsOtherSaleEditSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(tripsApi.tripsOtherSaleEditSave).mock.calls[0]?.[0]).toMatchObject({
      id: 13,
      version: 1,
      reason: '',
      sale: { trip_id: 1, shop: 'Haji shop', purchase_id: 7, qty: '5', note: 'cash sale' },
      confirm_cash_below_zero: false,
    });
  });

  it('a closed day: Edit reads "reopen the day and fix" and asks for the reason first', async () => {
    fakeTrips({ entries: [costEntry({ ...CLOSED_DAY_ENTRY, entry_date: '2026-10-05' })] });
    const { user } = await open();
    await selectRow(user, 'cost-12');
    const button = screen.getByRole('button', {
      name: startsWith(message(language, 'edit-action-fix-day')),
    });
    expect(button).toBeEnabled();
    await user.click(button);
    expect(
      await screen.findByRole('combobox', { name: named(message(language, 'edit-fix-label')) }),
    ).toBeInTheDocument();
  });

  it('a closed day: the box reopens the day of the entry as typed, DD-MM-YYYY (Umar, 13-11-2026)', async () => {
    fakeTrips({ entries: [costEntry({ ...CLOSED_DAY_ENTRY, entry_date: '2026-10-05' })] });
    backendReopens();
    const { user } = await open();
    await selectRow(user, 'cost-12');
    await user.click(
      screen.getByRole('button', { name: startsWith(message(language, 'edit-action-fix-day')) }),
    );
    await confirmFixDay(language, user, '05-10-2026');
  });

  it('a locked entry says why in words and Edit is off', async () => {
    fakeTrips({ entries: [costEntry({ can_edit: false, locked_reason: 'goods_moved' })] });
    const { user } = await open();
    await selectRow(user, 'cost-12');
    expect(edit()).toBeDisabled();
    expect(visible(document.body.textContent)).toContain(
      message(language, 'edit-locked-goods-moved'),
    );
  });
});

describe.each(LANGUAGES)('a trip purchase open for edit (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    vi.mocked(purchasesApi.purchasesEditPreview).mockResolvedValue(noChangeReview());
  });

  const open = async () => {
    const view = await renderApp('/trips/1');
    await screen.findByText(said(message(language, 'trips-detail-title', { code: '0070' })));
    return { ...view, user: userEvent.setup() };
  };
  const edit = () =>
    screen.getByRole('button', { name: startsWith(message(language, 'edit-action-edit')) });
  const labelled = (id: string) =>
    screen.findByLabelText<HTMLInputElement>((name) => visible(name) === message(language, id));

  async function openPurchase(user: ReturnType<typeof userEvent.setup>) {
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>(
        '.ag-row[row-id="purchase-11"] [col-id="entry"]',
      );
      if (found === null) throw new Error('no purchase row');
      return found;
    });
    await user.click(row);
    await user.click(edit());
  }

  it('opens as saved: the day read only, the figures as text, the band with the purchase number', async () => {
    fakeTrips({
      purchases: [e34Purchase({ qty_sold: 0, qty_left: 20 })],
      entries: [purchaseEntry()],
    });
    const { user } = await open();
    await openPurchase(user);
    expect((await labelled('purchases-field-qty')).value).toBe('20');
    expect((await labelled('purchases-field-rate')).value).toBe('500');
    expect((await labelled('glossary-kiraya')).value).toBe('400');
    expect((await labelled('purchases-field-supplier')).value).toBe('Haji Nazir');
    expect((await labelled('purchases-field-date')).readOnly).toBe(true);
    expect(visible(document.body.textContent)).toContain(
      message(language, 'edit-band', {
        what: message(language, 'glossary-khareedari'),
        number: message(language, 'purchases-number', { number: 1 }),
        date: '10-10-2026',
      }),
    );
  });

  it('saves with purchases_edit_save: the purchase id and version, the fields as typed, every partner’s source', async () => {
    fakeTrips({
      purchases: [e34Purchase({ qty_sold: 0, qty_left: 20 })],
      entries: [purchaseEntry()],
    });
    vi.mocked(purchasesApi.purchasesEditSave).mockResolvedValue(EDITED);
    const { user } = await open();
    await openPurchase(user);
    const rate = await labelled('purchases-field-rate');
    await user.clear(rate);
    await user.type(rate, '550');
    await waitFor(() => {
      expect(purchasesApi.purchasesEditPreview).toHaveBeenCalled();
    });
    press('F10');
    await waitFor(() => {
      expect(purchasesApi.purchasesEditSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(purchasesApi.purchasesEditSave).mock.calls[0]?.[0];
    expect(sent).toMatchObject({
      id: 7,
      version: 1,
      reason: '',
      purchase: {
        trip_id: 1,
        date: '10-10-2026',
        supplier: 'Haji Nazir',
        item_id: 61,
        packing_id: 71,
        qty: '20',
        rate: '550',
        freight: '400',
        confirm_cash_below_zero: false,
      },
    });
    expect(sent?.purchase.parts).toHaveLength(3);
  });

  it('goods already sold: the entry is locked, Edit is off and the reason is in words', async () => {
    fakeTrips({
      purchases: [e34Purchase()],
      entries: [purchaseEntry({ can_edit: false, locked_reason: 'goods_moved' })],
    });
    const { user } = await open();
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>(
        '.ag-row[row-id="purchase-11"] [col-id="entry"]',
      );
      if (found === null) throw new Error('no purchase row');
      return found;
    });
    await user.click(row);
    expect(edit()).toBeDisabled();
    expect(visible(document.body.textContent)).toContain(
      message(language, 'edit-locked-goods-moved'),
    );
  });
});

describe.each(LANGUAGES)('the header of a trip open for edit (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    vi.mocked(tripsApi.tripsHeaderEditPreview).mockResolvedValue(noChangeReview());
  });

  const open = async () => {
    const view = await renderApp('/trips/1');
    await screen.findByText(said(message(language, 'trips-detail-title', { code: '0070' })));
    return { ...view, user: userEvent.setup() };
  };

  it('opens with the market, who was sent and the note; the day is read only', async () => {
    fakeTrips({ trips: [e34Trip({ market: 'Lahore', sent_person: 'Aslam', note: 'early' })] });
    const { user } = await open();
    await user.click(
      screen.getByRole('button', { name: startsWith(message(language, 'trips-header-edit')) }),
    );
    expect(
      (
        await screen.findByLabelText<HTMLInputElement>(
          said(message(language, 'trips-field-market')),
        )
      ).value,
    ).toBe('Lahore');
    expect(
      screen.getByLabelText<HTMLInputElement>(said(message(language, 'trips-field-sent'))).value,
    ).toBe('Aslam');
    expect(
      screen.getByLabelText<HTMLInputElement>(said(message(language, 'trips-field-date'))).readOnly,
    ).toBe(true);
  });

  it('saves the header with its own day and the partners as they are', async () => {
    fakeTrips({ trips: [e34Trip({ market: 'Lahore' })] });
    vi.mocked(tripsApi.tripsHeaderEditSave).mockResolvedValue(EDITED);
    const { user } = await open();
    await user.click(
      screen.getByRole('button', { name: startsWith(message(language, 'trips-header-edit')) }),
    );
    const market = await screen.findByLabelText<HTMLInputElement>(
      said(message(language, 'trips-field-market')),
    );
    await user.clear(market);
    await user.type(market, 'Multan');
    await waitFor(() => {
      expect(tripsApi.tripsHeaderEditPreview).toHaveBeenCalled();
    });
    press('F10');
    await waitFor(() => {
      expect(tripsApi.tripsHeaderEditSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(tripsApi.tripsHeaderEditSave).mock.calls[0]?.[0];
    expect(sent).toMatchObject({
      id: 1,
      reason: '',
      trip: { date: '10-10-2026', market: 'Multan' },
    });
    expect(sent?.trip.partners).toHaveLength(3);
  });
});
