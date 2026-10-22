// A lot of trip goods in the sale book (plan 16b, PRD 12.3, FR-12.26, C21, C25), in the Urdu and the
// English app, against a fake backend: the panel under the lots grid offers the purchases of open
// trips with goods left, a choice fills the item, the packing and the quantity in the visible
// cells, and the panel shows what the backend worked out for the lot: the commission, the seller
// credit with each trip partner's part, the cost and the result, each with its working. The seller
// field of a lot is the only account field that offers اپنا مال (scope `lot_seller`).

import { QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useStore } from 'zustand';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { LotPreview } from '../../src/bindings/LotPreview';
import type { OwnLotPreview } from '../../src/bindings/OwnLotPreview';
import { createQueryClient } from '../../src/app/providers';
import { OwnLotPanel } from '../../src/features/bikri/components/OwnLotPanel';
import { BikriDraftProvider, createBikriDraft } from '../../src/features/bikri/draft-store';
import { lotColumns } from '../../src/features/bikri/columns';
import { isOwnLot } from '../../src/features/bikri/draft';
import { LOT } from '../../src/features/bikri/types';
import { LANGUAGES } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { EVENT1 } from '../helpers/bikri-data';
import { message, renderIn, visible } from '../helpers/render';
import { e34OwnLotPart, e34Purchase, fakeTrips } from '../helpers/trips';
import { named } from '../helpers/bikri-screen';

vi.mock('../../src/features/purchases/api');
vi.mock('../../src/features/trips/api');

const OWN_ACCOUNT = { id: 9, code: '030119998', name_ur: 'اپنا مال', name_en: 'Own Goods (Trips)' };

const said = (text: string) => (content: string) => visible(content) === text;

const ownPart = (result: number): OwnLotPreview => e34OwnLotPart(result);

function lotOf(own: OwnLotPreview | null): LotPreview {
  const base = EVENT1.lots[0];
  if (base === undefined) throw new Error('the fixture has no lot');
  return { ...base, is_own: own !== null, own };
}

