// The rest of a lot, behind "مزید" (PRD FR-12.2, 12.3): the مارکہ, store, weight, فون, کرایہ سٹور,
// who paid the freight, the tick "پیشگی سے کاٹیں" (FR-12.23), how each charge typed in the grid is
// meant (the amount itself, for each نگ, or for the lot), and the one-time seller. It is for the
// selected lot. A problem on one of these fields opens it by itself, so it is never hidden.
//
// Values are text on the lot's row (Code Standards F39). The app shows what a rule suggested for
// a charge as words beside it, and puts it in the cell of the grid (NFR-USE-27).

import { Localized, useLocalization } from '@fluent/react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import type { UiError } from '../../../bindings/UiError';
import { usePartyCard } from '../../../shared/accounts';
import { formatPaisa, useGrouping } from '../../../shared/format';
import type { EntryRow, PickedRecord } from '../../../shared/grid';
import {
  Button,
  Card,
  Checkbox,
  Field,
  MoneyInput,
  Num,
  Picker,
  SegmentedControl,
  fieldError,
} from '../../../shared/ui';
import { useBikriDraft } from '../draft-store';
import { isPukhtaSeller } from '../draft';
import { LOT, ON, type LotPreview } from '../types';
import { PukhtaFields } from './PukhtaFields';

interface LotMoreProps {
  row: EntryRow;
  lotIndex: number;
  /** The preview of this lot: the charges the rules suggested. */
  lot: LotPreview | undefined;
  error: UiError | null;
  /** A problem is on a field of this panel: it stays open. */
  hasProblem: boolean;
}

type Method = 'manual' | 'per_unit' | 'per_lot';
const METHODS: readonly Method[] = ['manual', 'per_unit', 'per_lot'];

/** The three charges whose meaning is chosen here. */
const CHARGES = [
  { method: LOT.labourMethod, titleId: 'glossary-mazdoori', pick: (lot: LotPreview) => lot.labour },
  {
    method: LOT.munshianaMethod,
    titleId: 'glossary-munshiana',
    pick: (lot: LotPreview) => lot.munshiana,
  },
  { method: LOT.phoneMethod, titleId: 'glossary-phone', pick: (lot: LotPreview) => lot.phone },
] as const;

function methodOf(text: string | undefined): Method {
  return text === 'per_unit' || text === 'per_lot' ? text : 'manual';
}

/**
 * The button "مزید" of the lot (PRD FR-12.2). It sits in the heading of the lots, next to "add a
 * lot", and opens the panel below the grid. The panel stays open while one of its fields has a
 * problem or the seller is a one-time person, so the button then has nothing to close.
 */
export function LotMoreToggle({ lotIndex, forced }: { lotIndex: number; forced: boolean }) {
  const stores = useBikriDraft();
  const open = stores.view((state) => state.moreOpen);
  return (
    <Button
      disabled={forced}
      onClick={() => {
        stores.view.getState().setMore(!open);
      }}
    >
      {open || forced ? (
        <ChevronUp aria-hidden="true" className="size-icon-md" />
      ) : (
        <ChevronDown aria-hidden="true" className="size-icon-md" />
      )}
      <Localized id="bikri-more-toggle" vars={{ lot: String(lotIndex + 1) }} />
    </Button>
  );
}

