// Typed wrappers around the partners commands (plan 11; module `partners`, Tech Stack 5.2).
// Components never call invoke() (Code Standards F4). The only file that knows the wire shape.
// Every command except the two lists takes one object named `input`.

import { callCommand } from '../../shared/api';
import type {
  PartnerCreateInput,
  PartnerDetail,
  PartnerOverviewRow,
  PartnerRow,
  PartnerUpdateInput,
  PartnersSetSharesInput,
  ShareInput,
  SharesCheck,
} from './types';

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
