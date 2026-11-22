// The fields only a sale in another shop has (PRD FR-12.32): which purchase the goods come from
// (only a purchase of this trip with goods left), the shop as typed, how many نگ were sold and
// the money received. The qty and its cost (C21) come off the purchase; the money comes into
// روکڑ and goes to the trip's partners by their % (C25). The preview shows all of it.

import { Localized, useLocalization } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Field, FreeTextInput, MoneyInput, QtyInput, Select, fieldError } from '../../../shared/ui';
import {
  OtherShopNone,
  OtherShopQtyError,
  offeredForOtherShop,
  purchaseNumberText,
  shopAfterPicking,
  type PurchaseRow,
} from '../../purchases';
import type { OtherSaleFields } from '../types';

interface OtherSaleFieldsFormProps {
  fields: OtherSaleFields;
  amount: string;
  /** The purchases of this trip, with what is left in each part (PRD FR-12.41). */
  purchases: readonly PurchaseRow[];
  onChange: (patch: { shop?: string; purchaseId?: string; qty?: string; amount?: string }) => void;
  error: UiError | null;
  /**
   * How many نگ are left for another shop on the chosen purchase, when the qty typed is more than
   * that (plan 90): the qty field says so and the save waits. Null: nothing to say (also on an edit).
   */
  qtyAbove?: number | null;
  /** The purchases are still being read: no "nothing left" yet. */
  purchasesLoading?: boolean;
}

export function OtherSaleFieldsForm({
  fields,
  amount,
  purchases,
  onChange,
  error,
  qtyAbove = null,
  purchasesLoading = false,
}: OtherSaleFieldsFormProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);
  // Only the purchases with goods left for another shop (plan 90, PRD FR-12.41); the one already
  // chosen stays, so an entry open for edit still shows the purchase it was made on.
  const offered = offeredForOtherShop(purchases, fields.purchaseId);
  const pick = (purchaseId: string) => {
    const previous = purchases.find((purchase) => String(purchase.id) === fields.purchaseId);
    const next = purchases.find((purchase) => String(purchase.id) === purchaseId);
    // The shop typed on that purchase fills the shop box (it stays editable, NFR-USE-27).
    onChange({
      purchaseId,
      shop: shopAfterPicking(fields.shop, previous?.shop_name, next?.shop_name),
    });
  };
  return (
    <>
      <div className="col-span-2">
        {offered.length === 0 ? (
          purchasesLoading ? null : (
            <OtherShopNone kind="purchase" />
          )
        ) : (
          <Field
            label={<Localized id="trips-sale-purchase" />}
            info={
              <Localized id="trips-sale-purchase-hint">
                <p />
              </Localized>
            }
            error={fieldError(error, 'purchase_id')}
          >
            {(props) => (
              <Select
                {...props}
                value={fields.purchaseId === '' ? undefined : fields.purchaseId}
                onChange={pick}
                options={offered.map((purchase) => ({
                  value: String(purchase.id),
                  label: l10n.getString('trips-sale-purchase-option', {
                    number: purchaseNumberText(purchase.number, say),
                    item: displayName(purchase.item, language),
                    packing: displayName(purchase.packing, language),
                    other_left: String(purchase.qty_other_left),
                  }),
                }))}
              />
            )}
          </Field>
        )}
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
        info={
          <Localized id="purchases-qty-hint">
            <p />
          </Localized>
        }
        error={
          qtyAbove === null ? fieldError(error, 'qty') : <OtherShopQtyError remaining={qtyAbove} />
        }
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
        info={
          <Localized id="trips-sale-amount-hint">
            <p />
          </Localized>
        }
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
