// The address of the hidden print window for an analytics view: `#/print?analytics=<view>&language=
// <ur|en>&paper=<a4|a5>&filters=<percent-encoded JSON of AnalyticsFilters>[&sort=<percent-encoded JSON of
// the screen's SortInput>]&job=<job>` (17e contract, plan 38).
// The window is opened by the backend, so every value is read as plain text and checked here. A value
// that is not right is a failed print, never a guess. The filters are the backend's own JSON for the
// same filters the screen asked with; the print-data command checks them again (Code Standards V0).

import { z } from 'zod';
import type { AnalyticsFilters } from '../../../bindings/AnalyticsFilters';
import type { AnalyticsPrintInput } from '../../../bindings/AnalyticsPrintInput';
import type { AnalyticsView } from '../../../bindings/AnalyticsView';
import { jobOf } from '../../../shared/print';
import { LANGUAGE_VALUES, paperOf, readJsonObject, sortOf } from '../../reports';

export interface AnalyticsPrintParams {
  /** The job name; the events are `print-ready-<job>` and `print-error-<job>`. */
  job: string;
  input: AnalyticsPrintInput;
}

/** Every view of the analytics (PRD 18.2). A test holds the list to the backend's catalogue. */
export const ANALYTICS_VIEW_IDS: readonly AnalyticsView[] = [
  'A',
  'B',
  'C',
  'D',
  'E',
  'F',
  'G',
  'H',
  'I',
  'J',
  'K',
];

const filtersSchema = z.custom<AnalyticsFilters>(
  (value) => typeof value === 'object' && value !== null && !Array.isArray(value),
);

/** True when the address is for an analytics view (it names one), even if the rest is wrong. */
export function isAnalyticsAddress(searchStr: string): boolean {
  return new URLSearchParams(searchStr).has('analytics');
}

/** The parameters of the print window for a view, or null when the address is not valid. */
export function parseAnalyticsPrintParams(searchStr: string): AnalyticsPrintParams | null {
  const search = new URLSearchParams(searchStr);
  const job = jobOf(searchStr);
  const view = ANALYTICS_VIEW_IDS.find((candidate) => candidate === search.get('analytics'));
  const language = LANGUAGE_VALUES.find((candidate) => candidate === search.get('language'));
  const paper = paperOf(search.get('paper'));
  const filters = readJsonObject(search.get('filters'), filtersSchema);
  if (job === null || view === undefined || language === undefined || paper === undefined) {
    return null;
  }
  const sort = sortOf(search.get('sort'));
  if (filters === null || sort === undefined) return null;
  // The order of the screen is in the address when the screen had one: the print has every row in it.
  return { job, input: { view, filters, language, paper, sort } };
}
