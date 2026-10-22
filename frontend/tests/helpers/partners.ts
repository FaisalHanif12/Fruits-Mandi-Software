// Test helper: an in-memory fake of the partners commands (plan 11 contract as built in T1). The
// wrappers are mocked in the test file with vi.mock; this sets their answers the way a healthy
// backend answers. The sums here are the fake backend's (a test double): the screen only shows
// what comes back and never adds a percent or an amount (CLAUDE.md rule 1).

import { vi } from 'vitest';
import * as partnersApi from '../../src/features/partners/api';
import type {
  PartnerOverviewRow,
  PartnerRow,
  ShareHistoryRow,
  ShareInput,
  SharesCheck,
} from '../../src/features/partners/types';
import { money } from './accounts';
import { fail } from './app';

const FULL_BPS = 10_000;
const PERCENT = /^(\d{1,3})(?:\.(\d{1,2}))?$/;

/** Percent text as basis points, or null when the text cannot be read (the real parse_bps). */
export function bpsOf(text: string): number | null {
  const match = PERCENT.exec(text.trim());
  if (match === null) return null;
  const whole = Number(match[1]);
  const fraction = Number((match[2] ?? '').padEnd(2, '0'));
  const bps = whole * 100 + fraction;
  return bps > FULL_BPS ? null : bps;
}

export function partnerRow(
  id: number,
  name_ur: string,
  name_en: string,
  share_bps: number,
  extra: Partial<PartnerRow> = {},
): PartnerRow {
  const code = String(id).padStart(4, '0');
  return {
    id,
    name_ur,
    name_en,
    capital_account_id: 100 + id,
    capital_account_code: `0602${code}`,
    drawings_account_id: 200 + id,
    drawings_account_code: `0603${code}`,
    venture_account_id: 300 + id,
    venture_account_code: `0604${code}`,
    share_bps,
    is_active: true,
    version: 1,
    ...extra,
  };
}

/** الف 40, ب 30, ج 30: the placeholder partners of PRD 4.2. */
export const THREE_PARTNERS: readonly PartnerRow[] = [
  partnerRow(1, 'الف', 'Partner A', 4000),
  partnerRow(2, 'ب', 'Partner B', 3000),
  partnerRow(3, 'ج', 'Partner C', 3000),
];

export function overviewRow(
  partner: PartnerRow,
  numbers: Partial<PartnerOverviewRow> = {},
): PartnerOverviewRow {
  return {
    partner_id: partner.id,
    name_ur: partner.name_ur,
    name_en: partner.name_en,
    share_bps: partner.share_bps,
    today_income_paisa: money(0),
    month_income_paisa: money(0),
    year_income_paisa: money(0),
    month_expense_paisa: money(0),
    month_profit_paisa: money(0),
    month_taken_paisa: money(0),
    capital_balance_paisa: money(0),
    ...numbers,
  };
}

export interface FakePartners {
  rows: PartnerRow[];
  /** The history of each partner, oldest first, by partner id. */
  history: Record<number, ShareHistoryRow[]>;
  overview: PartnerOverviewRow[];
  /** Every share set that was saved, as sent. */
  setShares: { effective_from: string; shares: ShareInput[] }[];
}

/** The check the real backend makes: same reading as the save (Code Standards B5). */
function checkOf(shares: readonly ShareInput[], activeIds: readonly number[]): SharesCheck {
  const errors: SharesCheck['errors'] = [];
  let total = 0;
  shares.forEach((row, index) => {
    const bps = bpsOf(row.share);
    if (bps === null) {
      errors.push({
        code: row.share.trim() === '' ? 'ERR_REQUIRED' : 'ERR_NOT_A_NUMBER',
        field: `shares[${String(index)}].share`,
        values: {},
      });
    } else {
      total += bps;
    }
  });
  const named = shares.map((row) => row.partner_id).sort();
  const wanted = [...activeIds].sort();
  if (JSON.stringify(named) !== JSON.stringify(wanted)) {
    errors.push({ code: 'ERR_PARTNER_NOT_LISTED', field: 'shares', values: {} });
  }
  if (total !== FULL_BPS) {
    errors.push({ code: 'ERR_SHARES_NOT_100', field: 'shares', values: { total_bps: total } });
  }
  return { total_bps: total, is_100: errors.length === 0, errors };
}

const dayNumber = (text: string): string => text.split('-').reverse().join('');

/**
 * Sets the answers of the partners commands. `today` is `DD-MM-YYYY` (the injected clock of the
 * day status fake, 10-10-2026).
 */
export function fakePartners(
  initial: readonly PartnerRow[] = [],
  today = '10-10-2026',
): FakePartners {
  const store: FakePartners = {
    rows: initial.map((row) => ({ ...row })),
    history: {},
    overview: [],
    setShares: [],
  };
  const activeIds = () => store.rows.filter((row) => row.is_active).map((row) => row.id);

  vi.mocked(partnersApi.partnersList).mockImplementation(() =>
    Promise.resolve(store.rows.map((row) => ({ ...row }))),
  );
  vi.mocked(partnersApi.partnersOverview).mockImplementation(() => Promise.resolve(store.overview));
  vi.mocked(partnersApi.partnersGet).mockImplementation((id) => {
    const row = store.rows.find((one) => one.id === id);
    if (row === undefined) return Promise.reject(fail({ code: 'ERR_NOT_FOUND', field: 'id' }));
    return Promise.resolve({ row: { ...row }, shares: store.history[id] ?? [] });
  });
  vi.mocked(partnersApi.partnersCreate).mockImplementation((input) => {
    const id = Math.max(0, ...store.rows.map((row) => row.id)) + 1;
    const row = partnerRow(id, input.name_ur, input.name_en, 0);
    store.rows.push(row);
    return Promise.resolve({ ...row });
  });
  vi.mocked(partnersApi.partnersUpdate).mockImplementation((input) => {
    const row = store.rows.find((one) => one.id === input.id);
    if (row === undefined) return Promise.reject(fail({ code: 'ERR_NOT_FOUND', field: 'id' }));
    if (!input.is_active && row.share_bps > 0) {
      return Promise.reject(fail({ code: 'ERR_PARTNER_HAS_SHARE', field: 'is_active' }));
    }
    row.name_ur = input.name_ur;
    row.name_en = input.name_en;
    row.is_active = input.is_active;
    row.version += 1;
    return Promise.resolve({ ...row });
  });
  vi.mocked(partnersApi.partnersCheckShares).mockImplementation((shares) =>
    Promise.resolve(checkOf(shares, activeIds())),
  );
  vi.mocked(partnersApi.partnersSetShares).mockImplementation((input) => {
    if (dayNumber(input.effective_from) < dayNumber(today)) {
      return Promise.reject(
        fail({ code: 'ERR_SHARE_BACKDATED', field: 'effective_from', values: { today } }),
      );
    }
    const check = checkOf(input.shares, activeIds());
    if (!check.is_100) {
      return Promise.reject(
        fail(
          ...check.errors.map((item) => ({
            code: item.code,
            field: item.field,
            values: item.values,
          })),
        ),
      );
    }
    store.setShares.push({ effective_from: input.effective_from, shares: [...input.shares] });
    for (const row of input.shares) {
      const partner = store.rows.find((one) => one.id === row.partner_id);
      if (partner === undefined) continue;
      partner.share_bps = bpsOf(row.share) ?? 0;
      partner.version += 1;
    }
    return Promise.resolve(store.rows.map((row) => ({ ...row })));
  });
  return store;
}
