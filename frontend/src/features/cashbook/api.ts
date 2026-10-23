// Typed wrappers around the commands of the cash book area (plan 17; modules `cashbook`,
// `vouchers`, `lent`, `khata`). Components never call invoke() (Code Standards F4). The only file
// of this module that knows the wire shape. Every command except the ones with no input takes one
// object named `input`. Money is typed as text going in and comes back as paisa (CLAUDE.md rule 1).
// There is no mode, bank or cheque in any of them: all money is روکڑ (PRD FR-10.4).

import type { ExpensePreviewInput } from '../../bindings/ExpensePreviewInput';
import type { KhataInInput } from '../../bindings/KhataInInput';
import type { KhataOutInput } from '../../bindings/KhataOutInput';
import type { KhataStatementInput } from '../../bindings/KhataStatementInput';
import type { LentListInput } from '../../bindings/LentListInput';
import type { MoneyLentInput } from '../../bindings/MoneyLentInput';
import type { MoneyLentReturnInput } from '../../bindings/MoneyLentReturnInput';
import type { PartnerTakesInput } from '../../bindings/PartnerTakesInput';
import type { PersonalExpenseInput } from '../../bindings/PersonalExpenseInput';
import type { PersonalReturnInput } from '../../bindings/PersonalReturnInput';
import type { RepeatLastMonthInput } from '../../bindings/RepeatLastMonthInput';
import type { ShopExpenseInput } from '../../bindings/ShopExpenseInput';
import { callCommand } from '../../shared/api';
import type {
  Cashbook,
  CashbookInput,
  ExpenseEntry,
  ExpenseHeads,
  ExpensePreview,
  GeneralInput,
  GeneralPreview,
  GeneralPreviewInput,
  KhataAccount,
  KhataCreateInput,
  KhataStatement,
  LentList,
  LentPerson,
  LentPersonCreateInput,
  PartnerTakes,
  RepeatLastMonth,
} from './types';
import type { VoucherSaved } from '../vouchers';

/**
 * The cash book of a range, one page: opening cash, every cash movement of every module, closing
 * cash, and the totals over the whole range (PRD 16.1, FR-19.11). There is no cash count (FR-16.1).
 */
export function cashbookGet(input: CashbookInput): Promise<Cashbook> {
  return callCommand<Cashbook>('cashbook_get', { input });
}

/** The shop expense heads, the most used first (PRD FR-7.62). */
export function expenseHeads(): Promise<ExpenseHeads> {
  return callCommand<ExpenseHeads>('masters_expense_heads');
}

/**
 * What saving an expense entry would do, from the same function as the save (PRD FR-16.8): each
 * partner's share of a shop expense, or the partner's total now and after for his own spending.
 */
export function expensePreview(
  entry: ExpenseEntry,
  replacing: number | null,
): Promise<ExpensePreview> {
  const input: ExpensePreviewInput = { entry, replacing_voucher_id: replacing };
  return callCommand<ExpensePreview>('vouchers_preview_expense', { input });
}

/** Saves a دکان خرچہ: one posting event, split by the shares of the day (PRD 16.2, E2E-20). */
export function saveShopExpense(input: ShopExpenseInput): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_shop_expense', { input });
}

/** Saves a ذاتی خرچہ: charged to that partner only, never split (PRD 16.3). */
export function savePersonalExpense(input: PersonalExpenseInput): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_personal_expense', { input });
}

/** Saves a ذاتی خرچہ واپسی: a partner gives money back (PRD FR-16.2). */
export function savePersonalReturn(input: PersonalReturnInput): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_personal_return', { input });
}

/** One partner's money taken, oldest first, page by page, with the totals (PRD FR-16.2). */
export function partnerTakes(input: PartnerTakesInput): Promise<PartnerTakes> {
  return callCommand<PartnerTakes>('partner_takes', { input });
}

/** This month's regular expenses proposed from last month, to check and save (PRD FR-7.62). */
export function repeatLastMonth(input: RepeatLastMonthInput): Promise<RepeatLastMonth> {
  return callCommand<RepeatLastMonth>('vouchers_repeat_last_month_preview', { input });
}

/** Lends money to a person (PRD 16.5): cash goes out. */
export function saveMoneyLent(input: MoneyLentInput): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_money_lent', { input });
}

/** A person returns money, in cash or from his salary (PRD FR-16.4). */
export function saveMoneyLentReturn(input: MoneyLentReturnInput): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_money_lent_return', { input });
}

/** Makes the account of a new person who can borrow (the `F4` of the screen, PRD 16.5). */
export function lentPersonCreate(input: LentPersonCreateInput): Promise<LentPerson> {
  return callCommand<LentPerson>('lent_person_create', { input });
}

/** One person's loans, oldest first, page by page, with his balance and the totals (FR-16.5). */
export function lentList(input: LentListInput): Promise<LentList> {
  return callCommand<LentList>('lent_list', { input });
}

/** Makes a new الگ کھاتہ (the `F4` of the screen, PRD 16.6). */
export function khataCreate(input: KhataCreateInput): Promise<KhataAccount> {
  return callCommand<KhataAccount>('khata_create', { input });
}

/** Money paid out of a khata's side: the khata goes minus, cash goes out (PRD FR-16.6). */
export function saveKhataOut(input: KhataOutInput): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_khata_out', { input });
}

/** Money received for a khata: it goes plus, cash comes in (PRD FR-16.6). */
export function saveKhataIn(input: KhataInInput): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_khata_in', { input });
}

/** A khata's entries in date order with the running signed balance, page by page (FR-16.6). */
export function khataStatement(input: KhataStatementInput): Promise<KhataStatement> {
  return callCommand<KhataStatement>('khata_statement', { input });
}

/**
 * Checks a جنرل entry without saving it (PRD 16.4): the totals, what is wrong with which line,
 * the partners' parts of income and expense lines and the cash after it. The same function as the
 * save (Code Standards B5). The screen adds nothing up.
 */
export function generalPreview(input: GeneralPreviewInput): Promise<GeneralPreview> {
  return callCommand<GeneralPreview>('general_preview', { input });
}

/** Saves a جنرل entry: only when the two sides are equal, nothing is written otherwise (PRD 16.4). */
export function generalSave(input: GeneralInput): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('general_save', { input });
}
