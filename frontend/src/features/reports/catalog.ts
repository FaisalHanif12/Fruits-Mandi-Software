// The catalogue of reports grouped as PRD 19.2 (Sales, Accounts, Collections, Business). The order
// of the groups and of the reports inside a group is the PRD's; the backend sends the list and
// each report's filters, the screen only groups it (no report is named here that the backend does
// not send).

import type { ReportCatalogEntry } from '../../bindings/ReportCatalogEntry';
import type { ReportGroup } from '../../bindings/ReportGroup';
import type { ReportId } from '../../bindings/ReportId';

/** The groups in the order of PRD 19.2. */
export const REPORT_GROUPS: readonly ReportGroup[] = [
  'sales',
  'accounts',
  'collections',
  'business',
];

export interface CatalogGroup {
  group: ReportGroup;
  /** Fluent message ID of the group's name. */
  titleId: string;
  entries: ReportCatalogEntry[];
}

/** The numeric part of an ID: R9 comes before R11 (PRD order inside a group). */
function numberOf(entry: ReportCatalogEntry): number {
  return Number(entry.id.replace(/^R/, ''));
}

/** The groups that have at least one report, in PRD order, each with its reports in PRD order. */
export function groupCatalog(entries: readonly ReportCatalogEntry[]): CatalogGroup[] {
  const groups: CatalogGroup[] = [];
  for (const group of REPORT_GROUPS) {
    const inGroup = entries
      .filter((entry) => entry.group === group)
      .toSorted((a, b) => numberOf(a) - numberOf(b));
    const first = inGroup[0];
    if (first !== undefined)
      groups.push({ group, titleId: first.group_title_id, entries: inGroup });
  }
  return groups;
}

/** The message that says in one line what a report shows: `report-r11-title` has `report-r11-help`. */
export function helpIdOf(entry: ReportCatalogEntry): string {
  return entry.title_id.replace(/-title$/, '-help');
}

/**
 * What the party box of a report takes (PRD 19.2). The catalogue says only that a report has a party
 * box; which accounts it can be is part of the report: R2, R3, R4, R7 and the dues reports R20 to R22
 * are about a buyer, R24 about a seller, R9 about one-time people only (it takes no other account),
 * and the rest about any account.
 */
export type PartyBox = 'any' | 'buyer' | 'seller' | 'one_time';

const BUYER_REPORTS: ReadonlySet<ReportId> = new Set(['R2', 'R3', 'R4', 'R7', 'R20', 'R21', 'R22']);
const SELLER_REPORTS: ReadonlySet<ReportId> = new Set(['R24']);
const ONE_TIME_REPORTS: ReadonlySet<ReportId> = new Set(['R9']);

export function partyBoxOf(id: ReportId): PartyBox {
  if (ONE_TIME_REPORTS.has(id)) return 'one_time';
  if (BUYER_REPORTS.has(id)) return 'buyer';
  if (SELLER_REPORTS.has(id)) return 'seller';
  return 'any';
}
