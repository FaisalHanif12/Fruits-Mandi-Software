// The address a button under an answer opens (PRD FR-7.74): the report or view the model read, with
// the same filters already in its boxes. The filters are the backend's own, as the tool read them
// (`SourceReportFilters`, `SourceAnalyticsFilters`, with the names of every record in them); this file
// only turns them into the address each screen reads (`ReportSearch`, `AnalyticsSearch`) and into
// the filters a print asks for. Nothing is worked out. The backend writes a date as `YYYY-MM-DD`; the
// boxes of every screen show `DD-MM-YYYY`, so a date is shown that way here (the backend reads both).

import type { AnalyticsFilters } from '../../bindings/AnalyticsFilters';
import type { AnswerRecord } from '../../bindings/AnswerRecord';
import type { ReportFilters } from '../../bindings/ReportFilters';
import type { SourceAnalyticsFilters } from '../../bindings/SourceAnalyticsFilters';
import type { SourceParty } from '../../bindings/SourceParty';
import type { SourceReportFilters } from '../../bindings/SourceReportFilters';
import type { AnalyticsSearch } from '../analytics';
import type { ReportSearch } from '../reports';
import { isoToDisplayDate } from '../../shared/format';

/** A record as a picker holds it, with its names, so the box shows who it is. */
function picked(record: AnswerRecord) {
  return {
    id: record.id,
    name_ur: record.name_ur,
    name_en: record.name_en,
    tag: record.tag,
    code: record.code ?? undefined,
  };
}

/**
 * The party an entry screen opens with (PRD FR-7.79): the whole record, so the field shows who he is at
 * once. The screens read it with `partyOfAddress` and leave every amount empty.
 */
export function entryPartyOf(record: AnswerRecord) {
  return picked(record);
}

/** The party of a report: an account, or one person inside a one-time account (PRD FR-13.5). */
function partyOf(party: SourceParty | null): Pick<ReportSearch, 'party' | 'person'> {
  if (party === null) return {};
  if (party.person_key === null) return { party: party.record.id };
  return {
    person: {
      accountId: party.record.id,
      personKey: party.person_key,
      name_ur: party.record.name_ur,
      name_en: party.record.name_en,
      phone: '',
      last_date: '',
    },
  };
}

/** The address of a report for the filters its tool was read with. */
export function reportSearchFromFilters(filters: SourceReportFilters): ReportSearch {
  return {
    // A date shortcut wins over the two dates, as it does in the filter boxes.
    ...(filters.shortcut === null ? {} : { period: filters.shortcut }),
    ...(filters.shortcut === null && filters.from !== ''
      ? { from: isoToDisplayDate(filters.from) }
      : {}),
    ...(filters.shortcut === null && filters.to !== '' ? { to: isoToDisplayDate(filters.to) } : {}),
    ...(filters.as_of === '' ? {} : { as_of: isoToDisplayDate(filters.as_of) }),
    ...partyOf(filters.party),
    ...(filters.seller === null ? {} : { seller: picked(filters.seller) }),
    ...(filters.zamindar === null ? {} : { zamindar: picked(filters.zamindar) }),
    ...(filters.item === null ? {} : { item: picked(filters.item) }),
    ...(filters.city === null ? {} : { city: picked(filters.city) }),
    ...(filters.adda === null ? {} : { adda: picked(filters.adda) }),
    ...(filters.collector == null ? {} : { collector: picked(filters.collector) }),
    ...(filters.partner === null ? {} : { partner: filters.partner.id }),
    ...(filters.garden === null ? {} : { garden: filters.garden.id }),
    ...(filters.bikri === null ? {} : { bikri: filters.bikri }),
    ...(filters.detail ? { detail: true } : {}),
    ...(filters.group_by === null ? {} : { group_by: filters.group_by }),
    ...(filters.status === null ? {} : { status: filters.status }),
    ...(filters.voucher_type === null ? {} : { voucher_type: filters.voucher_type }),
    ...(filters.trip_view === null ? {} : { trip_view: filters.trip_view }),
    ...(filters.stock_view == null ? {} : { stock_view: filters.stock_view }),
    ...(filters.garden_view === null ? {} : { garden_view: filters.garden_view }),
    ...(filters.guests_only ? { guests: true } : {}),
    ...(filters.item_group == null ? {} : { item_group: filters.item_group }),
    ...(filters.grade == null ? {} : { grade: filters.grade }),
  };
}

