// What the filter boxes of an analytics view hold, and how it becomes the filters the backend reads
// (PRD 18.1, FR-18.1). The screen keeps the filters as typed (dates as text, ids of the records
// picked) and sends them as they are: the backend reads the dates, the shortcuts and the party
// (CLAUDE.md rule 1). Nothing is worked out here.

import type { ReportPerson } from '../reports';
import type { PickerValue } from '../../shared/ui';
import type {
  AnalyticsCompare,
  AnalyticsFilterKind,
  AnalyticsFilters,
  AnalyticsGroupBy,
  AnalyticsPeriod,
  AnalyticsSide,
} from './types';

/** The period box: a shortcut of PRD 18.1, or the two dates the user types. */
export type PeriodChoice = AnalyticsPeriod | 'custom';

export const PERIOD_CHOICES: readonly PeriodChoice[] = [
  'custom',
  'today',
  'this_week',
  'this_month',
  'last_month',
  'this_year',
  'last_year',
];

export const GROUP_BY_CHOICES: readonly AnalyticsGroupBy[] = ['month', 'day', 'year'];
export const SIDE_CHOICES: readonly AnalyticsSide[] = ['both', 'seller', 'buyer'];
export const COMPARE_CHOICES: readonly AnalyticsCompare[] = ['off', 'previous', 'last_year'];

/** The Fluent message of a period choice: the report words where they exist, else the analytics ones. */
export function periodMessageId(choice: PeriodChoice): string {
  const dashed = choice.replaceAll('_', '-');
  return choice === 'this_year' || choice === 'last_year'
    ? `analytics-period-${dashed}`
    : `report-period-${dashed}`;
}

export interface AnalyticsFilterState {
  period: PeriodChoice;
  /** `DD-MM-YYYY` as typed (the backend also reads `10` and `10-9`). */
  from: string;
  to: string;
  /** View D only. */
  groupBy: AnalyticsGroupBy;
  item: PickerValue | null;
  seller: PickerValue | null;
  buyer: PickerValue | null;
  /** One person inside a one-time buyer account (PRD FR-13.5). */
  person: ReportPerson | null;
  /** What is typed in the person box, kept here so a language switch loses nothing. */
  personText: string;
  city: PickerValue | null;
  adda: PickerValue | null;
  side: AnalyticsSide;
  compare: AnalyticsCompare;
}

export const EMPTY_ANALYTICS_FILTERS: AnalyticsFilterState = {
  period: 'custom',
  from: '',
  to: '',
  groupBy: 'month',
  item: null,
  seller: null,
  buyer: null,
  person: null,
  personText: '',
  city: null,
  adda: null,
  side: 'both',
  compare: 'off',
};

/**
 * The filters as the backend reads them. Every key is sent. A box the view does not take is sent
 * empty, because the backend refuses a filter the view does not take if it is filled. `group_by` is
 * sent only for the day and year choices: month is the backend's default, so it is left null, like
 * a request of the backend's own tests.
 */
export function filtersToWire(
  state: AnalyticsFilterState,
  takes: readonly AnalyticsFilterKind[],
): AnalyticsFilters {
  const has = (kind: AnalyticsFilterKind) => takes.includes(kind);
  const period = has('period') && state.period !== 'custom' ? state.period : null;
  let buyer: AnalyticsFilters['buyer'] = null;
  if (has('buyer')) {
    if (state.person !== null) {
      buyer = { account_id: state.person.accountId, person_key: state.person.personKey };
    } else if (state.buyer !== null) {
      buyer = { account_id: state.buyer.id, person_key: null };
    }
  }
  return {
    period,
    from: has('period') && period === null ? state.from.trim() : '',
    to: has('period') && period === null ? state.to.trim() : '',
    group_by: has('group_by') && state.groupBy !== 'month' ? state.groupBy : null,
    item_id: has('item') ? (state.item?.id ?? null) : null,
    seller_id: has('seller') ? (state.seller?.id ?? null) : null,
    buyer,
    city_id: has('city') ? (state.city?.id ?? null) : null,
    adda_id: has('adda') ? (state.adda?.id ?? null) : null,
    side: has('side') ? state.side : 'both',
    compare: has('compare') ? state.compare : 'off',
  };
}

/** A record that was picked before its names are known: they come with the first answer. */
function bare(id: number | null): PickerValue | null {
  return id === null ? null : { id, name_ur: '', name_en: '', code: undefined };
}

/** The boxes a saved view starts with: ids only; the names are filled by the answer's echo. */
export function stateFromWire(filters: AnalyticsFilters): AnalyticsFilterState {
  const buyerId = filters.buyer?.account_id ?? null;
  const personKey = filters.buyer?.person_key ?? null;
  const personName = personKey === null ? '' : (personKey.split('|')[0] ?? '');
  return {
    period: filters.period ?? 'custom',
    from: filters.from,
    to: filters.to,
    groupBy: filters.group_by ?? 'month',
    item: bare(filters.item_id),
    seller: bare(filters.seller_id),
    buyer: bare(buyerId),
    person:
      buyerId === null || personKey === null
        ? null
        : {
            accountId: buyerId,
            personKey,
            name_ur: personName,
            name_en: personName,
            phone: '',
            last_date: '',
          },
    personText: '',
    city: bare(filters.city_id),
    adda: bare(filters.adda_id),
    side: filters.side,
    compare: filters.compare,
  };
}

/** True when no box that the view takes is filled: nothing to clear. */
export function isUnfiltered(
  state: AnalyticsFilterState,
  takes: readonly AnalyticsFilterKind[],
): boolean {
  const wire = filtersToWire(state, takes);
  return (
    wire.period === null &&
    wire.from === '' &&
    wire.to === '' &&
    wire.item_id === null &&
    wire.seller_id === null &&
    wire.buyer === null &&
    wire.city_id === null &&
    wire.adda_id === null &&
    wire.side === 'both' &&
    wire.compare === 'off'
  );
}
