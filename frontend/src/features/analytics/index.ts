// The public side of the `analytics` module (Tech Stack 5.2): what other modules may import.
export {
  analyticsCatalog,
  analyticsLines,
  analyticsPrintData,
  analyticsPrintView,
  analyticsRun,
  analyticsViewsGet,
  analyticsViewsSave,
} from './api';
export {
  useAnalyticsCatalog,
  useAnalyticsLines,
  useAnalyticsPages,
  useAnalyticsPrintData,
  useSavedViews,
  useSaveViews,
} from './hooks';
export { isAnalyticsAddress } from './print/params';
// Plan 49 (ask the app): a button under an answer opens a view with its filters in the address.
export { parseAnalyticsSearch, type AnalyticsSearch } from './search';
// The words and figures a table cell is written with, for the question panel's tables (plan 49).
export { useListContext } from './list-context';
export { AnalyticsPrintPage } from './print/AnalyticsPrintPage';
export type { AnalyticsSavedView, AnalyticsView } from './types';
