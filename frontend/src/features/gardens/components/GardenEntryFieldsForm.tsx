// The fields of the garden's entry form, by type (PRD FR-7.67, FR-12.35, FR-12.36). An installment
// and a running cost have the same two fields: the amount (it may be a simple sum such as
// `20000+5000`, PRD FR-7.48; the backend reads it) and why. A sale in another shop has the shop as
// typed, how many نگ were sold, the money received and a note. The day, and the sources of the
// partners, are on the screen for every type in the same places.

import { Localized } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { pickedOf } from '../../../shared/accounts';
import {
  DateInput,
  Field,
  FreeTextInput,
  MoneyInput,
  Picker,
  QtyInput,
  fieldError,
} from '../../../shared/ui';
import type { EntryDraft } from '../types';

interface GardenEntryFieldsFormProps {
  draft: EntryDraft;
  onChange: (patch: Partial<EntryDraft>) => void;
  error: UiError | null;
  /** Goods arrived: the weight field shows for an item sold by weight (the backend asks) or once typed. */
  showWeight: boolean;
}

export function GardenEntryFieldsForm({
  draft,
  onChange,
  error,
  showWeight,
}: GardenEntryFieldsFormProps) {
  if (draft.type === 'arrival') {
    return (
      <>
        <Field label={<Localized id="glossary-qism-maal" />} error={fieldError(error, 'item_id')}>
          {(props) => (
            <Picker
              {...props}
              scope="item"
              value={draft.item}
              onChange={(hit) => {
                onChange({ item: hit === null ? null : pickedOf(hit) });
              }}
            />
          )}
        </Field>
        <Field label={<Localized id="glossary-packing" />} error={fieldError(error, 'packing_id')}>
          {(props) => (
            <Picker
              {...props}
              scope="packing"
              value={draft.packing}
              onChange={(hit) => {
                onChange({ packing: hit === null ? null : pickedOf(hit) });
              }}
            />
          )}
        </Field>
        <Field
          label={<Localized id="gardens-arrival-qty" />}
          hint={<Localized id="gardens-arrival-qty-hint" />}
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
        {showWeight ? (
          <Field
            label={<Localized id="purchases-field-weight" />}
            error={fieldError(error, 'weight')}
          >
            {(props) => (
              <MoneyInput
                {...props}
                value={draft.weight}
                onChange={(weight) => {
                  onChange({ weight });
                }}
              />
            )}
          </Field>
        ) : null}
        <Field
          label={<Localized id="glossary-sell-date" />}
          hint={<Localized id="gardens-arrival-sell-date-hint" />}
          error={fieldError(error, 'sell_date')}
        >
          {(props) => (
            <DateInput
              {...props}
              // The box shows the day of the arrival until the user types another (NFR-USE-27).
              value={draft.sellDate === '' ? draft.date : draft.sellDate}
              onChange={(sellDate) => {
                onChange({ sellDate });
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
