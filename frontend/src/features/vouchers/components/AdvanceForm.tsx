// The fields of one پیشگی voucher (PRD 15.1, 15.4): date, seller with his party card and advances,
// amount, how the money moves, expected goods or season, narration, and the balance before and
// after from the backend. Everything is text in the screen's state, so a language switch loses
// nothing (F39). The screen owns the keys: F10 saves, F2 adds this voucher to the list.

import { Localized } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import {
  AccountPicker,
  Balance,
  Card,
  DateInput,
  Field,
  fieldError,
  FreeTextInput,
  MoneyInput,
} from '../../../shared/ui';
import type { AdvancePreviewRow, AdvanceDraft } from '../types';
import { AdvanceHistory } from './AdvanceHistory';
import { PaymentModeFields } from './PaymentModeFields';

interface AdvanceFormProps {
  draft: AdvanceDraft;
  onChange: (patch: Partial<AdvanceDraft>) => void;
  /** What the backend refused when this voucher was saved, with its field paths. */
  error: UiError | null;
  /** The backend's view of this voucher: the balance before and after. Null until it has one. */
  preview: AdvancePreviewRow | null;
}

export function AdvanceForm({ draft, onChange, error, preview }: AdvanceFormProps) {
  return (
    <Card>
      <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
        <Field label={<Localized id="vouchers-field-date" />} error={fieldError(error, 'date')}>
          {(props) => (
            <DateInput
              {...props}
              value={draft.date}
              onChange={(date) => {
                onChange({ date });
              }}
            />
          )}
        </Field>
        <Field
          label={<Localized id="vouchers-field-seller" />}
          error={fieldError(error, 'seller_account_id')}
        >
          {(props) => (
            <AccountPicker
              {...props}
              scope="seller"
              value={draft.seller}
              onChange={(hit) => {
                onChange({
                  seller:
                    hit === null
                      ? null
                      : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
                });
              }}
            />
          )}
        </Field>
      </div>
      <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
        <Field label={<Localized id="vouchers-field-amount" />} error={fieldError(error, 'amount')}>
          {(props) => (
            <MoneyInput
              {...props}
              value={draft.amount}
              onChange={(amount) => {
                onChange({ amount });
              }}
            />
          )}
        </Field>
        <PaymentModeFields
          value={draft}
          error={error}
          onChange={(patch) => {
            onChange(patch);
          }}
        />
        <Field
          label={<Localized id="vouchers-field-expected" />}
          hint={<Localized id="vouchers-field-expected-hint" />}
          error={fieldError(error, 'expected_goods')}
        >
          {(props) => (
            <FreeTextInput
              {...props}
              value={draft.expected_goods}
              onChange={(expected_goods) => {
                onChange({ expected_goods });
              }}
            />
          )}
        </Field>
        <Field
          label={<Localized id="vouchers-field-narration" />}
          error={fieldError(error, 'narration')}
        >
          {(props) => (
            <FreeTextInput
              {...props}
              value={draft.narration}
              onChange={(narration) => {
                onChange({ narration });
              }}
            />
          )}
        </Field>
      </div>
      {preview === null ? null : (
        <p className="flex flex-wrap items-baseline gap-x-6 gap-y-1 type-body">
          <span className="flex items-baseline gap-2">
            <Localized id="vouchers-current-balance" />
            <Balance balance={preview.current_balance} />
          </span>
          <span className="flex items-baseline gap-2">
            <Localized id="vouchers-new-balance" />
            <Balance balance={preview.new_balance} />
          </span>
        </p>
      )}
      {draft.seller === null ? null : <AdvanceHistory sellerId={draft.seller.id} />}
    </Card>
  );
}
