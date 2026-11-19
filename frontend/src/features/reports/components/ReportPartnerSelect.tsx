// The partner box of the filters (PRD FR-17.9, R29, R31). The partners are a short list (up to ten,
// NFR-SCL-5), so the box is the shared drop-down, not a picker. The list is the partners module's own
// (`usePartnerList`); names go through `displayName` (Code Standards F38). R31 cannot run without a
// partner, so its box has no "all" choice; R29 takes one partner or all of them. The fruit register (R37)
// is for a partner or for the shop's guests (PRD 16.7), so its box also offers "guests" and is
// labelled "For": the choice is a partner OR the guests, never both.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Field, Select } from '../../../shared/ui';
import { usePartnerList } from '../../partners';

/** The value of the "all partners" choice. A partner's own value is his id as text. */
const ALL = 'all';
/** The value of the "guests" choice of the fruit register. */
const GUESTS = 'guests';

interface ReportPartnerSelectProps {
  /** The partner chosen, or null for all (R29) or none yet (R31). */
  partner: number | null;
  /** False for R29: the box then has an "all partners" choice. */
  required: boolean;
  /** True for the fruit register (R37): the box also offers the shop's guests. */
  guests?: boolean;
  /** True when the guests are chosen (only when `guests` is on). */
  guestsOnly?: boolean;
  /** The choice made: a partner or null, and whether it is the guests (then the partner is null). */
  onChange: (partner: number | null, guestsOnly: boolean) => void;
  /** The backend's refusal for this box, shown under it. */
  error?: ReactNode;
}

export function ReportPartnerSelect({
  partner,
  required,
  guests = false,
  guestsOnly = false,
  onChange,
  error,
}: ReportPartnerSelectProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const partners = usePartnerList();
  const options = [
    ...(required ? [] : [{ value: ALL, label: l10n.getString('report-partner-all') }]),
    ...(guests ? [{ value: GUESTS, label: l10n.getString('fruit-for-guests') }] : []),
    ...(partners.data ?? []).map((row) => ({
      value: String(row.id),
      label: displayName(row, language),
    })),
  ];
  let value: string | undefined;
  if (guests && guestsOnly) value = GUESTS;
  else if (partner !== null) value = String(partner);
  else if (!required) value = ALL;
  return (
    <Field
      label={<Localized id={guests ? 'fruit-filter-for' : 'report-filter-partner'} />}
      // A list that could not be read is said in words under the box, and the box stays shut.
      error={partners.isError ? <Localized id="cashbook-partners-failed" /> : error}
    >
      {(control) => (
        <Select
          {...control}
          readOnly={control.readOnly === true || partners.isPending || partners.isError}
          value={value}
          onChange={(next) => {
            if (next === GUESTS) onChange(null, true);
            else onChange(next === ALL ? null : Number(next), false);
          }}
          options={options}
        />
      )}
    </Field>
  );
}
