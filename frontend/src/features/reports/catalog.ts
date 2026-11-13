// The catalogue of reports grouped as PRD 19.2 (Sales, Accounts, Collections, Business). The order
// of the groups and of the reports inside a group is the PRD's; the backend sends the list and
// each report's filters, the screen only groups it (no report is named here that the backend does
// not send).

import type { ReportCatalogEntry } from '../../bindings/ReportCatalogEntry';
import type { ReportFilterKind } from '../../bindings/ReportFilterKind';
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
 * are about a buyer, R24 about a seller, R32 about the people who can borrow (control 0103, PRD 16.5), and the rest about any account.
 */
export type PartyBox = 'any' | 'buyer' | 'seller' | 'lent';

const BUYER_REPORTS: ReadonlySet<ReportId> = new Set(['R2', 'R3', 'R4', 'R7', 'R20', 'R21', 'R22']);
const SELLER_REPORTS: ReadonlySet<ReportId> = new Set(['R24']);
const LENT_REPORTS: ReadonlySet<ReportId> = new Set(['R32']);

export function partyBoxOf(id: ReportId): PartyBox {
  if (LENT_REPORTS.has(id)) return 'lent';
  if (BUYER_REPORTS.has(id)) return 'buyer';
  if (SELLER_REPORTS.has(id)) return 'seller';
  return 'any';
}

/** The message that says what a report shows, in a few lines: `report-r5-title` has `report-r5-detail` (FR-19.14). */
export function detailIdOf(entry: ReportCatalogEntry): string {
  return `report-${entry.id.toLowerCase()}-detail`;
}

/** The message that says when to use a report, in one line: `report-r5-when` (FR-19.14). */
export function whenIdOf(entry: ReportCatalogEntry): string {
  return `report-${entry.id.toLowerCase()}-when`;
}

/**
 * The Fluent message of each filter box a report can have, the same words as the box itself (the filter
 * bar of the report). `null` is a kind that has no box of its own to name: the guests choice is part of
 * the partner box, and a lot has no box (it is set only by an address). Every kind is listed, so a
 * kind the backend adds later stops the build here until it is given its words.
 */
const FILTER_LABEL_IDS: Record<ReportFilterKind, string | null> = {
  date_range: 'report-filter-period',
  as_of: 'report-filter-as-of',
  party: 'report-filter-party',
  seller: 'report-filter-seller',
  zamindar: 'report-filter-zamindar',
  item: 'report-filter-item',
  city: 'report-filter-city',
  adda: 'report-filter-adda',
  detail: 'report-filter-detail',
  status: 'report-filter-status',
  group_by: 'report-filter-group-by',
  partner: 'report-filter-partner',
  voucher_type: 'report-filter-voucher-type',
  trip_view: 'report-filter-trip-view',
  garden: 'report-filter-garden',
  garden_view: 'report-filter-garden-view',
  bikri: 'report-filter-bikri',
  guests: null,
  collector: 'report-filter-collector',
  item_group: 'common-item-group',
  stock_view: 'report-filter-stock-view',
  grade: 'common-grade',
  lot: null,
};

/**
 * The words of the filter boxes a report shows, in the order the catalogue gives them. The day sheet
 * (R35) asks for one day and the money lent report (R32) for a borrower, as their own boxes do.
 */
export function filterLabelIdsOf(entry: ReportCatalogEntry): string[] {
  const ids: string[] = [];
  for (const kind of entry.filters) {
    let id = FILTER_LABEL_IDS[kind];
    if (kind === 'as_of' && entry.id === 'R35') id = 'report-filter-day';
    if (kind === 'party' && partyBoxOf(entry.id) === 'lent') id = 'report-filter-borrower';
    if (id !== null) ids.push(id);
  }
  return ids;
}

/**
 * The print only some reports have, besides the preview every report has (PRD FR-19.3, FR-12.12, R5, R35): the
 * seller's bill from the sale book, one slip for each buyer from the credit details, and one party's slip
 * or the table of every party from the day sheet. The message of the special print, or null.
 */
export function specialPrintIdOf(entry: ReportCatalogEntry): string | null {
  switch (entry.id) {
    case 'R1':
    case 'R5':
    case 'R35':
      return `report-guide-special-${entry.id.toLowerCase()}`;
    default:
      return null;
  }
}

/**
 * The Fluent messages of a report's column headings, in the order the report shows them. They come from
 * the same definition the report runs with (`column_title_ids` of the catalogue), so the guide card
 * never lists a column the report does not have.
 */
export function columnTitleIdsOf(entry: ReportCatalogEntry): readonly string[] {
  return entry.column_title_ids;
}
