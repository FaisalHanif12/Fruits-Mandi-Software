// The layout box of the gardens report (R36, PRD 19.2): by garden (the default) or by partner. It
// changes the columns of the table, not the figures: by partner shows each partner's share of a
// garden, what he paid from the shop and from his pocket, and his money back.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import type { ReportGardenView } from '../../../bindings/ReportGardenView';
import { Field, Select } from '../../../shared/ui';
import { GARDEN_VIEW_CHOICES, gardenViewMessageId } from '../filters';

interface ReportGardenViewSelectProps {
  value: ReportGardenView;
  onChange: (value: ReportGardenView) => void;
  error?: ReactNode;
}

export function ReportGardenViewSelect({ value, onChange, error }: ReportGardenViewSelectProps) {
  const { l10n } = useLocalization();
  return (
    <Field label={<Localized id="report-filter-garden-view" />} error={error}>
      {(control) => (
        <Select<ReportGardenView>
          {...control}
          value={value}
          onChange={onChange}
          options={GARDEN_VIEW_CHOICES.map((choice) => ({
            value: choice,
            label: l10n.getString(gardenViewMessageId(choice)),
          }))}
        />
      )}
    </Field>
  );
}
