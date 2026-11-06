// What the address of a report can carry (PRD FR-7.65): `#/reports/R13?party=20&from=01-10-2026`
// opens the ledger of account 20 with the dates already filled, the way the party card's buttons
// do. Everything is optional, and anything that does not read is left out.

import type { ReportGardenView } from '../../bindings/ReportGardenView';
import type { ReportGroupBy } from '../../bindings/ReportGroupBy';
import type { ReportPaidFilter } from '../../bindings/ReportPaidFilter';
import type { ReportTripView } from '../../bindings/ReportTripView';
import { pickedOrNull } from '../../shared/screen-state';
import { ITEM_GROUP_CHOICES, type ItemGroupChoice, type PickerValue } from '../../shared/ui';
import type {
  PeriodChoice,
  ReportBikri,
  ReportFilterState,
  ReportPerson,
  VoucherChoice,
} from './filters';
import {
  EMPTY_FILTERS,
  GARDEN_VIEW_CHOICES,
  GROUP_BY_CHOICES,
  PAID_CHOICES,
  PERIOD_CHOICES,
  TRIP_VIEW_CHOICES,
  VOUCHER_CHOICES,
} from './filters';

export interface ReportSearch {
  period?: PeriodChoice;
  from?: string;
  to?: string;
  as_of?: string;
  party?: number;
  /** A partner (R29, R31): the partner screens open his statement this way. */
  partner?: number;
  /** A one-time person (PRD FR-13.5): the global search opens his register this way. */
  person?: ReportPerson;
  /** The fruit register (R37) for the shop's guests only: the fruit screen's "Print" opens it this way. */
  guests?: boolean;
  /**
   * One item (R37 and every report that takes one): the item chosen in the fruit list, whole, or
   * only its id (the question panel, plan 49), or not at all.
   */
  item?: PickerValue;
  // ---- Plan 49 (ask the app): the rest of the report filters, so a button under an answer opens a
  // report exactly as the model read it. Each is optional; with none of them the address reads as it
  // always did. A record is its whole picker value or just its id (a bare id has no names yet). ----
  /** A seller account (R6 and the other reports that take one). */
  seller?: PickerValue;
  /** A زمیندار account. */
  zamindar?: PickerValue;
  /** A city. */
  city?: PickerValue;
  /** An اڈا. */
  adda?: PickerValue;
  /** One collector, a staff member (R22, R23, R35). */
  collector?: PickerValue;
  /** One kind of voucher (R15); `all` is the same as leaving it out. */
  voucher_type?: VoucherChoice;
  /** The group of the items, fruit or vegetables; `all` is the same as leaving it out. */
  item_group?: ItemGroupChoice;
  /** How the trip profit or loss (R33) is laid out. */
  trip_view?: ReportTripView;
  /** One garden (R36), by its id. */
  garden?: number;
  /** How the gardens report (R36) is laid out. */
  garden_view?: ReportGardenView;
  /** The one بکری of the sale book report (R1): its id and its number to show. */
  bikri?: ReportBikri;
  /** The income report (R28) by month or by day. */
  group_by?: ReportGroupBy;
  /** Paid or due lines only (R9); `all` is the same as leaving it out. */
  status?: ReportPaidFilter;
  /** The statement with its detail lines (R14). */
  detail?: boolean;
}

function text(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() !== '' ? value.trim() : undefined;
}

/** A positive whole number, from a number or from the text of one (an address carries text). */
function positiveId(value: unknown): number | undefined {
  const id = typeof value === 'string' && value.trim() !== '' ? Number(value) : value;
  return typeof id === 'number' && Number.isSafeInteger(id) && id > 0 ? id : undefined;
}

/** A record of the address: a whole picker value, or only an id, which has no names yet. */
function readPicked(value: unknown): PickerValue | undefined {
  const whole = pickedOrNull(value);
  if (whole !== null) return whole;
  const id = positiveId(value);
  return id === undefined ? undefined : { id, name_ur: '', name_en: '', code: undefined };
}

/** One of the values a screen knows, or nothing. */
function oneOf<T extends string>(choices: readonly T[], value: unknown): T | undefined {
  return choices.find((choice) => choice === value);
}

/**
 * The بکری of the sale book report: its id and its number (`bikri_number`), whole or not at all. A
 * row id is not the number the Owner knows, so a missing number is never replaced by the id.
 */
function readBikri(id: unknown, number: unknown): ReportBikri | undefined {
  const bikriId = positiveId(id);
  const bikriNumber = positiveId(number);
  return bikriId === undefined || bikriNumber === undefined
    ? undefined
    : { id: bikriId, number: bikriNumber };
}

