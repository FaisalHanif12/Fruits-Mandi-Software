// The period box of the commission page and the kharch page (PRD FR-9.3, FR-9.5, plan 77): a choice of one date, a
// month, a year or a range of dates, and the two dates of a range. A date of the range is taken when the box is left
// (shortcuts of FR-7.19); the page never sends half a date. Changing the kind carries what is on screen with it (the
// year of the month shown, the month of the date shown), so the Owner never starts again from the latest one.
// Nothing here is money and nothing is worked out: the page keeps the period in its address and the backend reads
// the days.

import { Localized, useLocalization } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { monthBounds, monthOf } from '../../../shared/format';
import { Field, Select, fieldError } from '../../../shared/ui';
import {
  COMMISSION_KINDS,
  yearOf,
  type CommissionKind,
  type PeriodFilters,
} from '../commission-search';
import { CommissionDateBox } from './CommissionNav';

/** The Fluent message of a period choice. */
const KIND_ID: Readonly<Record<CommissionKind, string>> = {
  day: 'period-day',
  month: 'period-month',
  year: 'period-year',
  range: 'period-range',
};

/**
 * What the page keeps when the period changes kind: the place the Owner was looking at, so he does not start
 * from the latest one again. A date becomes the year or the month that holds it; a month or a year opens the
 * range or the day it begins with. Null is "the latest" of each kind.
 */
export function carryToKind(
  kind: CommissionKind,
  now: PeriodFilters,
  today: string | null,
): Partial<PeriodFilters> {
  // The place now: the day, month, year or start of the range, as the longest text it has.
  const anchor =
    now.kind === 'day'
      ? now.day
      : now.kind === 'month'
        ? now.month
        : now.kind === 'year'
          ? now.year
          : now.from;
  const clear = { kind, month: null, day: null, year: null, from: null, to: null };
  if (anchor === null) return clear;
  switch (kind) {
    case 'year':
      return { ...clear, year: anchor.slice(0, 4) };
    case 'month':
      return { ...clear, month: anchor.length >= 7 ? anchor.slice(0, 7) : null };
    case 'day':
      return { ...clear, day: anchor.length === 10 ? anchor : null };
    case 'range': {
      // A whole year or month: from its first day to its last. The one that is not over ends today, which
      // is the latest (null).
      if (anchor.length === 4) {
        return {
          ...clear,
          from: `${anchor}-01-01`,
          to: today !== null && yearOf(today) === anchor ? null : `${anchor}-12-31`,
        };
      }
      if (anchor.length < 7) return clear;
      const month = anchor.slice(0, 7);
      const bounds = monthBounds(month);
      return {
        ...clear,
        from: bounds.from,
        to: today !== null && monthOf(today) === month ? null : bounds.to,
      };
    }
  }
}

interface PeriodFilterBoxProps {
  filters: PeriodFilters;
  onChange: (change: Partial<PeriodFilters>) => void;
  /** The backend's today, `YYYY-MM-DD`, or null until it is known. */
  today: string | null;
  /** The backend's refusal of the last read, so a wrong date shows under its box. */
  error: UiError | null;
}

/** The boxes of the period, to be placed in a filter card (a fragment: the card is the page's). */
export function PeriodFilterBox({ filters, onChange, today, error }: PeriodFilterBoxProps) {
  const { l10n } = useLocalization();
  return (
    <>
      <Field label={<Localized id="period-label" />} error={fieldError(error, 'period.kind')}>
        {(control) => (
          <Select<CommissionKind>
            {...control}
            value={filters.kind}
            onChange={(kind) => {
              onChange(carryToKind(kind, filters, today));
            }}
            options={COMMISSION_KINDS.map((kind) => ({
              value: kind,
              label: l10n.getString(KIND_ID[kind]),
            }))}
          />
        )}
      </Field>
      {filters.kind === 'range' ? (
        <>
          <CommissionDateBox
            key={`from-${filters.from ?? ''}`}
            label={<Localized id="common-period-from" />}
            value={filters.from}
            today={today}
            refusal={fieldError(error, 'period.from')}
            onCommit={(from) => {
              onChange({ from });
            }}
          />
          <CommissionDateBox
            key={`to-${filters.to ?? ''}`}
            label={<Localized id="common-period-to" />}
            value={filters.to}
            today={today}
            refusal={fieldError(error, 'period.to')}
            onCommit={(to) => {
              onChange({ to });
            }}
          />
        </>
      ) : null}
    </>
  );
}
