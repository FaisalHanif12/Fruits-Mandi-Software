// The answers of the dues pages behind the dashboard tiles (plan 79 items 2 and 3, PRD FR-9.6 to FR-9.8): the
// sellers of the plan's example (فیصل F, احسان, عدنان: the shop owes 202,780), the four parts of کل اگراہی
// after Events 1 to 5, the people who borrowed, the sellers with an open advance and the cash sales of
// 10-10-26. Built as the generated types the backend sends; money is paisa through `paisaForTest`.

import type { AdvancesReport } from '../../src/bindings/AdvancesReport';
import type { DeadAmountPage } from '../../src/bindings/DeadAmountPage';
import type { CashSalesDay } from '../../src/bindings/CashSalesDay';
import type { CashSalesMonth } from '../../src/bindings/CashSalesMonth';
import type { LentOpenPage } from '../../src/bindings/LentOpenPage';
import type { ReportParty } from '../../src/bindings/ReportParty';
import type { SellerDuesPage } from '../../src/bindings/SellerDuesPage';
import type { SellerDuesRow } from '../../src/bindings/SellerDuesRow';
import type { TotalDues } from '../../src/bindings/TotalDues';
import { paisaForTest } from './paisa';

const rupees = (value: number) => paisaForTest(value * 100);

export function party(
  id: number,
  code: string,
  ur: string,
  en: string,
  tag: string | null = null,
): ReportParty {
  return {
    account_id: id,
    code,
    short_code: code.slice(3),
    name_ur: ur,
    name_en: en,
    tag,
    person_key: null,
  };
}

const CITY = { ur: 'کوٹ رادھا کشن', en: 'Kot Radha Kishan' };

/** A seller the shop owes (a credit balance): payable and balance are the same figure. */
export function sellerOwed(
  id: number,
  code: string,
  ur: string,
  en: string,
  payable: number,
  last: string | null,
  tag: string | null = null,
): SellerDuesRow {
  return {
    party: party(id, code, ur, en, tag),
    city: CITY,
    balance: { amount_paisa: rupees(payable), side: 'cr' },
    payable_paisa: rupees(payable),
    advance_open_paisa: rupees(0),
    owes_paisa: rupees(0),
    last_change_date: last,
  };
}

export const FAISAL = sellerOwed(11, '0011', 'فیصل', 'Faisal', 72_740, '2026-11-16', 'F');
export const EHSAN = sellerOwed(12, '0012', 'احسان', 'Ehsan', 120_780, '2026-11-16');
export const ADNAN = sellerOwed(13, '0013', 'عدنان', 'Adnan', 9_260, '2026-11-16');

/** Plan 79 item 2a: سب shows the three sellers, the shop owes 202,780, the sellers owe nothing. */
export const SELLERS_ALL: SellerDuesPage = {
  rows: [FAISAL, EHSAN, ADNAN],
  total_count: 3,
  shop_owes_paisa: rupees(202_780),
  seller_owes_paisa: rupees(0),
  advances_open_paisa: rupees(0),
  total: { amount_paisa: rupees(202_780), side: 'cr' },
};

/** The sellers who owe the shop (nobody in the plan's example). */
export const SELLERS_NONE: SellerDuesPage = {
  rows: [],
  total_count: 0,
  shop_owes_paisa: rupees(202_780),
  seller_owes_paisa: rupees(0),
  advances_open_paisa: rupees(0),
  total: { amount_paisa: rupees(0), side: 'zero' },
};

/** After Events 1 to 5 (PRD FR-9.7): buyers 21,564, nobody else; the grand total is the tile's figure. */
export const TOTAL_DUES: TotalDues = {
  parts: [
    { section: 'buyers', count: 2, total_paisa: rupees(21_564) },
    { section: 'sellers', count: 0, total_paisa: rupees(0) },
    { section: 'dead', count: 0, total_paisa: rupees(0) },
    { section: 'others', count: 0, total_paisa: rupees(0) },
  ],
  grand_total_paisa: rupees(21_564),
};

