// Typed wrappers around the commands of the cash book area (plan 17; modules `cashbook`,
// `vouchers`, `lent`, `khata`). Components never call invoke() (Code Standards F4). The only file
// of this module that knows the wire shape. Every command except the ones with no input takes one
// object named `input`. Money is typed as text going in and comes back as paisa (CLAUDE.md rule 1).
// There is no mode, bank or cheque in any of them: all money is روکڑ (PRD FR-10.4).

import type { ExpensePreviewInput } from '../../bindings/ExpensePreviewInput';
import type { KhataCreateRequest } from '../../bindings/KhataCreateRequest';
import type { KhataInInput } from '../../bindings/KhataInInput';
import type { KhataOutInput } from '../../bindings/KhataOutInput';
import type { KhataSetLinkInput } from '../../bindings/KhataSetLinkInput';
import type { KhataStatementInput } from '../../bindings/KhataStatementInput';
import type { LentDues } from '../../bindings/LentDues';
import type { LentDuesInput } from '../../bindings/LentDuesInput';
import type { LentListInput } from '../../bindings/LentListInput';
import type { LentPersonCreateRequest } from '../../bindings/LentPersonCreateRequest';
import type { LentPersonUpdateInput } from '../../bindings/LentPersonUpdateInput';
import type { MoneyLentRequest } from '../../bindings/MoneyLentRequest';
import type { MoneyLentReturnInput } from '../../bindings/MoneyLentReturnInput';
import type { PartnerTakesInput } from '../../bindings/PartnerTakesInput';
import type { PrintOutcome } from '../../bindings/PrintOutcome';
import type { TablePrintData } from '../../bindings/TablePrintData';
import type { PersonalExpenseInput } from '../../bindings/PersonalExpenseInput';
import type { PersonalReturnInput } from '../../bindings/PersonalReturnInput';
import type { RepeatLastMonthInput } from '../../bindings/RepeatLastMonthInput';
import type { ShopExpenseInput } from '../../bindings/ShopExpenseInput';
import { callCommand } from '../../shared/api';
import type {
  CapitalPrintInput,
  CapitalWhere,
  CapitalWhereInput,
  PartnersOpeningCapitalPlaced,
  PartnersPlaceOpeningCapitalInput,
  PrintCapitalInput,
} from './capital-types';
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
  KhataStatement,
  LentList,
  LentPerson,
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

/**
 * The capital card of the روکڑ page as of a day (`DD-MM-YYYY` as typed, or null for today): where the partners' money is, in two blocks, whose it
 * is, and the opening difference when it is not 0 (PRD FR-16.14, plan 87). Behind the partner code: while the
 * section is locked the backend answers `ERR_PARTNER_LOCKED`.
 */
export function cashbookCapitalWhere(input: CapitalWhereInput): Promise<CapitalWhere> {
  return callCommand<CapitalWhere>('cashbook_capital_where', { input });
}

/**
 * Puts the opening capital of the chosen partners into روکڑ (plan 87, PRD FR-16.14, FR-21.4): each opening is
 * reversed and posted again on its own date with روکڑ on the other side. It is a command of the partners area, kept
 * here because only this card calls it. The reason and the Owner's sign are checked by the backend, which reopens
 * and closes the days itself.
 */
export function partnersPlaceOpeningCapital(
  input: PartnersPlaceOpeningCapitalInput,
): Promise<PartnersOpeningCapitalPlaced> {
  return callCommand<PartnersOpeningCapitalPlaced>('partners_place_opening_capital', { input });
}

/** The card as a print table, every line, in the print language (PRD FR-19.13). */
export function cashbookCapitalPrintData(input: CapitalPrintInput): Promise<TablePrintData> {
  return callCommand<TablePrintData>('cashbook_capital_print_data', { input });
}

/** Prints the card, or saves it as a PDF: the same day, language and paper as the preview. */
export function cashbookCapitalPrint(input: PrintCapitalInput): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('cashbook_capital_print', { input });
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

/**
 * Lends money to a person (PRD 16.5): cash goes out. `link` is what his loans are linked to (plan 76 point
 * 10); null keeps his link as it is. While he owes, a different link is refused (`ERR_LINK_FIXED_WHILE_OWED`).
 */
export function saveMoneyLent(input: MoneyLentRequest): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_money_lent', { input });
}

/** A person returns money, in cash or from his salary (PRD FR-16.4). */
export function saveMoneyLentReturn(input: MoneyLentReturnInput): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_money_lent_return', { input });
}

/**
 * Makes the account of a new person who can borrow (the `F4` of the screen, PRD 16.5), with his type and the
 * link of his loans (plan 76 point 10, PRD FR-16.13).
 */
export function lentPersonCreate(input: LentPersonCreateRequest): Promise<LentPerson> {
  return callCommand<LentPerson>('lent_person_create', { input });
}

/** Changes the type and the link of a person; the link changes only while he owes nothing. */
export function lentPersonUpdate(input: LentPersonUpdateInput): Promise<LentPerson> {
  return callCommand<LentPerson>('lent_person_update', { input });
}

/**
 * The page behind the dashboard tile of garden owners and others (PRD FR-16.13, plan 76 point 10): one month of
 * money given and returned with the person, type and link of each, and the people with their open amounts.
 * The totals are the backend's.
 */
export function lentDues(input: LentDuesInput): Promise<LentDues> {
  return callCommand<LentDues>('lent_dues', { input });
}

/** One person's loans, oldest first, page by page, with his balance and the totals (FR-16.5). */
export function lentList(input: LentListInput): Promise<LentList> {
  return callCommand<LentList>('lent_list', { input });
}

/** Makes a new الگ کھاتہ with its link: none, the shop or one partner (the `F4` of the screen, PRD 16.6). */
export function khataCreate(input: KhataCreateRequest): Promise<KhataAccount> {
  return callCommand<KhataAccount>('khata_create', { input });
}

/** Changes the link of a khata. Only while its balance is 0 (`ERR_LINK_LOCKED`). */
export function khataSetLink(input: KhataSetLinkInput): Promise<KhataAccount> {
  return callCommand<KhataAccount>('khata_set_link', { input });
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
