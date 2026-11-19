// The layout box of the trip profit or loss (R33, PRD 19.2): by trip (the default), item, day, month
// or partner. It changes the columns of the table, not the figures: every layout adds up to the
// same result (the partner layout shows each partner's part so far and the cost still left).

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import type { ReportTripView } from '../../../bindings/ReportTripView';
import { Field, Select } from '../../../shared/ui';
import { TRIP_VIEW_CHOICES, tripViewMessageId } from '../filters';

interface ReportTripViewSelectProps {
  value: ReportTripView;
  onChange: (value: ReportTripView) => void;
  error?: ReactNode;
}

export function ReportTripViewSelect({ value, onChange, error }: ReportTripViewSelectProps) {
  const { l10n } = useLocalization();
  return (
    <Field label={<Localized id="report-filter-trip-view" />} error={error}>
      {(control) => (
        <Select<ReportTripView>
          {...control}
          value={value}
          onChange={onChange}
          options={TRIP_VIEW_CHOICES.map((choice) => ({
            value: choice,
            label: l10n.getString(tripViewMessageId(choice)),
          }))}
        />
      )}
    </Field>
  );
}
