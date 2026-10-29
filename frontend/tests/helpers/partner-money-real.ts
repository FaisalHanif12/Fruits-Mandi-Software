// The real answers and requests of the backend for plan 17 (T4, second half): partner capital, the
// partner overview with its new columns, the partner statement, الگ کھاتہ and جنرل. The backend wrote
// them in its own test (backend/crates/services/tests/partner_money_fixtures.rs); they are copied
// here, never edited, and read as the generated types. Money in them is paisa, as on the wire. The
// story is the PRD worked day of 10-10-2026 with partner ب's 50,000 of capital on 11-10-2026
// (E2E-35, E2E-37) and the family khata of E2E-38.

import capitalHistory from '../fixtures/capital_history.json';
import capitalHistoryRequest from '../fixtures/capital_history_request.json';
import generalPreviewBalanced from '../fixtures/general_preview_balanced.json';
import generalPreviewBalancedRequest from '../fixtures/general_preview_balanced_request.json';
import generalPreviewUnbalanced from '../fixtures/general_preview_unbalanced.json';
import generalPreviewUnbalancedRequest from '../fixtures/general_preview_unbalanced_request.json';
import generalSave from '../fixtures/general_save.json';
import generalSaveNotBalanced from '../fixtures/general_save_not_balanced.json';
import generalSaveNotBalancedRequest from '../fixtures/general_save_not_balanced_request.json';
import generalSaveRequest from '../fixtures/general_save_request.json';
import khataCreate from '../fixtures/khata_create.json';
import khataCreateRequest from '../fixtures/khata_create_request.json';
import khataStatement from '../fixtures/khata_statement.json';
import khataStatementRequest from '../fixtures/khata_statement_request.json';
import partnersOverview from '../fixtures/partners_overview.json';
import partnersStatement from '../fixtures/partners_statement.json';
import partnersStatementPage from '../fixtures/partners_statement_page.json';
import partnersStatementPageRequest from '../fixtures/partners_statement_page_request.json';
import partnersStatementRequest from '../fixtures/partners_statement_request.json';
import vouchersPreviewCapitalOut from '../fixtures/vouchers_preview_capital_out.json';
import vouchersPreviewCapitalOutRequest from '../fixtures/vouchers_preview_capital_out_request.json';
import vouchersSaveCapitalIn from '../fixtures/vouchers_save_capital_in.json';
import vouchersSaveCapitalInRequest from '../fixtures/vouchers_save_capital_in_request.json';
import vouchersSaveCapitalOutCashConfirm from '../fixtures/vouchers_save_capital_out_cash_confirm.json';
import vouchersSaveCapitalOutCashConfirmRequest from '../fixtures/vouchers_save_capital_out_cash_confirm_request.json';
import vouchersSaveKhataIn from '../fixtures/vouchers_save_khata_in.json';
import vouchersSaveKhataInRequest from '../fixtures/vouchers_save_khata_in_request.json';
import vouchersSaveKhataOut from '../fixtures/vouchers_save_khata_out.json';
import vouchersSaveKhataOutMistakes from '../fixtures/vouchers_save_khata_out_mistakes.json';
import vouchersSaveKhataOutMistakesRequest from '../fixtures/vouchers_save_khata_out_mistakes_request.json';
import vouchersSaveKhataOutRequest from '../fixtures/vouchers_save_khata_out_request.json';
import type { CapitalHistory } from '../../src/bindings/CapitalHistory';
import type { CapitalHistoryInput } from '../../src/bindings/CapitalHistoryInput';
import type { CapitalInput } from '../../src/bindings/CapitalInput';
import type { GeneralInput } from '../../src/bindings/GeneralInput';
import type { GeneralPreview } from '../../src/bindings/GeneralPreview';
import type { GeneralPreviewInput } from '../../src/bindings/GeneralPreviewInput';
import type { KhataAccount } from '../../src/bindings/KhataAccount';
import type { KhataCreateInput } from '../../src/bindings/KhataCreateInput';
import type { KhataInInput } from '../../src/bindings/KhataInInput';
import type { KhataOutInput } from '../../src/bindings/KhataOutInput';
import type { KhataStatement } from '../../src/bindings/KhataStatement';
import type { KhataStatementInput } from '../../src/bindings/KhataStatementInput';
import type { PartnerOverviewRow } from '../../src/bindings/PartnerOverviewRow';
import type { PartnerStatement } from '../../src/bindings/PartnerStatement';
import type { PartnersStatementInput } from '../../src/bindings/PartnersStatementInput';
import type { UiError } from '../../src/bindings/UiError';
import type { VoucherEntry } from '../../src/bindings/VoucherEntry';
import type { VoucherPreview } from '../../src/bindings/VoucherPreview';
import type { VoucherSaved } from '../../src/bindings/VoucherSaved';
import { paisaForTest } from './paisa';

