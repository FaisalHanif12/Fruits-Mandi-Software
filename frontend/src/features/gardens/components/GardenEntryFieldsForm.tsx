// The fields of the garden's entry form, by type (PRD FR-7.67, FR-12.35, FR-12.36). An installment
// and a running cost have the same two fields: the amount (it may be a simple sum such as
// `20000+5000`, PRD FR-7.48; the backend reads it) and why. A sale in another shop has the shop as
// typed, how many نگ were sold, the money received and a note. The day, and the sources of the
// partners, are on the screen for every type in the same places.

import { Localized, useLocalization } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  DateInput,
  Field,
  FreeTextInput,
  MoneyInput,
  QtyInput,
  Select,
  fieldError,
} from '../../../shared/ui';
import {
  OtherShopNone,
  OtherShopQtyError,
  offeredForOtherShop,
  shopAfterPicking,
} from '../../purchases';
import { arrivalNumberText } from '../list-columns';
import type { ArrivalRow, EntryDraft } from '../types';

interface GardenEntryFieldsFormProps {
  draft: EntryDraft;
  onChange: (patch: Partial<EntryDraft>) => void;
  error: UiError | null;
  /**
   * The garden's goods arrived, for a sale in another shop (PRD FR-12.41). The sale picks the
   * entry its نگ come from: only those standing with goods left are offered.
   * A garden with no goods arrived has none, and the sale is typed by hand as before.
   */
  arrivals: readonly ArrivalRow[];
  /**
   * How many نگ are left for another shop on the chosen line, when the qty typed is more than that
   * (plan 90): the qty field says so and the save waits. Null: nothing to say (also on an edit).
   */
  qtyAbove?: number | null;
  /** The garden's goods arrived are still being read: no "nothing left" yet. */
  arrivalsLoading?: boolean;
}

export function GardenEntryFieldsForm({
  draft,
  onChange,
  error,
  arrivals,
  qtyAbove = null,
  arrivalsLoading = false,
}: GardenEntryFieldsFormProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  if (draft.type === 'arrival') {
    // The item lines are in the grid under the form (PRD FR-12.41); here is what the whole entry
    // shares: the day to sell, and a note every line carries.
    return (
      <>
        <Field
          label={<Localized id="glossary-sell-date" />}
          info={
            <Localized id="gardens-arrival-sell-date-hint">
              <p />
            </Localized>
          }
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
    // Only the lines with goods left for another shop (plan 90, PRD FR-12.41); the line already
    // chosen stays, so an entry open for edit still shows the line it was made on.
    const offered = offeredForOtherShop(
      arrivals.filter((arrival) => arrival.status === 'saved'),
      draft.arrivalId,
    );
    const pick = (arrivalId: string) => {
      const previous = arrivals.find((arrival) => String(arrival.id) === draft.arrivalId);
      const next = arrivals.find((arrival) => String(arrival.id) === arrivalId);
      // The shop typed on that line fills the shop box (it stays editable, NFR-USE-27).
      onChange({
        arrivalId,
        shop: shopAfterPicking(draft.shop, previous?.shop_name, next?.shop_name),
      });
    };
    return (
      <>
        <div className="col-span-2">
          {offered.length === 0 ? (
            arrivalsLoading ? null : (
              <OtherShopNone kind="arrival" />
            )
          ) : (
            <Field
              label={<Localized id="gardens-sale-arrival" />}
              info={
                <Localized id="gardens-sale-arrival-hint">
                  <p />
                </Localized>
              }
              error={fieldError(error, 'arrival_id')}
            >
              {(props) => (
                <Select
                  {...props}
                  value={draft.arrivalId === '' ? undefined : draft.arrivalId}
                  onChange={pick}
                  options={offered.map((arrival) => ({
                    value: String(arrival.id),
                    label: l10n.getString('gardens-sale-arrival-option', {
                      number: arrivalNumberText(arrival.number, (id, vars) =>
                        l10n.getString(id, vars),
                      ),
                      item: displayName(arrival.item, language),
                      packing: displayName(arrival.packing, language),
                      other_left: String(arrival.qty_other_left),
                    }),
                  }))}
                />
              )}
            </Field>
          )}
        </div>
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
          info={
            <Localized id="gardens-sale-qty-hint">
              <p />
            </Localized>
          }
          error={
            qtyAbove === null ? (
              fieldError(error, 'qty')
            ) : (
              <OtherShopQtyError remaining={qtyAbove} />
            )
          }
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
          info={
            <Localized id="gardens-sale-amount-hint">
              <p />
            </Localized>
          }
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
  if (draft.type === 'refund') {
    // The owner's refund after a rent relief (plan 78 D3): the money, and a note. It comes into روکڑ.
    return (
      <>
        <Field
          label={<Localized id="gardens-refund-amount" />}
          info={
            <Localized id="gardens-refund-amount-hint">
              <p />
            </Localized>
          }
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
        info={
          <Localized id="gardens-payment-amount-hint">
            <p />
          </Localized>
        }
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
          info={
            <Localized
              id={costType ? 'gardens-cost-reason-hint' : 'gardens-installment-reason-hint'}
            >
              <p />
            </Localized>
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
