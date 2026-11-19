// The key of "everything the filter holds": the row of a total, and of a chart point that stands for a
// whole column (PRD FR-18.2). One place, so the table and the chart open the same lines.

import type { AnalyticsKey } from '../../bindings/AnalyticsKey';

export const WHOLE_FILTER_KEY: AnalyticsKey = {
  item_id: null,
  seller_id: null,
  buyer_id: null,
  person_key: null,
  from: null,
  to: null,
  not_item_ids: [],
  partner_id: null,
  purchase_ids: [],
};
