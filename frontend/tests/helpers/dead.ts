// Test data for the dead amount (plan 40, PRD FR-10.10, E2E-44). The shapes are the generated types
// of the contract (T1); the numbers are the worked day (PRD 4.9, E2E-44): after Events 1 to 5 ارشد
// owes 18,684 and dies on 11-10-26; on 12-10-26 his son pays 4,000. They are not the backend's own
// answers yet (plan 40, T2 writes those); when it does, the real fixtures replace these.

import type { DeadAmountPage } from '../../src/bindings/DeadAmountPage';
import type { DeadInfo } from '../../src/bindings/DeadInfo';
import type { DeadPreview } from '../../src/bindings/DeadPreview';
import type { DeadRow } from '../../src/bindings/DeadRow';
import { must } from './must';
import { balance, money } from './accounts';
import {
  CARD_ARSHAD_DEAD,
  PREVIEW_ARSHAD,
  PREVIEW_NOTHING_OWED,
  R38_AFTER_RECOVERY,
} from './dead-real';

/** ارشد after the move of 11-10-26 (reason: died), as the backend's party card says it. */
export const ARSHAD_DEAD: DeadInfo = must(CARD_ARSHAD_DEAD.dead);

/** What moving ارشد would do on 11-10-26: he owes 18,684 and his last entry is of 10-10-26. */
export const ARSHAD_PREVIEW: DeadPreview = PREVIEW_ARSHAD;

/** A person who owes nothing: the move is refused with a code, never with text. */
export const NOTHING_OWED_PREVIEW: DeadPreview = PREVIEW_NOTHING_OWED;

/** A person who is also a seller: his other account is not moved with him (PRD E10, A68). */
export const LINKED_PREVIEW: DeadPreview = {
  ...ARSHAD_PREVIEW,
  linked: {
    account_id: 31,
    code: '030110001',
    name_ur: 'ارشد بیوپاری',
    name_en: 'Arshad Seller',
    tag: null,
    balance: balance(2000, 'cr'),
  },
};

/** The row of R38 after the recovery of 12-10-26: owed then 18,684, recovered 4,000, still 14,684. */
export const ARSHAD_ROW: DeadRow = must(R38_AFTER_RECOVERY.rows[0]);

/** A one-time person (پختہ کھاتہ) who left the business and was fully recovered. */
export const SHAHID_ROW: DeadRow = {
  mark_id: 2,
  account_id: 41,
  code: '03049999',
  name_ur: 'پختہ کھاتہ',
  name_en: 'Pukhta Khata',
  tag: null,
  party_type: 'pukhta',
  pukhta_person_key: 'shahid|03000000000',
  pukhta_name_ur: 'شاہد',
  pukhta_name_en: 'Shahid',
  reason: 'left',
  note: '',
  entry_date: '2026-10-11',
  owed_then_paisa: money(1000),
  recovered_paisa: money(1000),
  corrections_paisa: 0,
  still_owed_paisa: money(0),
  last_recovery_date: '2026-10-12',
  status: 'recovered',
};

/** R38 with its rows and the totals over the whole filter, written out (the screen adds up nothing). */
export function deadPage(
  rows: DeadRow[],
  totals: { owed: number; recovered: number; corrections?: number; still: number },
): DeadAmountPage {
  return {
    rows,
    row_count: rows.length,
    totals: {
      owed_then_paisa: money(totals.owed),
      recovered_paisa: money(totals.recovered),
      corrections_paisa: (totals.corrections ?? 0) * 100,
      still_owed_paisa: money(totals.still),
    },
  };
}

/** After the recovery of 12-10-26: the dead amount is 14,684, and ارشد is the only one. */
export const PAGE_ARSHAD = R38_AFTER_RECOVERY;

/** Both rows, as the filter "all" shows them. */
export const PAGE_BOTH = deadPage([ARSHAD_ROW, SHAHID_ROW], {
  owed: 19684,
  recovered: 5000,
  still: 14684,
});