export function LotMore({ row, lotIndex, lot, error, hasProblem }: LotMoreProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const stores = useBikriDraft();
  // A one-time seller needs his name and phone, so the panel opens for him by itself.
  const open = stores.view((state) => state.moreOpen) || hasProblem || isPukhtaSeller(row);
  const seller = row.picked?.[LOT.seller];
  const card = usePartyCard(seller?.id ?? null);
  const advance = card.data?.advance_open ?? null;
  // The tick shows only for a seller who has an advance open (PRD 12.3).
  const hasAdvance = advance !== null && formatPaisa(advance, { grouping }) !== '0';
  const path = (sub: string) => `lots[${String(lotIndex)}].${sub}`;
  const typed = (field: string) => row.typed[field] ?? '';
  const setTyped = (field: string, text: string) => {
    stores.lots.getState().updateTyped(row.rowId, field, text);
  };
  const setPicked = (field: string, hit: PickedRecord | null) => {
    stores.lots
      .getState()
      .updatePicked(
        row.rowId,
        field,
        hit === null
          ? null
          : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
      );
  };
  const methodLabel: Record<Method, string> = {
    manual: l10n.getString('bikri-method-manual'),
    per_unit: l10n.getString('bikri-method-per-unit'),
    per_lot: l10n.getString('bikri-method-per-lot'),
  };

  return (
    <div className="flex flex-col gap-2">
      {open ? (
        <Card compact>
          <div className="grid grid-cols-2 gap-x-4 gap-y-3 normal:grid-cols-4">
            <Field label={<Localized id="glossary-marka" />}>
              {(control) => (
                <Picker
                  {...control}
                  scope="marka"
                  value={row.picked?.[LOT.marka] ?? null}
                  onChange={(hit) => {
                    setPicked(LOT.marka, hit);
                  }}
                />
              )}
            </Field>
            <Field label={<Localized id="bikri-header-store" />}>
              {(control) => (
                <Picker
                  {...control}
                  scope="store"
                  value={row.picked?.[LOT.store] ?? null}
                  onChange={(hit) => {
                    setPicked(LOT.store, hit);
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="bikri-more-weight" />}
              error={fieldError(error, path('weight_received'))}
            >
              {(control) => (
                <MoneyInput
                  {...control}
                  value={typed(LOT.weight)}
                  onChange={(text) => {
                    setTyped(LOT.weight, text);
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="glossary-kiraya-store" />}
              error={fieldError(error, path('store_rent'))}
            >
              {(control) => (
                <MoneyInput
                  {...control}
                  value={typed(LOT.storeRent)}
                  onChange={(text) => {
                    setTyped(LOT.storeRent, text);
                  }}
                />
              )}
            </Field>
            <Field
              label={<Localized id="glossary-phone" />}
              error={fieldError(error, path('phone'))}
            >
              {(control) => (
                <MoneyInput
                  {...control}
                  value={typed(LOT.phone)}
                  placeholder={
                    lot?.phone === null || lot?.phone === undefined
                      ? ''
                      : formatPaisa(lot.phone.value_paisa, { grouping })
                  }
                  onChange={(text) => {
                    setTyped(LOT.phone, text);
                  }}
                />
              )}
            </Field>
          </div>
          <div className="flex flex-wrap items-center gap-x-6 gap-y-3">
            <div className="flex items-center gap-3">
              <Localized id="bikri-more-freight-paid-by">
                <span className="type-label" />
              </Localized>
              <SegmentedControl
                groupLabel={l10n.getString('bikri-more-freight-paid-by')}
                value={typed(LOT.freightPaidBy) === 'seller' ? 'seller' : 'firm'}
                onChange={(who) => {
                  setTyped(LOT.freightPaidBy, who);
                }}
                options={[
                  { value: 'firm', label: <Localized id="glossary-arhti" /> },
                  { value: 'seller', label: <Localized id="glossary-beopari" /> },
                ]}
              />
            </div>
            {hasAdvance ? (
              <div className="flex flex-wrap items-center gap-3">
                <Checkbox
                  checked={typed(LOT.adjustAdvance) !== '0'}
                  onChange={(checked) => {
                    setTyped(LOT.adjustAdvance, checked ? ON : '0');
                  }}
                  label={<Localized id="bikri-more-adjust-advance" />}
                />
                <Localized
                  id="bikri-more-advance-open"
                  vars={{ amount: formatPaisa(advance, { grouping }) }}
                  elems={{ num: <Num /> }}
                >
                  <span className="type-body text-ink-muted" />
                </Localized>
              </div>
            ) : null}
          </div>
          <div className="flex flex-col gap-2">
            <Localized id="bikri-more-charge-method">
              <p className="type-label" />
            </Localized>
            <div className="flex flex-wrap gap-x-6 gap-y-3">
              {CHARGES.map((charge) => {
                const suggested = lot === undefined ? null : charge.pick(lot);
                return (
                  <div key={charge.method} className="flex flex-wrap items-center gap-2">
                    <Localized id={charge.titleId}>
                      <span className="type-body" />
                    </Localized>
                    <SegmentedControl
                      groupLabel={l10n.getString(charge.titleId)}
                      value={methodOf(typed(charge.method))}
                      onChange={(method) => {
                        setTyped(charge.method, method);
                      }}
                      options={METHODS.map((method) => ({
                        value: method,
                        label: methodLabel[method],
                      }))}
                    />
                    {suggested?.source === 'rule' ? (
                      <Localized
                        id={`bikri-more-rule-${suggested.method.replaceAll('_', '-')}`}
                        vars={{ value: formatPaisa(suggested.value_paisa, { grouping }) }}
                        elems={{ num: <Num /> }}
                      >
                        <span className="type-body text-ink-muted" />
                      </Localized>
                    ) : null}
                  </div>
                );
              })}
            </div>
          </div>
          {isPukhtaSeller(row) ? (
            <PukhtaFields
              store={stores.lots}
              row={row}
              side="seller"
              keys={{
                ur: LOT.pukhtaUr,
                en: LOT.pukhtaEn,
                phone: LOT.pukhtaPhone,
                marks: `${LOT.pukhtaUr}Marks`,
              }}
              paths={{
                ur: path('pukhta.name_ur'),
                en: path('pukhta.name_en'),
                phone: path('pukhta.phone'),
              }}
              error={error}
              onDone={() => {
                const index = stores.lots.getState().rows.findIndex((r) => r.rowId === row.rowId);
                stores.lots.getState().setPendingFocus({ rowIndex: index, field: LOT.item });
              }}
            />
          ) : null}
        </Card>
      ) : null}
    </div>
  );
}
