// The fields of a purchase entry that are not on a line (PRD 12.10, FR-12.24, FR-12.41): who the
// goods were bought from (as typed) and the day to sell, which every line takes. The day, the note
// and the sources of the partners are on the trip screen for every type, in the same places (PRD
// FR-7.67); the item lines are in the lines grid. There is no way of paying here: a purchase is
// paid at once, from the shop's cash or the partners' pockets (PRD A53).
//
// Everything is text in the screen's state, so a language switch loses nothing (F39). Nothing
// here works out money: the numbers beside the form are the backend's, from the preview.

import { Localized } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { DateInput, Field, FreeTextInput, fieldError } from '../../../shared/ui';
import type { PurchaseEntryFields } from '../types';

interface PurchaseEntryFieldsFormProps {
  fields: PurchaseEntryFields;
  onChange: (patch: Partial<PurchaseEntryFields>) => void;
  /** What the backend refused or the preview found, with the field paths. */
  error: UiError | null;
  /** The day of the purchase as typed (`DD-MM-YYYY`): the date to sell starts as this day. */
  purchaseDate: string;
}

export function PurchaseEntryFieldsForm({
  fields,
  onChange,
  error,
  purchaseDate,
}: PurchaseEntryFieldsFormProps) {
  return (
    <>
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
