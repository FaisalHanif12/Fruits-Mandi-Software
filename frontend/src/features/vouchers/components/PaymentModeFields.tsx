// How the money moves, the same on every voucher (PRD 15.1): cash, a bank account, or a cheque on
// a bank account with its number and date. The bank fields show only when the mode needs them.
// Every voucher screen (پیشگی now, وصولی and ادائیگی later) uses this one piece (Code Standards F7).

import { Localized } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import {
  DateInput,
  Field,
  Picker,
  Select,
  TextInput,
  fieldError,
  type PickerValue,
} from '../../../shared/ui';
import type { VoucherMode } from '../types';

export interface PaymentModeValues {
  mode: VoucherMode;
  bank: PickerValue | null;
  cheque_no: string;
  cheque_date: string;
}

const MODES: readonly VoucherMode[] = ['cash', 'bank', 'cheque'];

interface PaymentModeFieldsProps {
  value: PaymentModeValues;
  onChange: (patch: Partial<PaymentModeValues>) => void;
  /** What the backend refused, to show next to the field it points at. */
  error: UiError | null;
}

export function PaymentModeFields({ value, onChange, error }: PaymentModeFieldsProps) {
  const needsBank = value.mode !== 'cash';
  const isCheque = value.mode === 'cheque';
  return (
    <>
      <Field label={<Localized id="vouchers-field-mode" />} error={fieldError(error, 'mode')}>
        {(props) => (
          <Select
            {...props}
            value={value.mode}
            onChange={(mode) => {
              onChange({ mode });
            }}
            options={MODES.map((mode) => ({
              value: mode,
              label: <Localized id={`vouchers-mode-${mode}`} />,
            }))}
          />
        )}
      </Field>
      {needsBank ? (
        <Field
          label={<Localized id="vouchers-field-bank" />}
          error={fieldError(error, 'bank_account_id')}
        >
          {(props) => (
            <Picker
              {...props}
              scope="bank_account"
              value={value.bank}
              onChange={(hit) => {
                onChange({
                  bank:
                    hit === null
                      ? null
                      : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
                });
              }}
            />
          )}
        </Field>
      ) : null}
      {isCheque ? (
        <>
          <Field
            label={<Localized id="vouchers-field-cheque-no" />}
            error={fieldError(error, 'cheque_no')}
          >
            {(props) => (
              <TextInput
                {...props}
                value={value.cheque_no}
                onChange={(cheque_no) => {
                  onChange({ cheque_no });
                }}
                lang="en"
              />
            )}
          </Field>
          <Field
            label={<Localized id="vouchers-field-cheque-date" />}
            error={fieldError(error, 'cheque_date')}
          >
            {(props) => (
              <DateInput
                {...props}
                value={value.cheque_date}
                onChange={(cheque_date) => {
                  onChange({ cheque_date });
                }}
              />
            )}
          </Field>
        </>
      ) : null}
    </>
  );
}
