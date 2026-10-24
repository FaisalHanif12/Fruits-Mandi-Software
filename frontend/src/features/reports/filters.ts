// What the filter boxes of a report hold, and how it becomes the filters the backend wants (PRD
// FR-19.1, FR-19.2, FR-19.10). The screen keeps the filters as typed (dates as text) and sends them
// as text: the backend reads the dates, the shortcuts and the party (CLAUDE.md rule 1). Nothing is
// worked out here.

import type { ReportDateShortcut } from '../../bindings/ReportDateShortcut';
import type { ReportFilterKind } from '../../bindings/ReportFilterKind';
import type { PukhtaPersonHit } from '../../bindings/PukhtaPersonHit';
import type { ReportFilters } from '../../bindings/ReportFilters';
import type { ReportGroupBy } from '../../bindings/ReportGroupBy';
import type { ReportPaidFilter } from '../../bindings/ReportPaidFilter';
import type { ReportTripView } from '../../bindings/ReportTripView';
import type { VoucherType } from '../../bindings/VoucherType';
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

/** The paid or due box of the one-time people register (R9): `all` is no filter. */
export type PaidChoice = ReportPaidFilter | 'all';

export const PAID_CHOICES: readonly PaidChoice[] = ['all', 'paid', 'due'];

/** The vouchers list (R15) takes one kind of voucher, or `all` for every kind (no filter). */
export type VoucherChoice = VoucherType | 'all';

/** In the order of the voucher kinds of PRD 15 and 16. Each has a `report-doc-<kind>` message. */
export const VOUCHER_CHOICES: readonly VoucherChoice[] = [
  'all',
  'receipt',
  'payment',
  'advance',
  'shop_expense',
  'personal_expense',
  'personal_return',
  'money_lent',
  'money_lent_return',
  'capital_in',
  'capital_out',
  'khata_out',
  'khata_in',
  'general',
];

/** The Fluent message of a voucher kind: `report-doc-shop-expense` for `shop_expense`. */
export function voucherMessageId(kind: VoucherType): string {
  return `report-doc-${kind.replaceAll('_', '-')}`;
}

/** The trip profit or loss (R33) is laid out by trip (the default), item, day, month or partner. */
export const TRIP_VIEW_CHOICES: readonly ReportTripView[] = [
  'trip',
  'item',
  'day',
  'month',
  'partner',
];

/** The Fluent message of a trip layout: `report-filter-trip-by-partner`. */
export function tripViewMessageId(view: ReportTripView): string {
  return `report-filter-trip-by-${view}`;
}

/** The income report groups its lines by month or by day (R28). */
export const GROUP_BY_CHOICES: readonly ReportGroupBy[] = ['month', 'day'];

/**
 * One known one-time person, picked in the person box (PRD FR-13.5, FR-13.9). He is a person inside
 * a پختہ کھاتہ account, so the backend wants the account and his key together. Both names are kept,
 * so a language switch only changes which one is shown.
 */
export interface ReportPerson {
  /** The پختہ کھاتہ account he has lines on. */
  accountId: number;
  /** The key that ties his lines together (name and phone, the backend's). */
  personKey: string;
  name_ur: string;
  name_en: string;
  phone: string;
  /** The day of his last line, `YYYY-MM-DD`, as the backend sent it. */
  last_date: string;
}

/**
 * A person the backend found, as the screen keeps him, with the account the report will ask for.
 * `accountId` is the one-time account the report is already on; null (R9, the global search) takes
 * his first account, the way the contract says. Null when he has no lines on the account asked for.
 */
export function personFromHit(hit: PukhtaPersonHit, accountId: number | null): ReportPerson | null {
  const account =
    accountId === null
      ? hit.accounts[0]
      : hit.accounts.find((candidate) => candidate.account_id === accountId);
  if (account === undefined) return null;
  return {
    accountId: account.account_id,
    personKey: hit.person_key,
    name_ur: hit.name_ur,
    name_en: hit.name_en,
    phone: hit.phone,
    last_date: hit.last_date,
  };
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
  /** Paid or due lines only (R9). */
  paid: PaidChoice;
  /** The income report by month or by day (R28). */
  groupBy: ReportGroupBy;
  /** One person inside a one-time account, when one is picked (R9 and any report on such an account). */
  person: ReportPerson | null;
  /** What is typed in the person box, kept here so a language switch loses nothing. */
  personText: string;
  /** The partner of a partner statement (R31, required) or of the expense report (R29, optional). */
  partner: number | null;
  /** One kind of voucher in the vouchers list (R15); `all` is no filter. */
  voucherType: VoucherChoice;
  /** How the trip profit or loss is laid out (R33). It is a layout, not a filter to clear. */
  tripView: ReportTripView;
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
  paid: 'all',
  groupBy: 'month',
  person: null,
  personText: '',
  partner: null,
  voucherType: 'all',
  tripView: 'trip',
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
  // A person inside a one-time account narrows the party to him; without one the party is the account.
  let party: ReportFilters['party'] = null;
  if (has('party')) {
    if (state.person !== null) {
      party = { account_id: state.person.accountId, person_key: state.person.personKey };
    } else if (state.party !== null) {
      party = { account_id: state.party.id, person_key: null };
    }
  }
  return {
    shortcut,
    from: dates && shortcut === null ? state.from.trim() : '',
    to: dates && shortcut === null ? state.to.trim() : '',
    as_of: has('as_of') ? state.asOf.trim() : '',
    party,
    seller_id: has('seller') ? (state.seller?.id ?? null) : null,
    zamindar_id: has('zamindar') ? (state.zamindar?.id ?? null) : null,
    item_id: has('item') ? (state.item?.id ?? null) : null,
    city_id: has('city') ? (state.city?.id ?? null) : null,
    adda_id: has('adda') ? (state.adda?.id ?? null) : null,
    partner_id: has('partner') ? state.partner : null,
    detail: has('detail') && state.detail,
    // The two newer keys are sent only when they are set; the backend reads a missing key as its
    // default (all lines; by month), so an old request stays valid.
    ...(has('status') && state.paid !== 'all' ? { status: state.paid } : {}),
    // A kind of voucher is sent only when one is picked (the backend reads a missing key as every
    // kind). The trip layout is sent whenever the report takes it, the default included.
    ...(has('voucher_type') && state.voucherType !== 'all'
      ? { voucher_type: state.voucherType }
      : {}),
    ...(has('trip_view') ? { trip_view: state.tripView } : {}),
    ...(has('group_by') && state.groupBy !== 'month' ? { group_by: state.groupBy } : {}),
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
    wire.partner_id === null &&
    !wire.detail &&
    wire.status === undefined &&
    wire.voucher_type === undefined
  );
}

/**
 * A report that needs a party or a partner cannot run until one is picked: R13 and R14 need an
 * account, R31 needs a partner (the catalogue says which box by the filters the report takes).
 */
export function canRun(
  state: ReportFilterState,
  entry: { filters: readonly ReportFilterKind[]; party_required: boolean },
): boolean {
  if (!entry.party_required) return true;
  return entry.filters.includes('partner') ? state.partner !== null : state.party !== null;
}
