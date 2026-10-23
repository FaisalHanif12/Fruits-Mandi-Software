// What the address of a report can carry (PRD FR-7.65): `#/reports/R13?party=20&from=01-10-2026`
// opens the ledger of account 20 with the dates already filled, the way the party card's buttons
// do. Everything is optional, and anything that does not read is left out.

import type { PeriodChoice, ReportFilterState, ReportPerson } from './filters';
import { EMPTY_FILTERS, PERIOD_CHOICES } from './filters';

export interface ReportSearch {
  period?: PeriodChoice;
  from?: string;
  to?: string;
  as_of?: string;
  party?: number;
  /** A one-time person (PRD FR-13.5): the global search opens his register this way. */
  person?: ReportPerson;
}

function text(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() !== '' ? value.trim() : undefined;
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
  const person = readPerson(search['person']);
  return {
    ...(period === undefined ? {} : { period }),
    ...(from === undefined ? {} : { from }),
    ...(to === undefined ? {} : { to }),
    ...(asOf === undefined ? {} : { as_of: asOf }),
    ...(Number.isSafeInteger(party) && party > 0 ? { party } : {}),
    ...(person === undefined ? {} : { person }),
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
  };
}
