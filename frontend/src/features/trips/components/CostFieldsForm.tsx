// The fields only a trip cost has (PRD FR-12.31): the amount of a cost of the whole trip (the
// truck, the food) and why. It is paid in by the partners like a purchase (the sources are on
// the trip screen) and spread over the purchases that still have goods left, which the preview
// shows. The amount may be a simple sum such as `3000+1000` (PRD FR-7.48); the backend reads it.

import { Localized } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { Field, FreeTextInput, MoneyInput, fieldError } from '../../../shared/ui';
import type { CostFields } from '../types';

interface CostFieldsFormProps {
  fields: CostFields;
  amount: string;
  onChange: (patch: { reason?: string; amount?: string }) => void;
  error: UiError | null;
}

export function CostFieldsForm({ fields, amount, onChange, error }: CostFieldsFormProps) {
  return (
    <>
      <Field
        label={<Localized id="trips-cost-amount" />}
        hint={<Localized id="trips-cost-amount-hint" />}
        error={fieldError(error, 'amount')}
      >
        {(props) => (
          <MoneyInput
            {...props}
            value={amount}
            onChange={(next) => {
              onChange({ amount: next });
            }}
          />
        )}
      </Field>
      <div className="col-span-2">
        <Field
          label={<Localized id="trips-cost-reason" />}
          hint={<Localized id="trips-cost-reason-hint" />}
          error={fieldError(error, 'reason')}
        >
          {(props) => (
            <FreeTextInput
              {...props}
              value={fields.reason}
              onChange={(reason) => {
                onChange({ reason });
              }}
            />
          )}
        </Field>
      </div>
    </>
  );
}
