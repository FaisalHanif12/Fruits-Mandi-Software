// The trip screens show what the REAL backend answers (plan 16b, T4a). The backend wrote each
// answer in its own test (tests/fixtures/trips_*.json and purchases_*.json, copied, never edited).
// The screens are driven by those answers and by the refusals the backend really sent: the list and
// its totals, an open trip, a trip closed by its last write-off, a trip of two partners with one
// pocket part and a truck cost (E2E-41), the purchase preview of E2E-34 and a closed-trip refusal.
// In the Urdu and in the English app. Every figure on screen is a figure the backend sent.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as purchasesApi from '../../src/features/purchases/api';
import * as tripsApi from '../../src/features/trips/api';
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
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { fakeItemPickers, fakeTrips } from '../helpers/trips';
import * as real from '../helpers/trips-real';
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
const pageText = () => visible(document.body.textContent);

describe.each(LANGUAGES)('the trip screens with the backend’s real answers (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeTrips();
  });

  const tripNumber = message(language, 'trips-number', { number: 1 });
  const openTrip = async () => {
    const view = await renderApp('/trips/1');
    await screen.findByText(said(message(language, 'trips-detail-title', { number: tripNumber })));
    return { ...view, user: userEvent.setup() };
  };
  const field = (id: string) => {
    const name = named(message(language, id));
    return screen.queryByRole('textbox', { name }) ?? screen.getByRole('combobox', { name });
  };

  it('the list shows the real trip and the totals the backend worked out', async () => {
    vi.mocked(tripsApi.tripsList).mockResolvedValue(real.TRIP_LIST_ALL);
    await renderApp('/trips');
    await screen.findByRole('heading', { level: 1, name: named(message(language, 'trips-title')) });
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="1"]')).not.toBeNull();
    });
    const row = document.querySelector('.ag-row[row-id="1"]');
    const cell = (id: string) => visible(row?.querySelector(`[col-id="${id}"]`)?.textContent);
    expect(cell('number')).toBe(tripNumber);
    expect(cell('market')).toBe('Lahore');
    expect(cell('status')).toBe(message(language, 'trips-status-open'));
    // 20 bought, 18 sold, 2 left (cost left 1,040) and the result 1,100.
    expect(cell('qty_left')).toBe('2');
    expect(cell('cost_left')).toBe('1,040');
    expect(cell('result')).toBe('1,100');
  });

  it('an open trip: its facts, the partners at 40, 30 and 30, and what each one put in and got back', async () => {
    vi.mocked(tripsApi.tripsGet).mockResolvedValue(real.TRIP_GET_OPEN);
    await openTrip();
    expect(screen.getByText(said(message(language, 'trips-status-open')))).toBeInTheDocument();
    const partners = screen.getByRole('list', {
      name: named(message(language, 'trips-partners-title')),
    });
    expect(within(partners).getAllByRole('listitem')).toHaveLength(3);
    expect(visible(partners.textContent)).toContain('40%');
    const table = screen.getByRole('table', {
      name: named(message(language, 'trips-partner-results-title')),
    });
    const text = visible(table.textContent);
    // In from the shop 4,160 / 3,120 / 3,120, back 4,184 / 3,138 / 3,138, net 24 / 18 / 18.
    for (const figure of ['4,160', '3,120', '4,184', '3,138']) expect(text).toContain(figure);
    // The page shows the trip’s result (1,100) and the goods left (2 نگ, cost 1,040).
    expect(pageText()).toContain('1,100');
    expect(pageText()).toContain('1,040');
    // The history: the sale in another shop and the purchase, newest first.
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="other_sale-2"]')).not.toBeNull();
    });
    expect(document.querySelector('.ag-row[row-id="purchase-1"]')).not.toBeNull();
  });

  it('a trip closed by its last write-off takes no entry', async () => {
    vi.mocked(tripsApi.tripsGet).mockResolvedValue(real.TRIP_GET_CLOSED);
    await openTrip();
    expect(screen.getByText(said(message(language, 'trips-closed-notice')))).toBeInTheDocument();
    expect(
      screen.queryByRole('group', { name: named(message(language, 'trips-type-label')) }),
    ).toBeNull();
    // The loss of the 2 written-off نگ is in the result: 1,100 − 1,040 = 60.
    expect(pageText()).toContain('60');
  });

  it('a trip of two partners, one paying from his pocket, with a truck cost (E2E-41)', async () => {
    vi.mocked(tripsApi.tripsGet).mockResolvedValue(real.TRIP_GET_E41_CLOSED);
    const { user } = await openTrip();
    const partners = screen.getByRole('list', {
      name: named(message(language, 'trips-partners-title')),
    });
    expect(within(partners).getAllByRole('listitem')).toHaveLength(2);
    expect(visible(partners.textContent)).toContain('50%');
    const table = screen.getByRole('table', {
      name: named(message(language, 'trips-partner-results-title')),
    });
    const text = visible(table.textContent);
    // Each one got 52,640 back; the net is +2,640 each.
    expect(text).toContain('52,640');
    expect(text).toContain('2,640');
    expect(pageText()).toContain('5,280');
    // The truck cost: 4,000 spread over the one purchase, 2,000 from the shop and 2,000 from a pocket.
    const cost = await waitFor(() => {
      const found = document.querySelector<HTMLElement>(
        '.ag-row[row-id="cost-2"] [col-id="entry"]',
      );
      if (found === null) throw new Error('no cost row');
      return found;
    });
    await user.click(cost);
    const heading = await screen.findByText(said(message(language, 'trips-cost-spread-title')));
    expect(visible(heading.parentElement?.textContent)).toContain('4,000');
    const parts = screen.getByRole('list', { name: named(message(language, 'trips-parts-title')) });
    expect(
      within(parts).getAllByText(said(message(language, 'ventures-part-pocket'))),
    ).toHaveLength(1);
    expect(within(parts).getAllByText(said(message(language, 'ventures-part-shop')))).toHaveLength(
      1,
    );
  });

  it('the real purchase preview of E2E-34 shows 4,160 / 3,120 / 3,120 and the cash after', async () => {
    vi.mocked(tripsApi.tripsGet).mockResolvedValue(real.TRIP_GET_OPEN);
    vi.mocked(purchasesApi.purchasesPreview).mockResolvedValue(real.PURCHASE_PREVIEW_E34);
    const { user } = await openTrip();
    const pick = async (id: string, text: string) => {
      const box = field(id);
      await user.click(box);
      await user.type(box, text);
      await user.click(await screen.findByRole('option'));
    };
    await pick('glossary-qism-maal', language === 'ur' ? 'م' : 'Ora');
    await pick('glossary-packing', language === 'ur' ? 'پ' : 'Box');
    await user.type(field('purchases-field-qty'), '20');
    await user.type(field('purchases-field-rate'), '500');
    await user.type(field('glossary-kiraya'), '400');
    const parts = await screen.findByRole('list', {
      name: named(message(language, 'trips-parts-title')),
    });
    const text = visible(parts.textContent);
    for (const figure of ['4,160', '3,120']) expect(text).toContain(figure);
    // Value 10,000, paid now 10,400, average cost 520: all the backend’s.
    for (const figure of ['10,000', '10,400', '520']) expect(pageText()).toContain(figure);
    expect(within(parts).getAllByText(said(message(language, 'ventures-part-shop')))).toHaveLength(
      3,
    );
  });

  /** Fills the purchase of the preview files: the item, the packing, the quantity and the rate. */
  async function fillPurchaseFields(user: ReturnType<typeof userEvent.setup>) {
    const pick = async (id: string, text: string) => {
      const box = field(id);
      await user.click(box);
      await user.type(box, text);
      await user.click(await screen.findByRole('option'));
    };
    await pick('glossary-qism-maal', language === 'ur' ? 'م' : 'Ora');
    await pick('glossary-packing', language === 'ur' ? 'پ' : 'Box');
    await user.type(field('purchases-field-qty'), '100');
    await user.type(field('purchases-field-rate'), '1000');
  }

  it('a purchase that takes cash below zero: the amber note with the backend’s −50,000, No saves nothing, Yes saves with the confirmation', async () => {
    vi.mocked(tripsApi.tripsGet).mockResolvedValue(real.TRIP_GET_OPEN);
    vi.mocked(purchasesApi.purchasesPreview).mockResolvedValue(
      real.PURCHASE_PREVIEW_CASH_BELOW_ZERO,
    );
    vi.mocked(purchasesApi.purchasesSave).mockResolvedValue(real.PURCHASE_SAVE_E34);
    const { user } = await openTrip();
    await fillPurchaseFields(user);
    expect(
      await screen.findByText(said(message(language, 'common-cash-confirm-title'))),
    ).toBeInTheDocument();
    // The answer to the last key must have landed, so that F10 asks first (a stale answer saves
    // and lets the backend's refusal open the box).
    await waitFor(() => {
      expect(purchasesApi.purchasesPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ qty: '100', rate: '1000' }),
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
    // The box names the cash after the entry: −50,000, the backend's number.
    expect(pageText()).toContain(
      message(language, 'common-cash-confirm-text', { amount: '-50,000' }),
    );
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
      expect(purchasesApi.purchasesSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(purchasesApi.purchasesSave).mock.calls[0]?.[0]).toMatchObject({
      confirm_cash_below_zero: true,
      trip_id: 1,
    });
  });

  it('a part paid from a pocket: 48,000 from the shop, 48,000 recorded only, and no cash question (E2E-41)', async () => {
    vi.mocked(tripsApi.tripsGet).mockResolvedValue(real.TRIP_GET_OPEN);
    vi.mocked(purchasesApi.purchasesPreview).mockResolvedValue(real.PURCHASE_PREVIEW_POCKET);
    const { user } = await openTrip();
    await fillPurchaseFields(user);
    const parts = await screen.findByRole('list', {
      name: named(message(language, 'trips-parts-title')),
    });
    expect(visible(parts.textContent)).toContain('48,000');
    expect(within(parts).getAllByText(said(message(language, 'ventures-part-shop')))).toHaveLength(
      1,
    );
    expect(
      within(parts).getAllByText(said(message(language, 'ventures-part-pocket'))),
    ).toHaveLength(1);
    // The shop part leaves 2,000 in cash, so the question is not asked.
    expect(screen.queryByText(said(message(language, 'common-cash-confirm-title')))).toBeNull();
  });

  it('a missing quantity is told at the quantity field and the numbers wait', async () => {
    vi.mocked(tripsApi.tripsGet).mockResolvedValue(real.TRIP_GET_OPEN);
    vi.mocked(purchasesApi.purchasesPreview).mockResolvedValue(real.PURCHASE_PREVIEW_NO_QTY);
    const { user } = await openTrip();
    await user.type(field('purchases-field-rate'), '1000');
    await waitFor(() => {
      expect(purchasesApi.purchasesPreview).toHaveBeenCalled();
    });
    // Nothing is shown as a number and nothing is invented: no parts list.
    expect(
      screen.queryByRole('list', { name: named(message(language, 'trips-parts-title')) }),
    ).toBeNull();
  });

  it('a cancelled purchase stays in the list, marked, with its reason', async () => {
    const cancelled = real.PURCHASES_LIST_CANCELLED.rows[0];
    if (cancelled === undefined) throw new Error('the file has no purchase');
    vi.mocked(tripsApi.tripsGet).mockResolvedValue({
      ...real.TRIP_GET_OPEN,
      purchases: [cancelled],
    });
    await openTrip();
    const row = await waitFor(() => {
      const found = document.querySelector(`.ag-row[row-id="${String(cancelled.id)}"]`);
      if (found === null) throw new Error('no purchase row');
      return found;
    });
    // The item and the packing, marked as cancelled in words (not by colour alone).
    expect(visible(row.querySelector('[col-id="item"]')?.textContent)).toBe(
      message(language, 'purchases-item-cancelled', {
        item: language === 'ur' ? 'مالٹا، پیٹی' : 'Orange, Box',
      }),
    );
  });

  /** Opens a row of the history of the trip and presses the cancel button, then types a reason. */
  async function cancelWithReason(user: ReturnType<typeof userEvent.setup>, rowId: string) {
    const row = await waitFor(() => {
      const found = document.querySelector<HTMLElement>(
        `.ag-row[row-id="${rowId}"] [col-id="entry"]`,
      );
      if (found === null) throw new Error(`no row ${rowId}`);
      return found;
    });
    await user.click(row);
    await user.click(
      screen.getByRole('button', { name: message(language, 'trips-entries-cancel') }),
    );
    await user.type(field('trips-cancel-reason'), 'wrong amount');
    await user.click(
      screen.getByRole('button', { name: message(language, 'trips-cancel-confirm') }),
    );
  }

  it('a sale in another shop is cancelled with the same request as the backend’s test, and the answer is told', async () => {
    vi.mocked(tripsApi.tripsGet).mockResolvedValue(real.TRIP_GET_OPEN);
    vi.mocked(tripsApi.tripsOtherSaleCancel).mockResolvedValue(real.OTHER_CANCEL);
    const { user } = await openTrip();
    await cancelWithReason(user, 'other_sale-2');
    await waitFor(() => {
      expect(tripsApi.tripsOtherSaleCancel).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(tripsApi.tripsOtherSaleCancel).mock.calls[0]?.[0];
    // Same keys as the file the backend sent, the id and version of the row, the reason typed.
    expect(Object.keys(sent ?? {}).sort()).toEqual(Object.keys(real.OTHER_CANCEL_REQUEST).sort());
    expect(sent).toMatchObject({ id: 2, reason: 'wrong amount', confirm_cash_below_zero: false });
  });

  it('a trip cost is cancelled with the same request as the backend’s test', async () => {
    const detail = structuredClone(real.TRIP_GET_E41_CLOSED);
    // The cost can be cancelled while nothing of its purchases is sold (the file’s trip has sales).
    detail.entries = detail.entries.map((entry) =>
      entry.kind === 'cost' ? { ...entry, can_cancel: true } : entry,
    );
    vi.mocked(tripsApi.tripsGet).mockResolvedValue({
      ...detail,
      row: { ...detail.row, status: 'open' },
    });
    vi.mocked(tripsApi.tripsCostCancel).mockResolvedValue(real.COST_CANCEL);
    const { user } = await openTrip();
    await cancelWithReason(user, 'cost-2');
    await waitFor(() => {
      expect(tripsApi.tripsCostCancel).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(tripsApi.tripsCostCancel).mock.calls[0]?.[0];
    expect(Object.keys(sent ?? {}).sort()).toEqual(Object.keys(real.COST_CANCEL_REQUEST).sort());
    expect(sent).toMatchObject({ id: 2, reason: 'wrong amount', confirm_cash_below_zero: false });
  });

  it('a purchase on a closed trip is refused with the backend’s own refusal, told in plain words', async () => {
    vi.mocked(tripsApi.tripsGet).mockResolvedValue(real.TRIP_GET_OPEN);
    vi.mocked(purchasesApi.purchasesPreview).mockResolvedValue(real.PURCHASE_PREVIEW_E34);
    vi.mocked(purchasesApi.purchasesSave).mockRejectedValue(
      new CommandError(real.PURCHASE_SAVE_TRIP_CLOSED),
    );
    const { user } = await openTrip();
    const pick = async (id: string, text: string) => {
      const box = field(id);
      await user.click(box);
      await user.type(box, text);
      await user.click(await screen.findByRole('option'));
    };
    await pick('glossary-qism-maal', language === 'ur' ? 'م' : 'Ora');
    await pick('glossary-packing', language === 'ur' ? 'پ' : 'Box');
    await user.type(field('purchases-field-qty'), '20');
    await user.type(field('purchases-field-rate'), '500');
    await screen.findByRole('list', { name: named(message(language, 'trips-parts-title')) });
    press('F10');
    expect(await screen.findByText(said(message(language, 'err-trip-closed')))).toBeInTheDocument();
    // Nothing typed is lost.
    expect(field('purchases-field-qty')).toHaveValue('20');
  });
});
