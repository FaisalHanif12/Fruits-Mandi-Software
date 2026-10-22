// What the filter boxes of a report hold, and how it becomes the filters the backend wants (PRD
// FR-19.1, FR-19.2, FR-19.10). The screen keeps the filters as typed (dates as text) and sends them
// as text: the backend reads the dates, the shortcuts and the party (CLAUDE.md rule 1). Nothing is
// worked out here.

import type { ReportDateShortcut } from '../../bindings/ReportDateShortcut';
import type { ReportFilterKind } from '../../bindings/ReportFilterKind';
import type { ReportFilters } from '../../bindings/ReportFilters';
import type { PickerValue } from '../../shared/ui';

/** The period box: a shortcut of FR-19.2, or the two dates the user types. */
export type PeriodChoice = ReportDateShortcut | 'custom';

export const PERIOD_CHOICES: readonly PeriodChoice[] = [
  'custom',
  'today',
  'yesterday',
  'this_week',
  'this_month',
  'last_month',
];

/** The Fluent message of a period choice. */
export function periodMessageId(choice: PeriodChoice): string {
  return `report-period-${choice.replaceAll('_', '-')}`;
}

/** What the filter boxes hold. Every box that a report does not take is simply not shown. */
export interface ReportFilterState {
  period: PeriodChoice;
  /** `DD-MM-YYYY` as typed (the backend also reads `10` and `10-9`). */
  from: string;
  to: string;
  asOf: string;
  party: PickerValue | null;
  seller: PickerValue | null;
  zamindar: PickerValue | null;
  item: PickerValue | null;
  city: PickerValue | null;
  adda: PickerValue | null;
  detail: boolean;
}

export const EMPTY_FILTERS: ReportFilterState = {
  period: 'custom',
  from: '',
  to: '',
  asOf: '',
  party: null,
  seller: null,
  zamindar: null,
  item: null,
  city: null,
  adda: null,
  detail: false,
};

/**
 * The filters as the backend reads them. Every key is sent. A box the report does not take is
 * sent empty, because the backend refuses a filter the report does not take if it is filled
 * (ERR_OUT_OF_RANGE), and a box left over from another report must never reach it.
 */
export function filtersToWire(
  state: ReportFilterState,
  takes: readonly ReportFilterKind[],
): ReportFilters {
  const has = (kind: ReportFilterKind) => takes.includes(kind);
  const dates = has('date_range');
  const shortcut = dates && state.period !== 'custom' ? state.period : null;
  return {
    shortcut,
    from: dates && shortcut === null ? state.from.trim() : '',
    to: dates && shortcut === null ? state.to.trim() : '',
    as_of: has('as_of') ? state.asOf.trim() : '',
    party:
      has('party') && state.party !== null
        ? { account_id: state.party.id, person_key: null }
        : null,
    seller_id: has('seller') ? (state.seller?.id ?? null) : null,
    zamindar_id: has('zamindar') ? (state.zamindar?.id ?? null) : null,
    item_id: has('item') ? (state.item?.id ?? null) : null,
    city_id: has('city') ? (state.city?.id ?? null) : null,
    adda_id: has('adda') ? (state.adda?.id ?? null) : null,
    detail: has('detail') && state.detail,
  };
}

/** True when no box that the report takes is filled: nothing to clear. */
export function isUnfiltered(
  state: ReportFilterState,
  takes: readonly ReportFilterKind[],
): boolean {
  const wire = filtersToWire(state, takes);
  return (
    wire.shortcut === null &&
    wire.from === '' &&
    wire.to === '' &&
    wire.as_of === '' &&
    wire.party === null &&
    wire.seller_id === null &&
    wire.zamindar_id === null &&
    wire.item_id === null &&
    wire.city_id === null &&
    wire.adda_id === null &&
    !wire.detail
  );
}

/** A report that needs a party cannot run until one is picked (R13, R14). */
export function canRun(
  state: ReportFilterState,
  entry: { filters: readonly ReportFilterKind[]; party_required: boolean },
): boolean {
  return !entry.party_required || state.party !== null;
}
