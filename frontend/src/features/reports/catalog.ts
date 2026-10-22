// The catalogue of reports grouped as PRD 19.2 (Sales, Accounts, Collections, Business). The order
// of the groups and of the reports inside a group is the PRD's; the backend sends the list and
// each report's filters, the screen only groups it (no report is named here that the backend does
// not send).

import type { ReportCatalogEntry } from '../../bindings/ReportCatalogEntry';
import type { ReportGroup } from '../../bindings/ReportGroup';

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
