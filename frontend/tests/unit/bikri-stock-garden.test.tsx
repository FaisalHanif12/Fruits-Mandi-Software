// Garden goods in باقی مال (plan 33 version 1.1, PRD FR-12.40), in the Urdu and the English app,
// against a fake backend: the goods arrived entries of open gardens with goods left are listed
// apart, each marked with its number in its own series (GA-1), its garden code (0080) and its day
// to sell; the card has the figures and leads to the garden screen. Every figure on the screen is
// a figure the backend sent. The screen is not "empty" when only garden goods are left.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { GardenStockRow } from '../../src/bindings/GardenStockRow';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { fakePlan16 } from '../helpers/bikri-plan16';
import { stockList } from '../helpers/bikri-plan16-data';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { fakeTrips } from '../helpers/trips';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/purchases/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** An entry of the garden of E2E-40 (0080): 40 پیٹی کیلا arrived on 10-10-26, 15 sold, 25 left. */
const GARDEN_ROW: GardenStockRow = {
  arrival_id: 12,
  number: 1,
  received_date: '2026-10-10',
  sell_date: '2026-10-11',
  garden: {
    id: 1,
    number: 1,
    code: '0080',
    name_ur: 'شاہ پور باغ',
    name_en: 'Shahpur Garden',
    account: { id: 41, code: '0080', name_ur: 'شاہ پور باغ', name_en: 'Shahpur Garden' },
  },
  item: { id: 62, code: '0045', name_ur: 'کیلا', name_en: 'Banana' },
  packing: { id: 1, code: '0050', name_ur: 'پیٹی', name_en: 'Box' },
  qty_arrived: 40,
  qty_our_left: 25,
  qty_other_left: 0,
  shop_name: null,
  qty_sold: 15,
  qty_written_off: 0,
  balance: 25,
  days_waiting: 2,
  write_offs: [],
};

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

describe.each(LANGUAGES)('garden goods in باقی مال (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    fakePlan16({ rows: [] });
    fakeTrips();
    vi.mocked(bikriApi.bikriStockList).mockResolvedValue({
      ...stockList([]),
      garden_rows: [GARDEN_ROW],
      garden_balance_total: 25,
    });
  });

  const open = async () => {
    const view = await renderApp('/bikri/stock');
    await screen.findByText(said(message(language, 'bikri-stock-garden-title')));
    return { ...view, user: userEvent.setup() };
  };

  const cellOf = (field: string) => {
    const found = document.querySelector(`.ag-row[row-id="12"] [col-id="${field}"]`);
    if (found === null) throw new Error(`no cell ${field}`);
    return visible(found.textContent);
  };

  it('lists the entry apart, with its number, its garden code and what is left, and the screen is not empty', async () => {
    await open();
    await waitFor(() => {
      expect(cellOf('number')).toBe(message(language, 'gardens-arrival-number', { number: 1 }));
    });
    expect(cellOf('garden')).toBe('0080');
    expect(cellOf('balance')).toBe('25');
    // The list of lots is empty but there are garden goods, so no "nothing left" message.
    expect(screen.queryByText(said(message(language, 'bikri-stock-empty')))).toBeNull();
    // The total under the list is the backend's.
    expect(document.querySelectorAll('.ag-row-pinned').length).toBeGreaterThan(0);
  });

  it('the card has the figures and the day to sell, and its button leads to the garden', async () => {
    const { router, user } = await open();
    const cell = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.ag-row[row-id="12"] [col-id="number"]');
      if (found === null) throw new Error('no cell');
      return found;
    });
    await user.click(cell);
    const card = (
      await screen.findByText(
        said(
          message(language, 'bikri-stock-garden-card-title', {
            number: message(language, 'gardens-arrival-number', { number: 1 }),
            item: language === 'ur' ? 'کیلا' : 'Banana',
          }),
        ),
      )
    ).closest('section');
    if (card === null) throw new Error('no card');
    const text = visible(card.textContent);
    for (const figure of ['0080', '10-10-2026', '11-10-2026', '40', '15', '25']) {
      expect(text).toContain(figure);
    }
    await user.click(
      await screen.findByRole('button', {
        name: said(message(language, 'bikri-stock-garden-open')),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/gardens/1');
    });
  });
});
