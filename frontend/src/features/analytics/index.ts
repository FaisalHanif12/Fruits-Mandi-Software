// The public side of the `analytics` module (Tech Stack 5.2): what other modules may import.
export {
  analyticsCatalog,
  analyticsLines,
  analyticsPrintData,
  analyticsRun,
  analyticsViewsGet,
  analyticsViewsSave,
} from './api';
export {
  ANALYTICS_PAGE_SIZE,
  useAnalyticsCatalog,
  useAnalyticsLines,
  useAnalyticsPages,
  useAnalyticsPrintData,
  useSavedViews,
  useSaveViews,
} from './hooks';
export type { AnalyticsSavedView, AnalyticsView } from './types';
