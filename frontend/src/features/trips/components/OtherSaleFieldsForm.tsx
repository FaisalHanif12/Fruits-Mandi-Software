// The fields only a sale in another shop has (PRD FR-12.32): which purchase the goods come from
// (only a purchase of this trip with goods left), the shop as typed, how many نگ were sold and
// the money received. The qty and its cost (C21) come off the purchase; the money comes into
// روکڑ and goes to the trip's partners by their % (C25). The preview shows all of it.

import { Localized, useLocalization } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Field, FreeTextInput, MoneyInput, QtyInput, Select, fieldError } from '../../../shared/ui';
import { purchaseNumberText, type PurchaseRow } from '../../purchases';
import type { OtherSaleFields } from '../types';

interface OtherSaleFieldsFormProps {
  fields: OtherSaleFields;
  amount: string;
  /** The purchases of this trip that have goods left, with what is left in each part (PRD FR-12.41). */
  purchases: readonly PurchaseRow[];
  onChange: (patch: { shop?: string; purchaseId?: string; qty?: string; amount?: string }) => void;
  error: UiError | null;
}

export function OtherSaleFieldsForm({
  fields,
  amount,
  purchases,
  onChange,
  error,
}: OtherSaleFieldsFormProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);
  return (
    <>
      <div className="col-span-2">
        <Field
          label={<Localized id="trips-sale-purchase" />}
          hint={<Localized id="trips-sale-purchase-hint" />}
          error={fieldError(error, 'purchase_id')}
        >
          {(props) => (
            <Select
              {...props}
              value={fields.purchaseId === '' ? undefined : fields.purchaseId}
              onChange={(purchaseId) => {
                onChange({ purchaseId });
              }}
              options={purchases.map((purchase) => ({
                value: String(purchase.id),
                label: l10n.getString('trips-sale-purchase-option', {
                  number: purchaseNumberText(purchase.number, say),
                  item: displayName(purchase.item, language),
                  packing: displayName(purchase.packing, language),
                  other_left: String(purchase.qty_other_left),
                  our_left: String(purchase.qty_our_left),
                }),
              }))}
            />
          )}
        </Field>
      </div>
      <div className="col-span-2">
        <Field label={<Localized id="trips-sale-shop" />} error={fieldError(error, 'shop')}>
          {(props) => (
            <FreeTextInput
              {...props}
              value={fields.shop}
              onChange={(shop) => {
                onChange({ shop });
              }}
            />
          )}
        </Field>
      </div>
      <Field
        label={<Localized id="trips-sale-qty" />}
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
      <Field
        label={<Localized id="trips-sale-amount" />}
        hint={<Localized id="trips-sale-amount-hint" />}
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
    </>
  );
}
