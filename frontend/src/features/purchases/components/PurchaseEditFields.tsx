// The fields of one trip purchase open for edit (plan 44, PRD FR-7.70, FR-12.24): the same fields a
// new purchase has on a line of the lines grid, in a form, because an edit changes one purchase.
// The day is read only: it never changes (the band says so). The sources of the partners are on the
// trip's edit screen, as on the form of a new entry.
//
// Everything is text in the screen's state, so a language switch loses nothing (F39). Nothing here
// works out money: the numbers beside the form are the backend's, from the review.

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
import type { PurchaseEditDraft } from '../edit-draft';

interface PurchaseEditFieldsProps {
  draft: PurchaseEditDraft;
  onChange: (patch: Partial<PurchaseEditDraft>) => void;
  error: UiError | null;
}

export function PurchaseEditFields({ draft, onChange, error }: PurchaseEditFieldsProps) {
  // The weight of a purchase is asked for an item sold by weight: the item chosen says so, and a
  // weight already there stays (PRD FR-12.38).
  const byWeight = draft.item?.soldByWeight === true || draft.weight !== '';
  return (
    <>
      <Field label={<Localized id="purchases-field-date" />} error={fieldError(error, 'date')}>
        {(props) => <DateInput {...props} value={draft.date} readOnly onChange={() => undefined} />}
      </Field>
      <Field
        label={<Localized id="glossary-sell-date" />}
        hint={<Localized id="purchases-sell-date-hint" />}
        error={fieldError(error, 'sell_date')}
      >
        {(props) => (
          <DateInput
            {...props}
            value={draft.sellDate}
            onChange={(sellDate) => {
              onChange({ sellDate });
            }}
          />
        )}
      </Field>
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
      <Field label={<Localized id="purchases-field-qty" />} error={fieldError(error, 'qty')}>
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
      {byWeight ? (
        <Field
          label={<Localized id="purchases-field-weight" />}
          error={fieldError(error, 'weight')}
        >
          {(props) => (
            <FreeTextInput
              {...props}
              value={draft.weight}
              onChange={(weight) => {
                onChange({ weight });
              }}
            />
          )}
        </Field>
      ) : (
        <div />
      )}
      <Field label={<Localized id="purchases-field-rate" />} error={fieldError(error, 'rate')}>
        {(props) => (
          <MoneyInput
            {...props}
            value={draft.rate}
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
            value={draft.freight}
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
            value={draft.labour}
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
            value={draft.otherCost}
            onChange={(otherCost) => {
              onChange({ otherCost });
            }}
          />
        )}
      </Field>
      <Field
        label={<Localized id="goods-line-col-other-qty" />}
        error={fieldError(error, 'other_qty')}
      >
        {(props) => (
          <QtyInput
            {...props}
            value={draft.otherQty}
            onChange={(otherQty) => {
              onChange({ otherQty });
            }}
          />
        )}
      </Field>
      <Field label={<Localized id="goods-line-col-shop" />} error={fieldError(error, 'shop_name')}>
        {(props) => (
          <FreeTextInput
            {...props}
            value={draft.shopName}
            onChange={(shopName) => {
              onChange({ shopName });
            }}
          />
        )}
      </Field>
      <Field
        label={<Localized id="purchases-field-supplier" />}
        error={fieldError(error, 'supplier')}
      >
        {(props) => (
          <FreeTextInput
            {...props}
            value={draft.supplier}
            onChange={(supplier) => {
              onChange({ supplier });
            }}
          />
        )}
      </Field>
      <Field label={<Localized id="purchases-field-note" />} error={fieldError(error, 'note')}>
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
    </>
  );
}