// A fixture file is the backend's own JSON; each is read as its generated type.
function readCapitalHistory(value: unknown): CapitalHistory {
  return value as CapitalHistory;
}
function readVoucherPreview(value: unknown): VoucherPreview {
  return value as VoucherPreview;
}
function readVoucherSaved(value: unknown): VoucherSaved {
  return value as VoucherSaved;
}
function readUiError(value: unknown): UiError {
  return value as UiError;
}
function readPartnerOverviewRow(value: unknown): PartnerOverviewRow[] {
  return value as PartnerOverviewRow[];
}
function readPartnerStatement(value: unknown): PartnerStatement {
  return value as PartnerStatement;
}
function readKhataAccount(value: unknown): KhataAccount {
  return value as KhataAccount;
}
function readKhataStatement(value: unknown): KhataStatement {
  return value as KhataStatement;
}
function readGeneralPreview(value: unknown): GeneralPreview {
  return value as GeneralPreview;
}
function readCapitalHistoryInput(value: unknown): CapitalHistoryInput {
  return value as CapitalHistoryInput;
}
function readentryVoucherEntryreplacingvoucheridnull(value: unknown): {
  entry: VoucherEntry;
  replacing_voucher_id: null;
} {
  return value as { entry: VoucherEntry; replacing_voucher_id: null };
}
function readCapitalInput(value: unknown): CapitalInput {
  return value as CapitalInput;
}
function readPartnersStatementInput(value: unknown): PartnersStatementInput {
  return value as PartnersStatementInput;
}
function readKhataCreateInput(value: unknown): KhataCreateInput {
  return value as KhataCreateInput;
}
function readKhataStatementInput(value: unknown): KhataStatementInput {
  return value as KhataStatementInput;
}
function readKhataOutInput(value: unknown): KhataOutInput {
  return value as KhataOutInput;
}
function readKhataInInput(value: unknown): KhataInInput {
  return value as KhataInInput;
}
function readGeneralPreviewInput(value: unknown): GeneralPreviewInput {
  return value as GeneralPreviewInput;
}
function readGeneralInput(value: unknown): GeneralInput {
  return value as GeneralInput;
}

// Answers
export const CAPITAL_HISTORY: CapitalHistory = readCapitalHistory(capitalHistory);
export const PREVIEW_CAPITAL_OUT: VoucherPreview = readVoucherPreview(vouchersPreviewCapitalOut);
export const SAVED_CAPITAL_IN: VoucherSaved = readVoucherSaved(vouchersSaveCapitalIn);
export const REFUSED_CAPITAL_OUT_CASH: UiError = readUiError(vouchersSaveCapitalOutCashConfirm);
export const OVERVIEW: PartnerOverviewRow[] = readPartnerOverviewRow(partnersOverview);
export const STATEMENT_B: PartnerStatement = readPartnerStatement(partnersStatement);
export const STATEMENT_A_PAGE: PartnerStatement = readPartnerStatement(partnersStatementPage);
export const KHATA_CREATED: KhataAccount = readKhataAccount(khataCreate);
export const KHATA_STATEMENT: KhataStatement = readKhataStatement(khataStatement);
export const SAVED_KHATA_OUT: VoucherSaved = readVoucherSaved(vouchersSaveKhataOut);
export const SAVED_KHATA_IN: VoucherSaved = readVoucherSaved(vouchersSaveKhataIn);
export const REFUSED_KHATA_MISTAKES: UiError = readUiError(vouchersSaveKhataOutMistakes);
export const GENERAL_BALANCED: GeneralPreview = readGeneralPreview(generalPreviewBalanced);
export const GENERAL_UNBALANCED: GeneralPreview = readGeneralPreview(generalPreviewUnbalanced);
export const SAVED_GENERAL: VoucherSaved = readVoucherSaved(generalSave);
export const REFUSED_GENERAL_NOT_BALANCED: UiError = readUiError(generalSaveNotBalanced);

// Requests: exactly the `input` each command took
export const REQUEST_CAPITAL_HISTORY: CapitalHistoryInput =
  readCapitalHistoryInput(capitalHistoryRequest);
