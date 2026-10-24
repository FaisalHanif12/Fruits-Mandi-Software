// The fields of the garden's entry form, by type (PRD FR-7.67, FR-12.35, FR-12.36). An installment
// and a running cost have the same two fields: the amount (it may be a simple sum such as
// `20000+5000`, PRD FR-7.48; the backend reads it) and why. A sale in another shop has the shop as
// typed, how many نگ were sold, the money received and a note. The day, and the sources of the
// partners, are on the screen for every type in the same places.

import { Localized } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { Field, FreeTextInput, MoneyInput, QtyInput, fieldError } from '../../../shared/ui';
import type { EntryDraft } from '../types';

interface GardenEntryFieldsFormProps {
  draft: EntryDraft;
  onChange: (patch: Partial<EntryDraft>) => void;
  error: UiError | null;
}

export function GardenEntryFieldsForm({ draft, onChange, error }: GardenEntryFieldsFormProps) {
  if (draft.type === 'other_sale') {
    return (
      <>
        <div className="col-span-2">
          <Field label={<Localized id="gardens-sale-shop" />} error={fieldError(error, 'shop')}>
            {(props) => (
              <FreeTextInput
                {...props}
                value={draft.shop}
                onChange={(shop) => {
                  onChange({ shop });
                }}
              />
            )}
          </Field>
        </div>
        <Field
          label={<Localized id="gardens-sale-qty" />}
          hint={<Localized id="gardens-sale-qty-hint" />}
          error={fieldError(error, 'qty')}
        >
          {(props) => (
            <QtyInput
              {...props}
              value={draft.qty}
              onChange={(qty) => {
                onChange({ qty });
              }}
            />
          )}
        </Field>
        <Field
          label={<Localized id="gardens-sale-amount" />}
          hint={<Localized id="gardens-sale-amount-hint" />}
          error={fieldError(error, 'amount')}
        >
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
        <div className="col-span-2">
          <Field label={<Localized id="gardens-field-note" />} error={fieldError(error, 'note')}>
            {(props) => (
              <FreeTextInput
                {...props}
                value={draft.note}
                onChange={(note) => {
                  onChange({ note });
                }}
              />
            )}
          </Field>
        </div>
      </>
    );
  }
  const costType = draft.type === 'cost';
  return (
    <>
      <Field
        label={<Localized id={costType ? 'gardens-cost-amount' : 'gardens-installment-amount'} />}
        hint={<Localized id="gardens-payment-amount-hint" />}
        error={fieldError(error, 'amount')}
      >
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
      <div className="col-span-2">
        <Field
          label={<Localized id="gardens-payment-reason" />}
          hint={
            <Localized
              id={costType ? 'gardens-cost-reason-hint' : 'gardens-installment-reason-hint'}
            />
          }
          error={fieldError(error, 'reason')}
        >
          {(props) => (
            <FreeTextInput
              {...props}
              value={draft.reason}
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
