// Typed wrappers around the commands of the `analytics` module (plan 21, PRD 18). Components never
// call invoke() (Code Standards F4). Every command takes one `{ input }` object, except
// `analytics_catalog` and `analytics_views_get`, which take none. All need the signed-in Owner
// (NFR-SEC-13). Analytics writes nothing (FR-18.8), except the list of saved views.

import { callCommand } from '../../shared/api';
import type {
  AnalyticsCatalogEntry,
  AnalyticsLinesInput,
  AnalyticsLinesPage,
  AnalyticsPage,
  AnalyticsPrintData,
  AnalyticsPrintInput,
  AnalyticsRunInput,
  AnalyticsSavedView,
} from './types';
import type { AnalyticsViewsSaveInput } from '../../bindings/AnalyticsViewsSaveInput';

/** The views that are built, with the filter boxes each takes. */
export function analyticsCatalog(): Promise<AnalyticsCatalogEntry[]> {
  return callCommand<AnalyticsCatalogEntry[]>('analytics_catalog');
}

/**
 * One page of a view, with the totals over the whole filter (PRD FR-19.11). Send every key of the
 * filters; the backend refuses a filter the view does not take, if it is filled.
 */
export function analyticsRun(input: AnalyticsRunInput): Promise<AnalyticsPage> {
  return callCommand<AnalyticsPage>('analytics_run', { input });
}

/** The lines behind one number (PRD FR-18.2), each with the بکری it belongs to. */
export function analyticsLines(input: AnalyticsLinesInput): Promise<AnalyticsLinesPage> {
  return callCommand<AnalyticsLinesPage>('analytics_lines', { input });
}

/** Everything a print of a view needs, in the print language (PRD FR-18.4). */
export function analyticsPrintData(input: AnalyticsPrintInput): Promise<AnalyticsPrintData> {
  return callCommand<AnalyticsPrintData>('analytics_print_data', { input });
}

/** The saved views of this user (PRD FR-18.5): a name, a view and the filters. */
export function analyticsViewsGet(): Promise<AnalyticsSavedView[]> {
  return callCommand<AnalyticsSavedView[]>('analytics_views_get');
}

/** Replaces the whole list of saved views (at most 50) and returns what is kept. */
export function analyticsViewsSave(input: AnalyticsViewsSaveInput): Promise<AnalyticsSavedView[]> {
  return callCommand<AnalyticsSavedView[]>('analytics_views_save', { input });
}
