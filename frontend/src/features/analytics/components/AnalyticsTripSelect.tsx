// The trip box of the analytics filters (PRD 18.1, FR-12.27): one trip, or all of them. The trips are
// a short list (a day trip now and then), so the box is the shared drop-down, not a picker. The list
// is the trips module's own (`useTripsList`); its pages are loaded one after the other until it is
// whole, because a drop-down has to hold every choice. A trip is named by `displayName` (Code
// Standards F38): its name is made from its number and its day.

import { Localized, useLocalization } from '@fluent/react';
import { useEffect, type ReactNode } from 'react';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Field, Select } from '../../../shared/ui';
import { useTripsList } from '../../trips';

/** The value of the "all trips" choice. A trip's own value is its id as text. */
const ALL = 'all';

/** Every trip: no day, no state, no partner and no text limit the list. */
const EVERY_TRIP = { from: null, to: null, status: null, partner_id: null, search: null } as const;

interface AnalyticsTripSelectProps {
  /** The trip chosen, or null for all of them. */
  trip: number | null;
  onChange: (trip: number | null) => void;
  /** The backend's refusal for this box, shown under it. */
  error?: ReactNode;
}

export function AnalyticsTripSelect({ trip, onChange, error }: AnalyticsTripSelectProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const trips = useTripsList(EVERY_TRIP);
  const { hasNextPage, isFetchingNextPage, fetchNextPage } = trips;
  // The list loads a page at a time; a drop-down wants all of it, so the next page is asked for
  // as soon as the last one has come.
  useEffect(() => {
    if (hasNextPage && !isFetchingNextPage) void fetchNextPage();
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);
  const rows = trips.data?.pages.flatMap((page) => page.rows) ?? [];
  let value = ALL;
  if (trip !== null) value = String(trip);
  const options = [
    { value: ALL, label: l10n.getString('analytics-trip-all') },
    ...rows.map((row) => ({ value: String(row.id), label: displayName(row, language) })),
  ];
  return (
    <Field
      label={<Localized id="analytics-filter-trip" />}
      // A list that could not be read is said in words under the box, and the box stays shut.
      error={trips.isError ? <Localized id="analytics-trip-failed" /> : error}
    >
      {(control) => (
        <Select
          {...control}
          readOnly={control.readOnly === true || trips.isPending || trips.isError}
          value={value}
          onChange={(next) => {
            onChange(next === ALL ? null : Number(next));
          }}
          options={options}
        />
      )}
    </Field>
  );
}
