// The partner box of the filters (PRD FR-17.9, R29, R31). The partners are a short list (up to ten,
// NFR-SCL-5), so the box is the shared drop-down, not a picker. The list is the partners module's own
// (`usePartnerList`); names go through `displayName` (Code Standards F38). R31 cannot run without a
// partner, so its box has no "all" choice; R29 takes one partner or all of them.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Field, Select } from '../../../shared/ui';
import { usePartnerList } from '../../partners';

/** The value of the "all partners" choice. A partner's own value is his id as text. */
const ALL = 'all';

interface ReportPartnerSelectProps {
  /** The partner chosen, or null for all (R29) or none yet (R31). */
  partner: number | null;
  /** False for R29: the box then has an "all partners" choice. */
  required: boolean;
  onChange: (partner: number | null) => void;
  /** The backend's refusal for this box, shown under it. */
  error?: ReactNode;
}

export function ReportPartnerSelect({
  partner,
  required,
  onChange,
  error,
}: ReportPartnerSelectProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const partners = usePartnerList();
  const options = [
    ...(required ? [] : [{ value: ALL, label: l10n.getString('report-partner-all') }]),
    ...(partners.data ?? []).map((row) => ({
      value: String(row.id),
      label: displayName(row, language),
    })),
  ];
  let value: string | undefined;
  if (partner !== null) value = String(partner);
  else if (!required) value = ALL;
  return (
    <Field
      label={<Localized id="report-filter-partner" />}
      // A list that could not be read is said in words under the box, and the box stays shut.
      error={partners.isError ? <Localized id="cashbook-partners-failed" /> : error}
    >
      {(control) => (
        <Select
          {...control}
          readOnly={control.readOnly === true || partners.isPending || partners.isError}
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
