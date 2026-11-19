// The amber band over an entry that is open for edit (plan 44, PRD FR-7.70): "You are changing
// Sale 5 (10-10-26)". It tells at once that this is not a new entry, that the day does not change,
// and, when the days were reopened for the fix, which days close again after the save (FR-21.4).
// Every entry screen uses this one band.

import { Localized, useLocalization } from '@fluent/react';
import { daysText } from '../edit/useFixDay';
import { isoToDisplayDate } from '../format';
import { Notice } from './Notice';

/** The kinds of entry that have a band, each with the glossary word that names it. */
const WHAT_ID = {
  bikri: 'glossary-bikri',
  receipt: 'glossary-wasooli',
  payment: 'glossary-adaigi',
  advance: 'glossary-peshgi-khata',
  fruit: 'glossary-phal',
  expense: 'glossary-kharcha',
  lent: 'glossary-naqd-udhar',
  khata: 'glossary-khata',
  capital: 'glossary-sarmaya',
  journal: 'glossary-general',
  garden: 'glossary-bagh',
  trip: 'glossary-safar',
  purchase: 'glossary-khareedari',
} as const;

export type EditBandKind = keyof typeof WHAT_ID;

interface EditBandProps {
  /** The kind of entry; the band names it with its glossary word. */
  kind: EditBandKind;
  /** The number of the entry as shown ("5", "و-3"). */
  number: string;
  /** The day of the entry, `YYYY-MM-DD`. */
  date: string;
  /** The days that were reopened for this fix, `YYYY-MM-DD`. Empty when no day was reopened. */
  reopenedDays?: readonly string[];
}

export function EditBand({ kind, number, date, reopenedDays = [] }: EditBandProps) {
  const { l10n } = useLocalization();
  return (
    <Notice tone="warn">
      <span className="flex flex-col gap-1">
        <Localized
          id="edit-band"
          vars={{ what: l10n.getString(WHAT_ID[kind]), number, date: isoToDisplayDate(date) }}
        >
          <span className="font-semibold" />
        </Localized>
        <Localized id="edit-band-date-fixed">
          <span />
        </Localized>
        {reopenedDays.length === 0 ? null : (
          <Localized id="edit-band-days" vars={{ days: daysText(reopenedDays, l10n) }}>
            <span />
          </Localized>
        )}
      </span>
    </Notice>
  );
}
