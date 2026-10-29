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
  useAnalyticsCatalog,
  useAnalyticsLines,
  useAnalyticsPages,
  useAnalyticsPrintData,
  useSavedViews,
  useSaveViews,
} from './hooks';
export { isAnalyticsAddress } from './print/params';
export { AnalyticsPrintPage } from './print/AnalyticsPrintPage';
export type { AnalyticsSavedView, AnalyticsView } from './types';
