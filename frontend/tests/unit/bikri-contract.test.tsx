// The screen and the backend agree (plan 12 contract, T4). The backend wrote, in its own test, the
// request the screen is meant to send for Event 1 and for E2E-26, and the answer to each. This
// types the same entry by keyboard and checks two things: the draft the screen sends is the draft
// in the backend's file (keys aside, commas aside), and the screen then shows what the backend's
// answer says. Urdu and English. The files are the backend's: they are copied, never edited.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { E2E26_REQUEST, EVENT1_REQUEST } from '../helpers/bikri-data';
import {
  cellOf,
  focusedIn,
  lotsGrid,
  named,
  openBook,
  salesGrid,
  textOf,
  typeEvent1Lot,
} from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

/** The draft as the backend's file has it: our row keys renamed lot-1, line-1 ..., commas out. */
function normalized(draft: BikriDraft): unknown {
  const copy = structuredClone(draft);
  copy.lots.forEach((lot, lotIndex) => {
    lot.key = `lot-${String(lotIndex + 1)}`;
    lot.sales.forEach((sale, saleIndex) => {
      sale.key = `line-${String(saleIndex + 1)}`;
    });
  });
  return withoutCommas(copy);
}

function withoutCommas(value: unknown): unknown {
  if (typeof value === 'string') return /^[\d,]+$/.test(value) ? value.replaceAll(',', '') : value;
  if (Array.isArray(value)) return value.map(withoutCommas);
  if (typeof value === 'object' && value !== null) {
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, withoutCommas(v)]));
  }
  return value;
}

/** The draft of the backend's file, in the same form. */
const wanted = (request: typeof EVENT1_REQUEST): unknown => withoutCommas(request.draft);

describe.each(LANGUAGES)('the draft the screen sends is the backend’s (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open() {
    const book = await openBook(language);
    // No truck memory: the backend's files have no اڈا and no filled fields.
    vi.mocked(bikriApi.assistTruckMemory).mockResolvedValue({
      last: null,
      same_day: null,
      suggestions: [],
    });
    return book;
  }

  it('Event 1: header, lot and the two sale lines are the same as bikri_preview_event1_request.json', async () => {
    const { backend, user } = await open();
    await typeEvent1Lot(user, language);
    await user.keyboard('1{Enter}20{Enter}600{Enter}');
    // The second line starts at 30 before the buyer is chosen; the munshi sees it, then goes on.
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'qty')).toBe('30');
    });
    await user.keyboard('2{Enter}{Enter}580{Enter}');
    await waitFor(() => {
      const sent = backend.previews.at(-1)?.lots[0]?.sales[1]?.seller_rate;
      expect(sent).toBe('580');
    });
    const sent = normalized(backend.lastDraft()) as BikriDraft;
    const file = wanted(EVENT1_REQUEST) as BikriDraft;
    expect(sent.header).toEqual(file.header);
    const { sales: sentSales, ...sentLot } = sent.lots[0] ?? { sales: [] };
    const { sales: fileSales, ...fileLot } = file.lots[0] ?? { sales: [] };
    expect(sentLot).toEqual(fileLot);
    expect(sentSales.slice(0, 2)).toEqual(fileSales.slice(0, 2));
    // The third row is blank (no buyer, no rate): the backend ignores what else is in it.
    expect(sentSales[2]).toMatchObject({ buyer_account_id: null, seller_rate: '' });
    expect(fileSales[2]).toMatchObject({ buyer_account_id: null, seller_rate: '' });
  });

  it('E2E-26: one finished line and a blank line that already starts at 30 are the same as bikri_preview_e2e26_request.json', async () => {
    const { backend, user } = await open();
    await typeEvent1Lot(user, language);
    await user.keyboard('1{Enter}20{Enter}600{Enter}');
    const sales = salesGrid(language);
    // The blank second line starts at what the lot has left, before any buyer is chosen.
    await waitFor(() => {
      expect(textOf(sales, 1, 'qty')).toBe('30');
    });
    await waitFor(() => {
      expect(backend.previews.at(-1)?.lots[0]?.sales[1]?.qty).toBe('30');
    });
    expect(normalized(backend.lastDraft())).toEqual(wanted(E2E26_REQUEST));
  });
});

describe.each(LANGUAGES)('E2E-26 on the screen, from the backend’s answer (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('line 2 starts at 30, the rate is empty with 600 as grey text, Ctrl+D copies 600, ⓘ shows 12,000 × 6 ÷ 100 = 720', async () => {
    const { user } = await openBook(language);
    vi.mocked(bikriApi.assistTruckMemory).mockResolvedValue({
      last: null,
      same_day: null,
      suggestions: [],
    });
    await typeEvent1Lot(user, language);
    await user.keyboard('1{Enter}20{Enter}600{Enter}');
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('1:buyer');
    });
    // The blank second line: 30 in the quantity, the rate empty, the last rate as grey text.
    await waitFor(() => {
      expect(textOf(sales, 1, 'qty')).toBe('30');
    });
    const rate = cellOf(sales, 1, 'rate');
    await waitFor(() => {
      expect(rate).toHaveClass('cell-hint');
    });
    expect(visible(rate.textContent).trim()).toBe('600');
    // 30 of the 50 are not sold yet: the backend says so, and the lot's quantity cell shows it.
    await waitFor(() => {
      expect(cellOf(lotsGrid(language), 0, 'qty')).toHaveClass('cell-warn');
    });
    expect(visible(document.body.textContent)).toContain(
      message(language, 'err-qty-left-unsold', { left: 30 }),
    );
    // Buyer, Enter takes the 30, then Ctrl+D copies the rate of the line above.
    await user.keyboard('2{Enter}{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('1:rate');
    });
    await user.keyboard('{Control>}d{/Control}');
    await waitFor(() => {
      expect(rate).not.toHaveClass('cell-hint');
    });
    expect(visible(rate.textContent).trim()).toBe('600');
    // ⓘ on the seller commission of line 1.
    await user.click(
      within(cellOf(sales, 0, 'sellerComm')).getByRole('button', {
        name: named(message(language, 'calc-info-label')),
      }),
    );
    const working = message(language, 'working-percent', {
      base: '12,000',
      bps: '6',
      result: '720',
    });
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(working);
    });
    expect(screen.getAllByRole('button').length).toBeGreaterThan(0);
  });
});
