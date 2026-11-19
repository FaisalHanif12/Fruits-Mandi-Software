// Types of the partners module. What the backend owns comes from the generated bindings (Code
// Standards F2). What the user types is text (CLAUDE.md rule 1): a share is the percent text
// as typed ("40", "33.33"), and the backend reads it. The screen never adds percents or money.

import type { CapitalHistory } from '../../bindings/CapitalHistory';
import type { CapitalHistoryInput } from '../../bindings/CapitalHistoryInput';
import type { CapitalInput } from '../../bindings/CapitalInput';
import type { CapitalRow } from '../../bindings/CapitalRow';
import type { PartnerCreateInput } from '../../bindings/PartnerCreateInput';
import type { PartnerDetail } from '../../bindings/PartnerDetail';
import type { PartnerOverviewRow } from '../../bindings/PartnerOverviewRow';
import type { PartnerRow } from '../../bindings/PartnerRow';
import type { PartnerStatement } from '../../bindings/PartnerStatement';
import type { PartnerUpdateInput } from '../../bindings/PartnerUpdateInput';
import type { PartnersSetSharesInput } from '../../bindings/PartnersSetSharesInput';
import type { PartnersStatementInput } from '../../bindings/PartnersStatementInput';
import type { ShareHistoryRow } from '../../bindings/ShareHistoryRow';
import type { ShareInput } from '../../bindings/ShareInput';
import type { SharesCheck } from '../../bindings/SharesCheck';
import type { StatementLine } from '../../bindings/StatementLine';
import type { VoucherEntry } from '../../bindings/VoucherEntry';

export type {
  CapitalHistory,
  CapitalHistoryInput,
  CapitalInput,
  CapitalRow,
  PartnerCreateInput,
  PartnerDetail,
  PartnerOverviewRow,
  PartnerRow,
  PartnerStatement,
  PartnerUpdateInput,
  PartnersSetSharesInput,
  PartnersStatementInput,
  ShareHistoryRow,
  ShareInput,
  SharesCheck,
  StatementLine,
};

/** The entry the preview takes for a capital move (PRD 17.4). */
export type CapitalEntry = Extract<VoucherEntry, { type: 'capital_in' | 'capital_out' }>;