/** A world where every section has someone, so each list and each total shows. */
export const TOTAL_DUES_FULL: TotalDues = {
  parts: [
    { section: 'buyers', count: 2, total_paisa: rupees(21_564) },
    { section: 'sellers', count: 1, total_paisa: rupees(1_500) },
    { section: 'dead', count: 1, total_paisa: rupees(18_684) },
    { section: 'others', count: 1, total_paisa: rupees(5_000) },
  ],
  grand_total_paisa: rupees(46_748),
};

export const LENT_OPEN: LentOpenPage = {
  rows: [
    {
      party: party(30, '0060', 'اسلم', 'Aslam'),
      person_type: 'staff',
      open_paisa: rupees(2_000),
      oldest_open_date: '2026-10-10',
      overdue_paisa: rupees(2_000),
    },
    {
      party: party(31, '0061', 'نواز', 'Nawaz'),
      person_type: 'garden_owner',
      open_paisa: rupees(15_000),
      oldest_open_date: '2026-11-01',
      overdue_paisa: rupees(0),
    },
  ],
  total_count: 2,
  total_paisa: rupees(17_000),
  overdue_paisa: rupees(2_000),
};

/** The opening position (before Event 1): رحیم was given 15,000, adjusted 0, open 15,000. */
export const ADVANCES_OPEN: AdvancesReport = {
  as_of: '2026-10-10',
  rows: [
    {
      account: { id: 10, code: '0010', name_ur: 'رحیم', name_en: 'Rahim' },
      given_paisa: rupees(15_000),
      returned_paisa: rupees(0),
      adjusted_paisa: rupees(0),
      open_paisa: rupees(15_000),
      // Nothing is payable yet, so the whole advance is not covered (plan 100 A).
      uncovered_paisa: rupees(15_000),
      advance_covered: null,
      last_date: '2026-09-01',
    },
  ],
  total_count: 1,
  offset: 0,
  limit: 25,
  given_paisa: rupees(15_000),
  returned_paisa: rupees(0),
  adjusted_paisa: rupees(0),
  open_paisa: rupees(15_000),
  uncovered_paisa: rupees(15_000),
  advance_covered: null,
};

/** October 2026 after Events 1 to 5: one day, 10-10-26, one buyer paid on the spot, 12,800. */
export const CASH_MONTH: CashSalesMonth = {
  month: '2026-10',
  rows: [{ date: '2026-10-10', buyer_count: 1, cash_paisa: rupees(12_800) }],
  buyer_days_total: 1,
  total_paisa: rupees(12_800),
};

/** 10-10-26: شاہد (چکتا), بکری 2, bill 12,800, paid 12,800. */
export const CASH_DAY: CashSalesDay = {
  date: '2026-10-10',
  rows: [
    {
      party: party(0, '0030', 'شاہد', 'Shahid'),
      is_chukta: true,
      bikri_id: 2,
      bikri_number: 2,
      bill_paisa: rupees(12_800),
      cash_paisa: rupees(12_800),
    },
  ],
  total_count: 1,
  buyer_count: 1,
  bill_total_paisa: rupees(12_800),
  cash_total_paisa: rupees(12_800),
};

/** The dead amount after ارشد... here خالد, a buyer moved on 11-10-26 who still owes 18,684 (PRD FR-10.10, E2E-44). */
export const DEAD: DeadAmountPage = {
  rows: [
    {
      mark_id: 5,
      account_id: 40,
      code: '0024',
      name_ur: 'خالد',
      name_en: 'Khalid',
      tag: null,
      party_type: 'buyer',
      reason: 'died',
      note: '',
      entry_date: '2026-10-11',
      owed_then_paisa: rupees(18_684),
      recovered_paisa: rupees(0),
      corrections_paisa: 0,
      still_owed_paisa: rupees(18_684),
      last_recovery_date: null,
      status: 'owing',
    },
  ],
  row_count: 1,
  totals: {
    owed_then_paisa: rupees(18_684),
    recovered_paisa: rupees(0),
    corrections_paisa: 0,
    still_owed_paisa: rupees(18_684),
  },
};
