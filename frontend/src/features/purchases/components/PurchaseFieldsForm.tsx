// The fields only a purchase has (PRD 12.10, FR-12.24): who it was bought from (as typed), the item
// and packing, the quantity (and the weight of an item sold by weight), the rate, and the three
// costs paid at once. The day, the note and the sources of the partners are on the trip screen
// for every type, in the same places (PRD FR-7.67). There is no way of paying here: a purchase
// is paid at once, from the shop's cash or the partners' pockets (PRD A53).
//
// Everything is text in the screen's state, so a language switch loses nothing (F39). Nothing
// here works out money: the numbers under the form are the backend's, from the preview.

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
  type PickerValue,
} from '../../../shared/ui';
import type { PurchaseFields } from '../types';

interface PurchaseFieldsFormProps {
  fields: PurchaseFields;
  onChange: (patch: Partial<PurchaseFields>) => void;
  /** What the backend refused or the preview found, with the field paths. */
  error: UiError | null;
  /** The weight field shows: the item is sold by weight, or a weight was typed. */
  showWeight: boolean;
  /** The day of the purchase as typed (`DD-MM-YYYY`): the date to sell starts as this day. */
  purchaseDate: string;
}

/** The two names of a record a picker gave back. */
function valueOf(hit: { id: number; code: string; name_ur: string; name_en: string }): PickerValue {
  return pickedOf(hit);
}

export function PurchaseFieldsForm({
  fields,
  onChange,
  error,
  showWeight,
  purchaseDate,
}: PurchaseFieldsFormProps) {
  return (
    <>
      <div className="col-span-2">
        <Field
          label={<Localized id="purchases-field-supplier" />}
          hint={<Localized id="purchases-supplier-hint" />}
          error={fieldError(error, 'supplier')}
        >
          {(props) => (
            <FreeTextInput
              {...props}
              value={fields.supplier}
              onChange={(supplier) => {
                onChange({ supplier });
              }}
            />
          )}
        </Field>
      </div>
      <Field label={<Localized id="glossary-qism-maal" />} error={fieldError(error, 'item_id')}>
        {(props) => (
          <Picker
            {...props}
            scope="item"
            value={fields.item}
            onChange={(hit) => {
              onChange({ item: hit === null ? null : valueOf(hit) });
            }}
          />
        )}
      </Field>
      <Field label={<Localized id="glossary-packing" />} error={fieldError(error, 'packing_id')}>
        {(props) => (
          <Picker
            {...props}
            scope="packing"
            value={fields.packing}
            onChange={(hit) => {
              onChange({ packing: hit === null ? null : valueOf(hit) });
            }}
          />
        )}
      </Field>
      <Field
        label={<Localized id="purchases-field-qty" />}
        hint={<Localized id="purchases-qty-hint" />}
        error={fieldError(error, 'qty')}
      >
        {(props) => (
          <QtyInput
            {...props}
            value={fields.qty}
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
              value={fields.weight}
              onChange={(weight) => {
                onChange({ weight });
              }}
            />
          )}
        </Field>
      ) : null}
      <Field
        label={<Localized id="purchases-field-rate" />}
        hint={<Localized id="purchases-rate-hint" />}
        error={fieldError(error, 'rate')}
      >
        {(props) => (
          <MoneyInput
            {...props}
            value={fields.rate}
            onChange={(rate) => {
              onChange({ rate });
            }}
          />
        )}
      </Field>
      <Field label={<Localized id="glossary-kiraya" />} error={fieldError(error, 'freight')}>
        {(props) => (
          <MoneyInput
            {...props}
            value={fields.freight}
            onChange={(freight) => {
              onChange({ freight });
            }}
          />
        )}
      </Field>
      <Field label={<Localized id="glossary-mazdoori" />} error={fieldError(error, 'labour')}>
        {(props) => (
          <MoneyInput
            {...props}
            value={fields.labour}
            onChange={(labour) => {
              onChange({ labour });
            }}
          />
        )}
      </Field>
      <Field
        label={<Localized id="purchases-field-other-cost" />}
        error={fieldError(error, 'other_cost')}
      >
        {(props) => (
          <MoneyInput
            {...props}
            value={fields.otherCost}
            onChange={(otherCost) => {
              onChange({ otherCost });
            }}
          />
        )}
      </Field>
      <Field
        label={<Localized id="glossary-sell-date" />}
        hint={<Localized id="purchases-sell-date-hint" />}
        error={fieldError(error, 'sell_date')}
      >
        {(props) => (
          <DateInput
            {...props}
            // The box shows the day of the purchase until the user types another (NFR-USE-27).
            value={fields.sellDate === '' ? purchaseDate : fields.sellDate}
            onChange={(sellDate) => {
              onChange({ sellDate });
            }}
          />
        )}
      </Field>
    </>
  );
}
