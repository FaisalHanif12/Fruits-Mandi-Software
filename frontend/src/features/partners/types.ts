// Types of the partners module. What the backend owns comes from the generated bindings (Code
// Standards F2). What the user types is text (CLAUDE.md rule 1): a share is the percent text
// as typed ("40", "33.33"), and the backend reads it. The screen never adds percents or money.

import type { PartnerCreateInput } from '../../bindings/PartnerCreateInput';
import type { PartnerDetail } from '../../bindings/PartnerDetail';
import type { PartnerOverviewRow } from '../../bindings/PartnerOverviewRow';
import type { PartnerRow } from '../../bindings/PartnerRow';
import type { PartnerUpdateInput } from '../../bindings/PartnerUpdateInput';
import type { PartnersSetSharesInput } from '../../bindings/PartnersSetSharesInput';
import type { ShareHistoryRow } from '../../bindings/ShareHistoryRow';
import type { ShareInput } from '../../bindings/ShareInput';
import type { SharesCheck } from '../../bindings/SharesCheck';

export type {
  PartnerCreateInput,
  PartnerDetail,
  PartnerOverviewRow,
  PartnerRow,
  PartnerUpdateInput,
  PartnersSetSharesInput,
  ShareHistoryRow,
  ShareInput,
  SharesCheck,
};
