// The canned data of the day close tests has the shape of the backend's real answer (plan 17, F5,
// PRD FR-21.1, 4.9). The backend wrote the real answer in its own test (tests/fixtures/*.json,
// copied, never edited). This compares the keys and kinds at every level, so a test that passes on
// canned data cannot hide a field the real backend does not send, or one it sends and the canned
// data forgot. Then it checks that the screen sends the request the backend's own test sent.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import type { DayClosePreviewInput } from '../../src/bindings/DayClosePreviewInput';
import type { UiError } from '../../src/bindings/UiError';
import * as dayApi from '../../src/features/day/api';
import * as invoke from '../../src/shared/api/invoke';
import * as real from '../helpers/day-close-real';
import { same, sendsTheSame } from '../helpers/shapes';
import { fakeDay } from '../helpers/vouchers';

vi.mock('../../src/app/api');
vi.mock('../../src/features/day/api');
vi.mock('../../src/shared/api/invoke');

describe('the canned day close answer has the real shape', () => {
  it('the preview and its summary have the keys and kinds the backend sends', async () => {
    fakeDay();
    const canned = await dayApi.dayClosePreview('');
    expect(same(real.DAY_CLOSE_PREVIEW, canned)).toBeNull();
    // The summary and a partner line, checked on their own so a drift shows in the message.
    expect(same(real.DAY_CLOSE_PREVIEW.summary, canned.summary)).toBeNull();
    expect(same(real.DAY_CLOSE_PREVIEW.summary.partners[0], canned.summary.partners[0])).toBeNull();
    expect(appApi.dayStatus).toBeDefined();
  });

  it('the real answer is the story of the fixture: one بکری, the numbers of PRD 4.9 for it', () => {
    const { summary } = real.DAY_CLOSE_PREVIEW;
    expect(summary.bikri_count).toBe(1);
    expect(summary.gross_paisa).toBe(2_940_000);
    expect(summary.commission_paisa).toBe(392_800);
    expect(summary.shared_income_paisa).toBe(394_800);
    expect(summary.partners.map((row) => row.income_paisa)).toEqual([158_000, 118_400, 118_400]);
    expect(summary.partners.map((row) => row.expense_paisa)).toEqual([120_000, 90_000, 90_000]);
    expect(summary.partners.map((row) => row.profit_paisa)).toEqual([38_000, 28_400, 28_400]);
    // The partner reference has an empty code, as the contract says.
    expect(summary.partners.every((row) => row.partner.code === '')).toBe(true);
    expect(real.DAY_CLOSE_PREVIEW.trial_difference).toEqual({ amount_paisa: 0, side: 'zero' });
  });
});

describe('the other real answers of the day close preview', () => {
  it('the E2E-16 answer with stock left has the same shape and the story of the fixture', async () => {
    fakeDay();
    const canned = await dayApi.dayClosePreview('');
    expect(same(real.DAY_CLOSE_PREVIEW_STOCK_LEFT, canned)).toBeNull();
    const { summary } = real.DAY_CLOSE_PREVIEW_STOCK_LEFT;
    expect([summary.open_lots, summary.open_qty]).toEqual([1, 5]);
    expect(summary.gross_paisa).toBe(2_650_000);
    expect(summary.commission_paisa).toBe(354_000);
    expect(summary.shared_income_paisa).toBe(356_000);
    expect(real.REQUEST_DAY_CLOSE_PREVIEW_STOCK_LEFT).toEqual({ up_to: '10-10-2026' });
  });

  it('the whole worked day of PRD 4.9 has the same shape and the numbers of the PRD', async () => {
    fakeDay();
    const canned = await dayApi.dayClosePreview('');
    expect(same(real.DAY_CLOSE_PREVIEW_WORKED_DAY, canned)).toBeNull();
    const day = real.DAY_CLOSE_PREVIEW_WORKED_DAY;
    // 2 بکری, خام 41,400, commission 5,448, shared income 5,488 (PRD 4.9), in paisa.
    expect(day.summary.bikri_count).toBe(2);
    expect(day.summary.gross_paisa).toBe(4_140_000);
    expect(day.summary.commission_paisa).toBe(544_800);
    expect(day.summary.shared_income_paisa).toBe(548_800);
    expect(day.summary.partners.map((row) => row.profit_paisa)).toEqual([99_600, 74_600, 74_600]);
    // Cash 58,484 and the trial balance 85,048 on both sides, difference zero (E2E-8, E2E-10).
    expect(day.cash_in_hand).toEqual({ amount_paisa: 5_848_400, side: 'dr' });
    expect([day.trial_dr_paisa, day.trial_cr_paisa]).toEqual([8_504_800, 8_504_800]);
    expect(day.trial_difference.side).toBe('zero');
    expect(real.REQUEST_DAY_CLOSE_PREVIEW_WORKED_DAY).toEqual({ up_to: '' });
  });

  it('the refusal of a closed day has the shape of a refusal, on the date field', () => {
    const canned: UiError = {
      ok: false,
      errors: [{ code: 'ERR_DATE_CLOSED', field: 'up_to', values: { closed_up_to: '10-10-2026' } }],
      ref: 'E-20261010-0007',
    };
    expect(same(real.REFUSED_DAY_CLOSED, canned)).toBeNull();
    expect(real.REFUSED_DAY_CLOSED.errors[0]).toMatchObject({
      code: 'ERR_DATE_CLOSED',
      field: 'up_to',
      values: { closed_up_to: '10-10-2026' },
    });
    expect(real.REQUEST_DAY_CLOSE_PREVIEW_CLOSED_DAY).toEqual({ up_to: '10-10-2026' });
  });
});

describe('the screen sends the request the backend’s own test sent', () => {
  const call = vi.mocked(invoke.callCommand);

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('the wrapper sends the command name and the one `{ input }` object', async () => {
    // The day api is faked for the other tests; the real wrapper is read here, over the faked invoke.
    const actual = await vi.importActual<typeof import('../../src/features/day/api')>(
      '../../src/features/day/api',
    );
    call.mockResolvedValue(real.DAY_CLOSE_PREVIEW);
    await actual.dayClosePreview(real.REQUEST_DAY_CLOSE_PREVIEW.up_to);
    const [name, args] = call.mock.calls[0] ?? [];
    expect(name).toBe('day_close_preview');
    expect(args).toEqual({ input: real.REQUEST_DAY_CLOSE_PREVIEW });
    const sent = args as { input: DayClosePreviewInput };
    expect(sendsTheSame(sent.input, real.REQUEST_DAY_CLOSE_PREVIEW)).toBeNull();
  });
});
