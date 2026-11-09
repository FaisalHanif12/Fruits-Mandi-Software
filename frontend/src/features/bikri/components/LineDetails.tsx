// The details of the sale line that has focus (PRD 12.4, FR-11.4, FR-11.10, FR-12.3): the commission
// of the seller, the commission of the buyer and the لاگا, each with its rule tag, its amount and
// the ⓘ; a different buyer rate; the weight of a weight item; a note; and the one-time buyer.
//
// - A commission or لاگا can be typed on the line when the Owner allows it (the setting starts on,
//   FR-11.10). It is sent as typed, with the method chosen beside it, and saved with the method
//   "دستی". The field shows the value the rule gave as grey text. When the setting is off the
//   values are shown and cannot be changed.
// - The tag says which rule decided, for example "رحیم + کیلا 5%" (FR-11.4). The backend sends only
//   the level of the rule, so the names are the seller or buyer and the item of the line.
// - Everything is text on the row (Code Standards F39). Every amount is the backend's, with its
//   own working in the ⓘ (F42).

import { Localized, useLocalization } from '@fluent/react';
import type { LineCharge } from '../../../bindings/LineCharge';
import type { UiError } from '../../../bindings/UiError';
import { useGrouping } from '../../../shared/format';
import { useLanguage } from '../../../shared/i18n';
import { useFormNavigation } from '../../../shared/keyboard';
import type { EntryRow, EntryStore, PickedRecord } from '../../../shared/grid';
import {
  CalcInfo,
  Checkbox,
  Field,
  FreeTextInput,
  MoneyInput,
  Num,
  PercentInput,
  SegmentedControl,
  fieldError,
} from '../../../shared/ui';
import { chargeNumber, ruleTag, type Say } from '../preview-view';
import { ON, SALE, type LinePreview } from '../types';

type Method = 'percent' | 'per_unit';

function methodOf(text: string, fallback: Method): Method {
  return text === 'percent' || text === 'per_unit' ? text : fallback;
}

interface ChargeSpec {
  titleId: string;
  value: string;
  method: string;
  fallback: Method;
  sub: string;
  pick: (calc: NonNullable<LinePreview['calc']>) => LineCharge;
  whose: 'seller' | 'buyer';
}

const CHARGES: readonly ChargeSpec[] = [
  {
    titleId: 'bikri-line-seller-comm',
    value: SALE.sellerCommValue,
    method: SALE.sellerCommMethod,
    fallback: 'percent',
    sub: 'seller_comm',
    pick: (calc) => calc.seller_comm,
    whose: 'seller',
  },
  {
    titleId: 'bikri-line-buyer-comm',
    value: SALE.buyerCommValue,
    method: SALE.buyerCommMethod,
    fallback: 'percent',
    sub: 'buyer_comm',
    pick: (calc) => calc.buyer_comm,
    whose: 'buyer',
  },
  {
    titleId: 'glossary-laga',
    value: SALE.lagaValue,
    method: SALE.lagaMethod,
    fallback: 'per_unit',
    sub: 'laga',
    pick: (calc) => calc.laga,
    whose: 'buyer',
  },
];

interface LineDetailsProps {
  store: EntryStore;
  row: EntryRow;
  /** The seller of the lot and its item, for the names in the rule tags. */
  seller: PickedRecord | undefined;
  item: PickedRecord | undefined;
  lotIndex: number;
  lineIndex: number;
  line: LinePreview | undefined;
  /**
   * May a commission or لاگا be typed on a line? (the setting, PRD FR-11.10). Null while the
   * setting is not known yet: the fields look as they do when it is on, and the note is hidden.
   */
  allowOverride: boolean | null;
  /** Show the weight field: the item is sold by weight. */
  usesWeight: boolean;
  error: UiError | null;
  /** Enter on the last field: the screen takes focus back to the grid. */
  onDone: () => void;
}

