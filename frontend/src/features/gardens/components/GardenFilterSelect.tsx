// The garden box of a filter bar (PRD 18.1, PRD 19.2 R36): one garden, or all of them. The gardens are
// a short list (a few each season), so the box is the shared drop-down, not a picker. The list is
// the gardens module's own (`useGardensList`); its pages are loaded one after the other until it is
// whole, because a drop-down has to hold every choice. A garden is named by `displayName` (Code
// Standards F38). The report page and the analytics page both use it, with their own label.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import { useLoadRest } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Field, Select } from '../../../shared/ui';
import { useGardensList } from '../hooks';

/** The value of the "all gardens" choice. A garden's own value is its id as text. */
const ALL = 'all';

/** Every garden: no day, no state, no partner and no text limit the list. */
const EVERY_GARDEN = {
  from: null,
  to: null,
  status: null,
  partner_id: null,
  search: null,
} as const;

interface GardenFilterSelectProps {
  /** The garden chosen, or null for all of them. */
  garden: number | null;
  onChange: (garden: number | null) => void;
  /** The Fluent message of the label: each filter bar has its own word for the box. */
  labelId: string;
  /** The backend's refusal for this box, shown under it. */
  error?: ReactNode;
}

export function GardenFilterSelect({ garden, onChange, labelId, error }: GardenFilterSelectProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const gardens = useGardensList(EVERY_GARDEN);
  useLoadRest(gardens, true);
  const rows = gardens.data?.pages.flatMap((page) => page.rows) ?? [];
  let value = ALL;
  if (garden !== null) value = String(garden);
  const options = [
    { value: ALL, label: l10n.getString('gardens-filter-all') },
    // The code of the garden (type 008, plan 34) and its name, so two gardens are easy to tell apart.
    ...rows.map((row) => ({
      value: String(row.id),
      label: `${row.code} ${displayName(row, language)}`,
    })),
  ];
  return (
    <Field
      label={<Localized id={labelId} />}
      // A list that could not be read is said in words under the box, and the box stays shut.
      error={gardens.isError ? <Localized id="gardens-filter-failed" /> : error}
    >
      {(control) => (
        <Select
          {...control}
          readOnly={control.readOnly === true || gardens.isPending || gardens.isError}
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