export const REQUEST_PREVIEW_CAPITAL_OUT: { entry: VoucherEntry; replacing_voucher_id: null } =
  readentryVoucherEntryreplacingvoucheridnull(vouchersPreviewCapitalOutRequest);
export const REQUEST_SAVE_CAPITAL_IN: CapitalInput = readCapitalInput(vouchersSaveCapitalInRequest);
export const REQUEST_SAVE_CAPITAL_OUT_CASH: CapitalInput = readCapitalInput(
  vouchersSaveCapitalOutCashConfirmRequest,
);
export const REQUEST_STATEMENT_B: PartnersStatementInput =
  readPartnersStatementInput(partnersStatementRequest);
export const REQUEST_STATEMENT_A_PAGE: PartnersStatementInput = readPartnersStatementInput(
  partnersStatementPageRequest,
);
export const REQUEST_KHATA_CREATE: KhataCreateInput = readKhataCreateInput(khataCreateRequest);
export const REQUEST_KHATA_STATEMENT: KhataStatementInput =
  readKhataStatementInput(khataStatementRequest);
export const REQUEST_SAVE_KHATA_OUT: KhataOutInput = readKhataOutInput(vouchersSaveKhataOutRequest);
export const REQUEST_SAVE_KHATA_OUT_MISTAKES: KhataOutInput = readKhataOutInput(
  vouchersSaveKhataOutMistakesRequest,
);
export const REQUEST_SAVE_KHATA_IN: KhataInInput = readKhataInInput(vouchersSaveKhataInRequest);
export const REQUEST_GENERAL_BALANCED: GeneralPreviewInput = readGeneralPreviewInput(
  generalPreviewBalancedRequest,
);
export const REQUEST_GENERAL_UNBALANCED: GeneralPreviewInput = readGeneralPreviewInput(
  generalPreviewUnbalancedRequest,
);
export const REQUEST_SAVE_GENERAL: GeneralInput = readGeneralInput(generalSaveRequest);
export const REQUEST_SAVE_GENERAL_NOT_BALANCED: GeneralInput = readGeneralInput(
  generalSaveNotBalancedRequest,
);

const rupees = (amount: number) => paisaForTest(amount * 100);

/**
 * The statement of partner الف in the E2E-34 run (D4-A33): -4,160 for his part of the purchase from
 * the shop's cash, +480 of commission, +3,384 of the sale of 15 box of orange, ending at -296.
 * The backend wrote no fixture for it. It is written here by hand from what `statement.rs` puts on
 * each kind of line (heading, detail, `values.kind` and the value keys), and the shape test checks
 * those keys. Ask the backend for a real fixture when one is written.
 */
export const STATEMENT_A_E2E34: PartnerStatement = {
  partner: { id: 1, code: '', name_ur: 'الف', name_en: 'Partner A' },
  opening_total_paisa: rupees(0),
  closing_total_paisa: rupees(-296),
  total_count: 3,
  totals: { plus_paisa: rupees(3864), minus_paisa: rupees(4160) },
  months: [],
  lines: [
    {
      entry_date: '2026-10-10',
      heading: 'trip',
      detail: 'trip_paid',
      entry_id: 5,
      number: 1,
      venture_code: '0070',
      secondary_number: 1,
      subject: null,
      packing: null,
      reason: '',
      amount_paisa: rupees(-4160),
      running_paisa: rupees(-4160),
      values: { kind: 'purchase' },
    },
    {
      entry_date: '2026-10-10',
      heading: 'bikri',
      detail: 'income_share',
      entry_id: 6,
      number: 1,
      venture_code: null,
      secondary_number: null,
      subject: null,
      packing: null,
      reason: '',
      amount_paisa: rupees(480),
      running_paisa: rupees(-3680),
      values: {},
    },
    {
      entry_date: '2026-10-10',
      heading: 'trip',
      detail: 'trip_credit',
      entry_id: 6,
      number: 1,
      venture_code: '0070',
      secondary_number: 1,
      subject: { id: 91, code: '', name_ur: 'مالٹا', name_en: 'Orange' },
      packing: null,
      reason: '',
      amount_paisa: rupees(3384),
      running_paisa: rupees(-296),
      values: {
        qty: 15,
        sales_paisa: 846000,
        cost_paisa: 780000,
        result_paisa: 66000,
        share_bps: 4000,
      },
    },
  ],
};