/** The address of an analytics view for the filters its tool was read with. */
export function analyticsSearchFromFilters(filters: SourceAnalyticsFilters): AnalyticsSearch {
  return {
    ...(filters.period === null ? {} : { period: filters.period }),
    ...(filters.period === null && filters.from !== ''
      ? { from: isoToDisplayDate(filters.from) }
      : {}),
    ...(filters.period === null && filters.to !== '' ? { to: isoToDisplayDate(filters.to) } : {}),
    ...(filters.group_by === null ? {} : { group_by: filters.group_by }),
    ...(filters.item === null ? {} : { item: picked(filters.item) }),
    ...(filters.seller === null ? {} : { seller: picked(filters.seller) }),
    ...(filters.buyer === null ? {} : { buyer: picked(filters.buyer.record) }),
    ...(filters.city === null ? {} : { city: picked(filters.city) }),
    ...(filters.adda === null ? {} : { adda: picked(filters.adda) }),
    ...(filters.side === 'both' ? {} : { side: filters.side }),
    ...(filters.compare === 'off' ? {} : { compare: filters.compare }),
    ...(filters.goods === 'all' ? {} : { goods: filters.goods }),
    ...(filters.trip_view === null ? {} : { trip_view: filters.trip_view }),
    ...(filters.partner === null ? {} : { partner: filters.partner.id }),
    ...(filters.trip === null ? {} : { trip: filters.trip.id }),
    ...(filters.garden === null ? {} : { garden: filters.garden.id }),
    ...(filters.item_group == null ? {} : { item_group: filters.item_group }),
    ...(filters.grade == null ? {} : { grade: filters.grade }),
  };
}

/** The filters a report print asks for: the ids of the records, in the shape the backend reads. */
export function reportFiltersOf(filters: SourceReportFilters): ReportFilters {
  return {
    shortcut: filters.shortcut,
    from: isoToDisplayDate(filters.from),
    to: isoToDisplayDate(filters.to),
    as_of: isoToDisplayDate(filters.as_of),
    party:
      filters.party === null
        ? null
        : { account_id: filters.party.record.id, person_key: filters.party.person_key },
    seller_id: filters.seller?.id ?? null,
    zamindar_id: filters.zamindar?.id ?? null,
    item_id: filters.item?.id ?? null,
    city_id: filters.city?.id ?? null,
    adda_id: filters.adda?.id ?? null,
    detail: filters.detail,
    partner_id: filters.partner?.id ?? null,
    garden_id: filters.garden?.id ?? null,
    bikri_id: filters.bikri?.id ?? null,
    // The keys the backend reads as "missing = the default" are left out when the source had none.
    ...(filters.group_by === null ? {} : { group_by: filters.group_by }),
    ...(filters.status === null ? {} : { status: filters.status }),
    ...(filters.voucher_type === null ? {} : { voucher_type: filters.voucher_type }),
    ...(filters.trip_view === null ? {} : { trip_view: filters.trip_view }),
    ...(filters.stock_view == null ? {} : { stock_view: filters.stock_view }),
    ...(filters.garden_view === null ? {} : { garden_view: filters.garden_view }),
    ...(filters.guests_only ? { guests_only: true } : {}),
    ...(filters.collector == null ? {} : { collector_id: filters.collector.id }),
    ...(filters.item_group == null ? {} : { item_group: filters.item_group }),
    ...(filters.grade == null ? {} : { grade: filters.grade }),
  };
}

/** The filters an analytics print asks for. */
export function analyticsFiltersOf(filters: SourceAnalyticsFilters): AnalyticsFilters {
  return {
    period: filters.period,
    from: isoToDisplayDate(filters.from),
    to: isoToDisplayDate(filters.to),
    group_by: filters.group_by,
    item_id: filters.item?.id ?? null,
    seller_id: filters.seller?.id ?? null,
    buyer:
      filters.buyer === null
        ? null
        : { account_id: filters.buyer.record.id, person_key: filters.buyer.person_key },
    city_id: filters.city?.id ?? null,
    adda_id: filters.adda?.id ?? null,
    side: filters.side,
    compare: filters.compare,
    partner_id: filters.partner?.id ?? null,
    trip_id: filters.trip?.id ?? null,
    garden_id: filters.garden?.id ?? null,
    goods: filters.goods,
    trip_view: filters.trip_view,
    ...(filters.item_group == null ? {} : { item_group: filters.item_group }),
    ...(filters.grade == null ? {} : { grade: filters.grade }),
  };
}
