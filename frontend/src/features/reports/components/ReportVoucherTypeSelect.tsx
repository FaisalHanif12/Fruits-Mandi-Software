// The voucher kind box of the vouchers list (R15, PRD 19.2, FR-19.10): every kind, or one. The
// kinds are the ones of the vouchers screen, each by its own word (`report-doc-<kind>`), so the same
// word is used here, on the voucher and on the print. Picking "all kinds" sends no filter.

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import { Field, Select } from '../../../shared/ui';
import { VOUCHER_CHOICES, voucherMessageId, type VoucherChoice } from '../filters';

interface ReportVoucherTypeSelectProps {
  value: VoucherChoice;
  onChange: (value: VoucherChoice) => void;
  /** The backend's refusal for this box, shown under it. */
  error?: ReactNode;
}

export function ReportVoucherTypeSelect({ value, onChange, error }: ReportVoucherTypeSelectProps) {
  const { l10n } = useLocalization();
  return (
    <Field label={<Localized id="report-filter-voucher-type" />} error={error}>
      {(control) => (
        <Select<VoucherChoice>
          {...control}
          value={value}
          onChange={onChange}
          options={VOUCHER_CHOICES.map((choice) => ({
            value: choice,
            label: l10n.getString(
              choice === 'all' ? 'report-voucher-type-all' : voucherMessageId(choice),
            ),
          }))}
        />
      )}
    </Field>
  );
}