/** A person in the address, whole or not at all: a half-read person would ask for the wrong lines. */
function readPerson(value: unknown): ReportPerson | undefined {
  if (typeof value !== 'object' || value === null) return undefined;
  const found = value as Record<string, unknown>;
  const accountId = found['accountId'];
  const personKey = found['personKey'];
  const nameUr = found['name_ur'];
  const nameEn = found['name_en'];
  const phone = found['phone'];
  const lastDate = found['last_date'];
  if (
    typeof accountId !== 'number' ||
    !Number.isSafeInteger(accountId) ||
    accountId <= 0 ||
    typeof personKey !== 'string' ||
    personKey === '' ||
    typeof nameUr !== 'string' ||
    typeof nameEn !== 'string' ||
    typeof phone !== 'string' ||
    typeof lastDate !== 'string'
  ) {
    return undefined;
  }
  return { accountId, personKey, name_ur: nameUr, name_en: nameEn, phone, last_date: lastDate };
}

/** Reads the address. A period must be one the screen knows; a party must be a positive whole number. */
export function parseReportSearch(search: Record<string, unknown>): ReportSearch {
  const period = PERIOD_CHOICES.find((choice) => choice === search['period']);
  const from = text(search['from']);
  const to = text(search['to']);
  const asOf = text(search['as_of']);
  const party = Number(search['party']);
  const partner = Number(search['partner']);
  const person = readPerson(search['person']);
  const guests = search['guests'] === true || search['guests'] === 'true';
  const item = readPicked(search['item']);
  const seller = readPicked(search['seller']);
  const zamindar = readPicked(search['zamindar']);
  const city = readPicked(search['city']);
  const adda = readPicked(search['adda']);
  const collector = readPicked(search['collector']);
  // `all` is the same as leaving the kind out, so it is not carried.
  const voucherType = oneOf(VOUCHER_CHOICES, search['voucher_type']);
  const itemGroup = oneOf(ITEM_GROUP_CHOICES, search['item_group']);
  const tripView = oneOf(TRIP_VIEW_CHOICES, search['trip_view']);
  const garden = positiveId(search['garden']);
  const gardenView = oneOf(GARDEN_VIEW_CHOICES, search['garden_view']);
  const bikri = readBikri(search['bikri'], search['bikri_number']);
  const groupBy = oneOf(GROUP_BY_CHOICES, search['group_by']);
  const status = oneOf(PAID_CHOICES, search['status']);
  const detail = search['detail'] === true || search['detail'] === 'true';
  return {
    ...(period === undefined ? {} : { period }),
    ...(from === undefined ? {} : { from }),
    ...(to === undefined ? {} : { to }),
    ...(asOf === undefined ? {} : { as_of: asOf }),
    ...(Number.isSafeInteger(party) && party > 0 ? { party } : {}),
    ...(Number.isSafeInteger(partner) && partner > 0 ? { partner } : {}),
    ...(person === undefined ? {} : { person }),
    ...(guests ? { guests: true } : {}),
    ...(item === undefined ? {} : { item }),
    ...(seller === undefined ? {} : { seller }),
    ...(zamindar === undefined ? {} : { zamindar }),
    ...(city === undefined ? {} : { city }),
    ...(adda === undefined ? {} : { adda }),
    ...(collector === undefined ? {} : { collector }),
    ...(voucherType === undefined || voucherType === 'all' ? {} : { voucher_type: voucherType }),
    ...(itemGroup === undefined || itemGroup === 'all' ? {} : { item_group: itemGroup }),
    ...(tripView === undefined ? {} : { trip_view: tripView }),
    ...(garden === undefined ? {} : { garden }),
    ...(gardenView === undefined ? {} : { garden_view: gardenView }),
    ...(bikri === undefined ? {} : { bikri }),
    ...(groupBy === undefined ? {} : { group_by: groupBy }),
    ...(status === undefined || status === 'all' ? {} : { status }),
    ...(detail ? { detail: true } : {}),
  };
}

/** The filter boxes the address starts with. A party comes without its names; the page fills them. */
export function stateFromSearch(search: ReportSearch): ReportFilterState {
  return {
    ...EMPTY_FILTERS,
    period: search.period ?? 'custom',
    from: search.from ?? '',
    to: search.to ?? '',
    asOf: search.as_of ?? '',
    party:
      search.party === undefined
        ? null
        : { id: search.party, name_ur: '', name_en: '', code: undefined },
    person: search.person ?? null,
    partner: search.partner ?? null,
    guestsOnly: search.guests === true && search.partner === undefined,
    item: search.item ?? null,
    // Plan 49: each box the address names; the rest keep their starting value.
    seller: search.seller ?? null,
    zamindar: search.zamindar ?? null,
    city: search.city ?? null,
    adda: search.adda ?? null,
    collector: search.collector ?? null,
    voucherType: search.voucher_type ?? 'all',
    itemGroup: search.item_group ?? 'all',
    tripView: search.trip_view ?? 'trip',
    garden: search.garden ?? null,
    gardenView: search.garden_view ?? 'garden',
    bikri: search.bikri ?? null,
    groupBy: search.group_by ?? 'month',
    paid: search.status ?? 'all',
    detail: search.detail === true,
  };
}
