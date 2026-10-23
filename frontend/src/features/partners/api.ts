// Typed wrappers around the partners commands (plan 11; module `partners`, Tech Stack 5.2).
// Components never call invoke() (Code Standards F4). The only file that knows the wire shape.
// Every command except the two lists takes one object named `input`.

import { callCommand } from '../../shared/api';
import type {
  CapitalHistory,
  CapitalHistoryInput,
  CapitalInput,
  PartnerCreateInput,
  PartnerDetail,
  PartnerOverviewRow,
  PartnerRow,
  PartnerStatement,
  PartnerUpdateInput,
  PartnersSetSharesInput,
  PartnersStatementInput,
  ShareInput,
  SharesCheck,
} from './types';
import type { VoucherSaved } from '../vouchers';

/** Every partner, active ones first. `share_bps` is the share on today's date. */
export function partnersList(): Promise<PartnerRow[]> {
  return callCommand<PartnerRow[]>('partners_list');
}

/** One partner with his share history, oldest first. */
export function partnersGet(id: number): Promise<PartnerDetail> {
  return callCommand<PartnerDetail>('partners_get', { input: { id } });
}

/** Adds a partner and his capital and drawings accounts. He has no share until shares are set. */
export function partnersCreate(input: PartnerCreateInput): Promise<PartnerRow> {
  return callCommand<PartnerRow>('partners_create', { input });
}

/** Changes a partner's names, or makes him active or inactive. */
export function partnersUpdate(input: PartnerUpdateInput): Promise<PartnerRow> {
  return callCommand<PartnerRow>('partners_update', { input });
}

/** Sets the shares of all active partners from a day (PRD FR-17.1, FR-17.2). */
export function partnersSetShares(input: PartnersSetSharesInput): Promise<PartnerRow[]> {
  return callCommand<PartnerRow[]>('partners_set_shares', { input });
}

/**
 * The running total of the typed shares, worked out by the backend with the same reading as the
 * save. The screen never adds percents itself (CLAUDE.md rule 1). Read only.
 */
export function partnersCheckShares(shares: readonly ShareInput[]): Promise<SharesCheck> {
  return callCommand<SharesCheck>('partners_check_shares', { input: { shares } });
}

/** One row per active partner for the overview page (PRD 17.3). */
export function partnersOverview(): Promise<PartnerOverviewRow[]> {
  return callCommand<PartnerOverviewRow[]>('partners_overview');
}

/** A partner puts money into the shop: cash comes in, his capital grows (PRD 17.4, E2E-35). */
export function saveCapitalIn(input: CapitalInput): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_capital_in', { input });
}

/** A partner takes capital back: cash goes out, his capital goes down (PRD 17.4, FR-17.8). */
export function saveCapitalOut(input: CapitalInput): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_capital_out', { input });
}

/** One partner's capital history, oldest first, page by page, with his balance (PRD 17.4). */
export function capitalHistory(input: CapitalHistoryInput): Promise<CapitalHistory> {
  return callCommand<CapitalHistory>('capital_history', { input });
}

/**
 * One partner's statement, page by page: a dated line for every amount that reaches him, with the
 * running کل رقم, and the totals over the whole range (PRD FR-17.9, FR-17.10, FR-19.11).
 */
export function partnersStatement(input: PartnersStatementInput): Promise<PartnerStatement> {
  return callCommand<PartnerStatement>('partners_statement', { input });
}