describe.each(LANGUAGES)('a lot of trip goods (%s)', (language) => {
  beforeEach(() => {
    vi.resetAllMocks();
    // The names in the panel follow the app language, which the screen keeps in this store.
    act(() => {
      useLanguageStore.setState({ language });
    });
    fakeTrips({ purchases: [e34Purchase()] });
  });

  function Panel({
    stores,
    lot,
  }: {
    stores: ReturnType<typeof createBikriDraft>;
    lot: LotPreview | undefined;
  }) {
    const row = useStore(stores.lots, (state) => state.rows[0]);
    if (row === undefined) return null;
    return <OwnLotPanel row={row} lotIndex={0} error={null} lot={lot} />;
  }

  function show(lot: LotPreview | undefined, typed: Record<string, string> = {}) {
    const stores = createBikriDraft('10-10-2026');
    const rowId = stores.lots.getState().rows[0]?.rowId ?? '';
    act(() => {
      stores.lots.getState().patchRow(rowId, {
        typed,
        picked: { [LOT.seller]: OWN_ACCOUNT },
      });
    });
    renderIn(
      language,
      <QueryClientProvider client={createQueryClient()}>
        <BikriDraftProvider value={stores}>
          <Panel stores={stores} lot={lot} />
        </BikriDraftProvider>
      </QueryClientProvider>,
    );
    return { stores, rowId };
  }

  const item = language === 'ur' ? 'مالٹا' : 'Orange';
  const box = language === 'ur' ? 'پیٹی' : 'Box';

  it('the seller of an own lot is the اپنا مال account', () => {
    const { stores } = show(undefined);
    const row = stores.lots.getState().rows[0];
    expect(row !== undefined && isOwnLot(row)).toBe(true);
  });

  it('offers the purchases of open trips with goods left and says what the lot gives once known', async () => {
    show(lotOf(null));
    expect(await screen.findByRole('combobox')).toBeInTheDocument();
    expect(
      screen.getByText(said(message(language, 'bikri-own-numbers-pending'))),
    ).toBeInTheDocument();
  });

  it('choosing a purchase fills the item, the packing and the quantity like typed values', async () => {
    const { stores, rowId } = show(undefined);
    const user = userEvent.setup();
    await user.click(await screen.findByRole('combobox'));
    const option = await screen.findByRole('option');
    // The option names the purchase, its trip, the goods and what is left.
    expect(visible(option.textContent)).toBe(
      message(language, 'bikri-own-option', {
        number: message(language, 'purchases-number', { number: 1 }),
        trip: message(language, 'trips-number', { number: 1 }),
        item,
        packing: box,
        qty_left: 5,
      }),
    );
    await user.click(option);
    await waitFor(() => {
      expect(
        stores.lots.getState().rows.find((row) => row.rowId === rowId)?.typed[LOT.purchase],
      ).toBe('7');
    });
    const row = stores.lots.getState().rows.find((candidate) => candidate.rowId === rowId);
    expect(row?.typed[LOT.qty]).toBe('5');
    expect(row?.picked?.[LOT.item]?.id).toBe(61);
    expect(row?.picked?.[LOT.packing]?.id).toBe(71);
    // The facts of the purchase are the backend's: bought 20, sold 15, left 5, cost left 2,600.
    const facts = visible(document.body.textContent);
    for (const figure of ['20', '15', '2,600']) expect(facts).toContain(figure);
  });

  it('shows the commission, the seller credit with each trip partner’s part, the cost and the result', async () => {
    show(lotOf(ownPart(660)), { [LOT.purchase]: '7' });
    const credit = await screen.findByText(said(message(language, 'bikri-own-credit')));
    const panel = credit.closest('section');
    if (panel === null) throw new Error('no panel');
    const text = visible(panel.textContent);
    // E2E-34: 8,460 goes to the trip’s partners as 3,384 / 2,538 / 2,538; cost 7,800; result 660.
    for (const figure of ['8,460', '3,384', '2,538', '7,800', '660'])
      expect(text).toContain(figure);
    // The commission is the backend’s number for the lot, with its working.
    const commission = screen.getByText(said(message(language, 'bikri-own-commission')));
    expect(commission.closest('div')?.textContent).toContain('1,764');
    // Each calculated amount has its ⓘ: the commission, each partner’s part, the cost, the result.
    expect(within(panel).getAllByRole('button').length).toBeGreaterThanOrEqual(6);
    expect(
      screen.getByText(said(message(language, 'glossary-tijarati-munafa'))),
    ).toBeInTheDocument();
  });

  it('a loss is red and also told in words', async () => {
    show(lotOf(ownPart(-1_940)), { [LOT.purchase]: '7' });
    expect(
      await screen.findByText(said(message(language, 'glossary-tijarati-nuqsan'))),
    ).toBeInTheDocument();
    expect(screen.getByText(said(message(language, 'purchases-loss-note')))).toBeInTheDocument();
  });

  it('a saved lot keeps its purchase: the choice is read only', async () => {
    show(lotOf(ownPart(660)), { [LOT.purchase]: '7', [LOT.lotId]: '51' });
    const choice = await screen.findByRole('combobox', {
      name: named(message(language, 'bikri-own-purchase')),
    });
    expect(choice).toBeDisabled();
  });
});

describe('who may be the seller of a lot (PRD FR-12.26)', () => {
  it('the seller column of the lots grid asks the backend for scope lot_seller, which adds اپنا مال', () => {
    const seller = lotColumns('normal').find((column) => column.field === LOT.seller);
    expect(seller?.picker).toEqual({ kind: 'account', scope: 'lot_seller' });
  });

  it('every other picker column of the lots grid is a master list', () => {
    const others = lotColumns('normal').filter(
      (column) => column.type === 'picker' && column.field !== LOT.seller,
    );
    expect(others.length).toBeGreaterThan(0);
    for (const column of others) expect(column.picker?.kind).toBe('master');
  });
});
