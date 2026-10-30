// The address of the hidden print window for the collection list (plan 17e, PRD FR-15.6):
// `#/print?collection=<YYYY-MM-DD>&doc=collection_list&language=<ur|en>[&collector=<id>][&city=<id>]
// &job=<job>`. The window is opened by the backend, so every value is read as plain text and checked
// here. A value that is not right is a failed print, never a guess. The day is `YYYY-MM-DD` in the
// address and `DD-MM-YYYY` in the command (PRD FR-7.19), so the same day is sent. The job name is
// checked by the same function as every other print window (one rule for the window).

import type { CollectionPrintDataInput } from '../../../bindings/CollectionPrintDataInput';
import type { Language } from '../../../bindings/Language';
import { isoToDisplayDate } from '../../../shared/format';
import { jobOf, positiveInt } from '../../../shared/print';

export interface CollectionPrintParams {
  /** The job name; the events are `print-ready-<job>` and `print-error-<job>`. */
  job: string;
  input: CollectionPrintDataInput;
}

const LANGUAGE_VALUES: readonly Language[] = ['ur', 'en'];
const ISO_DAY = /^(\d{4})-(\d{2})-(\d{2})$/;

/** True when the address is for the collection list (it names a day), even if the rest is wrong. */
export function isCollectionAddress(searchStr: string): boolean {
  return new URLSearchParams(searchStr).has('collection');
}

/** The day of the address as a real date, or null: `2026-02-30` is not one. */
function realDay(text: string | null): string | null {
  const parts = text === null ? null : ISO_DAY.exec(text);
  if (parts === null) return null;
  const [, year, month, day] = parts.map(Number);
  if (year === undefined || month === undefined || day === undefined) return null;
  const made = new Date(Date.UTC(year, month - 1, day));
  const same =
    made.getUTCFullYear() === year && made.getUTCMonth() === month - 1 && made.getUTCDate() === day;
  return same && text !== null ? text : null;
}

/** An optional filter id: missing is null, a value that is not a positive whole number is wrong. */
function optionalId(text: string | null): number | null | undefined {
  if (text === null) return null;
  return positiveInt(text) ?? undefined;
}

/** The parameters of the print window for the collection list, or null when the address is not valid. */
export function parseCollectionPrintParams(searchStr: string): CollectionPrintParams | null {
  const search = new URLSearchParams(searchStr);
  const job = jobOf(searchStr);
  const day = realDay(search.get('collection'));
  const language = LANGUAGE_VALUES.find((candidate) => candidate === search.get('language'));
  const collector = optionalId(search.get('collector'));
  const city = optionalId(search.get('city'));
  if (job === null) return null;
  if (
    day === null ||
    search.get('doc') !== 'collection_list' ||
    language === undefined ||
    collector === undefined ||
    city === undefined
  ) {
    return null;
  }
  return {
    job,
    input: {
      date: isoToDisplayDate(day),
      collector_staff_id: collector,
      city_id: city,
      search: search.get('search'),
      language,
    },
  };
}
