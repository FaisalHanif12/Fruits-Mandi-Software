// What the filter boxes of an analytics view hold, and how it becomes the filters the backend reads
// (PRD 18.1, FR-18.1). The screen keeps the filters as typed (dates as text, ids of the records
// picked) and sends them as they are: the backend reads the dates, the shortcuts and the party
// (CLAUDE.md rule 1). Nothing is worked out here.

import type { GradeChoice } from '../../shared/format';
import type { ItemGroupChoice, PickerValue } from '../../shared/ui';
import type {
  AnalyticsCompare,
  AnalyticsFilterKind,
  AnalyticsFilters,
  AnalyticsGoods,
  AnalyticsGroupBy,
  AnalyticsMeasure,
  AnalyticsPeriod,
  AnalyticsSide,
  AnalyticsTripView,
  AnalyticsView,
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
/**
 * The grouping of the views that may be read without one (A, B, C, E, F, I, J, K and L: PRD FR-18.9, plan 77): none,
 * or one row for each day, month or year, with the period as the first column.
 */
export const BUCKET_CHOICES = ['none', 'day', 'month', 'year'] as const;
export type BucketChoice = (typeof BUCKET_CHOICES)[number];

/**
 * The views whose grouping is a choice: left out, the view is read as a whole. Views D, G and M are always
 * grouped (a month when none is sent), so they keep `groupBy`.
 */
export const OPTIONAL_GROUPING_VIEWS: ReadonlySet<AnalyticsView> = new Set<AnalyticsView>([
  'A',
  'B',
  'C',
  'E',
  'F',
  'I',
  'J',
  'K',
  'L',
]);

/**
 * What the charts, the matrices and the rankings of a view show (PRD FR-18.10, plan 84, the box "ناپ"): sales
 * (the first and the default), the buyer's bill, the quantity, or one of the commissions. The tables of the
 * views keep all their columns; the measure decides the chart, the cells of views D, E and L with their
 * totals, and the order of the lists of view H. Quantity is a count, never rupees (the cell's kind says so).
 */
export const MEASURE_CHOICES = [
  'gross',
  'buyer_total',
  'qty',
  'total_comm',
  'seller_comm',
  'buyer_comm',
  'laga',
] as const satisfies readonly AnalyticsMeasure[];
export type MeasureChoice = AnalyticsMeasure;
/** The measure a new screen starts with: sales (PRD FR-18.10). */
export const DEFAULT_MEASURE: MeasureChoice = 'gross';
/**
 * The measure of a view that names none: the total commission, the one figure the views drew before plan 84. A
 * view saved before then opens as it was saved, and a result that names none is read so.
 */
export const CLASSIC_MEASURE: MeasureChoice = 'total_comm';

/** The column of views D, E and L that adds up a row: its heading names the measure its cells hold. */
export const ROW_TOTAL_TITLE_ID = 'analytics-col-row-total';

/**
 * The heading message of a column for a measure (PRD FR-18.10): the row total names what its cells hold ("Total
 * Gross Sale"); every other column keeps its message. Total commission is the message the column always had.
 */
export function measureTitleId(titleId: string, measure: MeasureChoice): string {
  return titleId === ROW_TOTAL_TITLE_ID && measure !== 'total_comm'
    ? `${titleId}-${measure.replaceAll('_', '-')}`
    : titleId;
}

/**
 * True when a view's catalogue entry lists the measure: views A, B, C, D, E, G, H and L. The others (F, I, J, K, M)
 * never get one from the screen (the backend refuses it there).
 */
export function takesMeasure(takes: readonly string[]): boolean {
  return takes.includes('measure');
}

export const SIDE_CHOICES: readonly AnalyticsSide[] = ['both', 'seller', 'buyer'];
export const COMPARE_CHOICES: readonly AnalyticsCompare[] = ['off', 'previous', 'last_year'];
/** Which goods: all of them, those consigned to the shop, the trips' goods or the gardens' (PRD 18.1). */
export const GOODS_CHOICES: readonly AnalyticsGoods[] = ['all', 'consigned', 'trip', 'garden'];
/** How the trips view (J) is laid out: by trip (the first), purchase, item or day (PRD 18.2). */
export const TRIP_VIEW_CHOICES: readonly AnalyticsTripView[] = ['trip', 'purchase', 'item', 'day'];

/**
 * The Fluent message of a trip layout: the report words where they exist (by trip, item, day), else
 * the analytics one (by purchase).
 */
export function tripViewMessageId(view: AnalyticsTripView): string {
  return view === 'purchase'
    ? 'analytics-filter-trip-by-purchase'
    : `report-filter-trip-by-${view}`;
}

/** The Fluent message of a period choice: the report words where they exist, else the analytics ones. */
export function periodMessageId(choice: PeriodChoice): string {
  return choice === 'this_year' || choice === 'last_year'
    ? `analytics-period-${choice.replaceAll('_', '-')}`
    : `report-period-${choice.replaceAll('_', '-')}`;
}

export interface AnalyticsFilterState {
  period: PeriodChoice;
  /** `DD-MM-YYYY` as typed (the backend also reads `10` and `10-9`). */
  from: string;
  to: string;
  /** Views D, G and M: the columns are a day, a month (the first choice) or a year. */
  groupBy: AnalyticsGroupBy;
  /** Views F, I, J and K: no grouping (the first choice), or a row for each day, month or year. */
  buckets: BucketChoice;
  item: PickerValue | null;
  seller: PickerValue | null;
  buyer: PickerValue | null;
  city: PickerValue | null;
  adda: PickerValue | null;
  side: AnalyticsSide;
  compare: AnalyticsCompare;
  /** One partner (views F, G, J), or all of them (null). */
  partner: number | null;
  /** One trip (views that take it), or all (null). */
  trip: number | null;
  /** One garden (view I and the views that take it), or all (null). */
  garden: number | null;
  goods: AnalyticsGoods;
  /** All the items of a group, fruit or vegetables (views A, D and G); never together with an item. */
  itemGroup: ItemGroupChoice;
  /** The quality of the sale lines (view A; PRD FR-12.42). `all` is no filter. */
  grade: GradeChoice;
  /** The layout of view J. */
  tripView: AnalyticsTripView;
  /** What the chart, the matrices and the lists of the views that take it show (PRD FR-18.10). */
  measure: MeasureChoice;
}

export const EMPTY_ANALYTICS_FILTERS: AnalyticsFilterState = {
  period: 'custom',
  from: '',
  to: '',
  groupBy: 'month',
  buckets: 'none',
  item: null,
  seller: null,
  buyer: null,
  city: null,
  adda: null,
  side: 'both',
  compare: 'off',
  partner: null,
  trip: null,
  garden: null,
  goods: 'all',
  itemGroup: 'all',
  grade: 'all',
  tripView: 'trip',
  measure: DEFAULT_MEASURE,
};

/**
 * The filters as the backend reads them. Every key is sent. A box the view does not take is sent
 * empty, because the backend refuses a filter the view does not take if it is filled. `group_by` is
 * sent only for the day and year choices: month is the backend's default, so it is left null, like
 * a request of the backend's own tests. A view whose grouping is a choice (`OPTIONAL_GROUPING_VIEWS`) sends its
 * `buckets` as they are, and nothing for "none".
 */
export function filtersToWire(
  state: AnalyticsFilterState,
  takes: readonly AnalyticsFilterKind[],
  view?: AnalyticsView,
): AnalyticsFilters {
  const has = (kind: AnalyticsFilterKind) => takes.includes(kind);
  const period = has('period') && state.period !== 'custom' ? state.period : null;
  // A چکتا buyer has no ledger and is not in the buyer box (PRD FR-13.4), so a buyer is an account.
  const buyer: AnalyticsFilters['buyer'] =
    has('buyer') && state.buyer !== null ? { account_id: state.buyer.id, person_key: null } : null;
  return {
    period,
    from: has('period') && period === null ? state.from.trim() : '',
    to: has('period') && period === null ? state.to.trim() : '',
    group_by: !has('group_by') ? null : groupByOf(state, view),
    item_id: has('item') ? (state.item?.id ?? null) : null,
    seller_id: has('seller') ? (state.seller?.id ?? null) : null,
    buyer,
    city_id: has('city') ? (state.city?.id ?? null) : null,
    adda_id: has('adda') ? (state.adda?.id ?? null) : null,
    side: has('side') ? state.side : 'both',
    compare: has('compare') ? state.compare : 'off',
    partner_id: has('partner') ? state.partner : null,
    trip_id: has('trip') ? state.trip : null,
    garden_id: has('garden') ? state.garden : null,
    goods: has('goods') ? state.goods : 'all',
    // A group of items is sent only when it is chosen, and never with one item (the backend refuses both).
    ...(has('item_group') && state.itemGroup !== 'all' && state.item === null
      ? { item_group: state.itemGroup }
      : {}),
    // One quality is sent only when it is chosen (the backend reads a missing key as every line).
    ...(has('grade') && state.grade !== 'all' ? { grade: state.grade } : {}),
    // The first layout is the backend's default, so it is left null, like the month of group_by.
    trip_view: has('trip_view') && state.tripView !== 'trip' ? state.tripView : null,
    // The screen always sends the measure to a view that lists it (a view without one reads the classic total
    // commission), and never to a view that takes none (the backend refuses it there).
    ...(takesMeasure(takes) ? { measure: state.measure } : {}),
  };
}

/** The grouping that is sent for a view: its buckets, or its month, day or year (month is the default). */
function groupByOf(
  state: AnalyticsFilterState,
  view: AnalyticsView | undefined,
): AnalyticsGroupBy | null {
  if (view !== undefined && OPTIONAL_GROUPING_VIEWS.has(view)) {
    return state.buckets === 'none' ? null : state.buckets;
  }
  return state.groupBy === 'month' ? null : state.groupBy;
}

/** A record that was picked before its names are known: they come with the first answer. */
function bare(id: number | null): PickerValue | null {
  return id === null ? null : { id, name_ur: '', name_en: '', code: undefined };
}

/** The boxes a saved view starts with: ids only; the names are filled by the answer's echo. */
export function stateFromWire(filters: AnalyticsFilters): AnalyticsFilterState {
  const buyerId = filters.buyer?.account_id ?? null;
  return {
    period: filters.period ?? 'custom',
    from: filters.from,
    to: filters.to,
    groupBy: filters.group_by ?? 'month',
    buckets: filters.group_by ?? 'none',
    item: bare(filters.item_id),
    seller: bare(filters.seller_id),
    buyer: bare(buyerId),
    city: bare(filters.city_id),
    adda: bare(filters.adda_id),
    side: filters.side,
    compare: filters.compare,
    partner: filters.partner_id,
    trip: filters.trip_id,
    garden: filters.garden_id,
    goods: filters.goods,
    itemGroup: filters.item_group ?? 'all',
    grade: filters.grade ?? 'all',
    tripView: filters.trip_view ?? 'trip',
    measure: measureOfFilters(filters),
  };
}

/**
 * The measure a set of filters (the ones sent, or the ones a result echoes) carries. Filters that name none
 * are the classic view: the total commission (a view saved before the measure existed reads so).
 */
export function measureOfFilters(filters: {
  measure?: AnalyticsMeasure | undefined;
}): MeasureChoice {
  return MEASURE_CHOICES.find((choice) => choice === filters.measure) ?? CLASSIC_MEASURE;
}

/** True when no box that the view takes is filled: nothing to clear. */
export function isUnfiltered(
  state: AnalyticsFilterState,
  takes: readonly AnalyticsFilterKind[],
  view?: AnalyticsView,
): boolean {
  const wire = filtersToWire(state, takes, view);
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
    wire.compare === 'off' &&
    wire.partner_id === null &&
    wire.trip_id === null &&
    wire.garden_id === null &&
    wire.goods === 'all' &&
    wire.item_group === undefined &&
    wire.grade === undefined
  );
}