export function LineDetails({
  store,
  row,
  seller,
  item,
  lotIndex,
  lineIndex,
  line,
  allowOverride,
  usesWeight,
  error,
  onDone,
}: LineDetailsProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const say: Say = (id, vars) => l10n.getString(id, vars);
  const path = (sub: string) => `lots[${String(lotIndex)}].sales[${String(lineIndex)}].${sub}`;
  const typed = (field: string) => row.typed[field] ?? '';
  const setTyped = (field: string, text: string) => {
    store.getState().updateTyped(row.rowId, field, text);
  };
  const navigation = useFormNavigation(onDone);
  const calc = line?.calc ?? null;
  const buyer = row.picked?.[SALE.buyer];
  const differs = typed(SALE.differs) === ON;
  // Only a setting that is really off switches the fields off (PRD FR-11.10).
  const off = allowOverride === false;

  return (
    <section
      {...navigation}
      aria-labelledby={`line-details-${row.rowId}`}
      className="flex flex-col gap-3"
    >
      <Localized id="bikri-line-title" vars={{ line: String(lineIndex + 1) }}>
        <h3 id={`line-details-${row.rowId}`} className="type-label text-ink" />
      </Localized>
      <div className="grid grid-cols-3 gap-x-3 gap-y-2">
        {CHARGES.map((spec) => {
          const charge = calc === null ? null : spec.pick(calc);
          const method = methodOf(typed(spec.method), spec.fallback);
          const names = { party: spec.whose === 'seller' ? seller : buyer, item };
          const Input = method === 'percent' ? PercentInput : MoneyInput;
          return (
            <Field
              key={spec.sub}
              label={<Localized id={spec.titleId} />}
              hint={
                charge === null ? undefined : (
                  <span className="flex flex-wrap items-center gap-x-2">
                    <span>{ruleTag(charge, names, language, grouping, say)}</span>
                  </span>
                )
              }
              error={fieldError(error, path(spec.sub))}
            >
              {(control) => (
                <div className="flex flex-col gap-2">
                  <Input
                    {...control}
                    readOnly={off}
                    value={typed(spec.value)}
                    placeholder={charge === null ? '' : chargeNumber(charge, grouping)}
                    onChange={(text) => {
                      setTyped(spec.value, text);
                    }}
                  />
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    {off ? null : (
                      <SegmentedControl
                        groupLabel={l10n.getString(spec.titleId)}
                        value={method}
                        onChange={(next) => {
                          setTyped(spec.method, next);
                        }}
                        options={[
                          { value: 'percent', label: l10n.getString('bikri-line-method-percent') },
                          {
                            value: 'per_unit',
                            label: l10n.getString('bikri-line-method-per-unit'),
                          },
                        ]}
                      />
                    )}
                    {charge === null ? null : (
                      <span className="inline-flex items-center gap-1">
                        <Num paisa={charge.amount} />
                        <CalcInfo working={charge.working} />
                      </span>
                    )}
                  </div>
                </div>
              )}
            </Field>
          );
        })}
      </div>
      {off ? (
        <Localized id="bikri-line-override-off">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
      <div className="grid grid-cols-3 gap-x-3 gap-y-2">
        <div className="flex flex-col gap-2">
          <Checkbox
            checked={differs}
            onChange={(checked) => {
              setTyped(SALE.differs, checked ? ON : '');
            }}
            label={<Localized id="bikri-line-differs" />}
          />
          {differs ? (
            <Field
              label={<Localized id="bikri-line-buyer-rate" />}
              error={fieldError(error, path('buyer_rate'))}
            >
              {(control) => (
                <div className="flex items-center gap-2">
                  <MoneyInput
                    {...control}
                    value={typed(SALE.buyerRate)}
                    onChange={(text) => {
                      setTyped(SALE.buyerRate, text);
                    }}
                  />
                  {calc === null ? null : (
                    <span className="inline-flex items-center gap-1">
                      <Localized id="glossary-rate-farq" />
                      <Num paisa={calc.rate_diff} />
                      <CalcInfo working={calc.workings.rate_diff} />
                    </span>
                  )}
                </div>
              )}
            </Field>
          ) : null}
        </div>
        {/* The seller amount of the line: in the grid from the middle window up, here in every window. */}
        <Field label={<Localized id="bikri-col-seller-amount" />} readOnly>
          {(control) => (
            <output id={control.id} className="inline-flex items-center gap-1 type-body text-ink">
              {calc === null ? null : (
                <>
                  <Num paisa={calc.seller_amount} />
                  <CalcInfo working={calc.workings.seller_amount} />
                </>
              )}
            </output>
          )}
        </Field>
        {/* What is left on the buyer's account (PRD FR-12.44): the backend worked it out. A buyer who must
            pay in full has his paid cell closed, and the note says why. A refusal of the paid amount shows here
            too, as it does on the cell. */}
        <Field
          label={<Localized id="bikri-line-left" />}
          readOnly
          hint={
            calc?.paid_fixed === true ? (
              // The backend's working says why: the buyer pays in full, or the saved line is kept.
              <Localized
                id={
                  calc.workings.paid.message_id === 'working-paid-kept'
                    ? 'bikri-paid-kept-note'
                    : 'bikri-paid-fixed-note'
                }
              />
            ) : undefined
          }
          error={fieldError(error, path('paid'))}
        >
          {(control) => (
            <output id={control.id} className="inline-flex items-center gap-1 type-body text-ink">
              {calc === null ? null : (
                <>
                  <Num paisa={calc.left_paisa} />
                  <CalcInfo working={calc.workings.left} />
                </>
              )}
            </output>
          )}
        </Field>
        {usesWeight ? (
          <Field
            label={<Localized id="bikri-line-weight" />}
            error={fieldError(error, path('weight'))}
          >
            {(control) => (
              <MoneyInput
                {...control}
                value={typed(SALE.weight)}
                onChange={(text) => {
                  setTyped(SALE.weight, text);
                }}
              />
            )}
          </Field>
        ) : null}
        <Field label={<Localized id="bikri-line-note" />}>
          {(control) => (
            <FreeTextInput
              {...control}
              value={typed(SALE.note)}
              onChange={(text) => {
                setTyped(SALE.note, text);
              }}
            />
          )}
        </Field>
      </div>
    </section>
  );
}
