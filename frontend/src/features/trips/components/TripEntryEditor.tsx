// The entry form of one trip (PRD FR-7.67, FR-12.24, FR-12.31, FR-12.32): a type list at the top,
// خریداری, سفر خرچ and دوسری دکان میں فروخت, and under it only the fields of that type, always in
// the same places: the day first, then the fields of the type, then the note, then where each
// partner's money comes from. Changing the type keeps the day, the amount and the note, and
// clears the rest. Each type has its own number series, its own posting and its own checks.
//
// - Keys: F10 or Ctrl+S saves, F2 starts a new entry (asking first if there is typed work). The
//   form gets its Enter and arrow keys from shared/keyboard (F27).
// - Everything typed is text in this screen's state, so a language switch loses nothing (F39).
// - The frontend works out no money: every amount, each partner's part, the spread of a cost,
//   the cash after the entry and every warning come from the backend preview, which is the same
//   function as the save (Code Standards B5).
// - The sources default to the trip's and can be changed on the entry (PRD A54). A shop part
//   comes off the partner's total and out of روکڑ; a pocket part is only recorded (FR-12.30).
// - A purchase entry holds many item lines in a grid (PRD FR-12.41). The lines live in a row store
//   above the grid (F39). One save writes every line, each as its own purchase. A mistake on a
//   line shows on its cell, and the list under the form tells it too.
// - Four states: the form (empty), the preview loading or failed (an error with a retry), and
//   the numbers. The trip's own loading and error states are on the page.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Plus, TriangleAlert } from 'lucide-react';
import { useMemo, useRef, useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import type { Working } from '../../../bindings/Working';
import type { CashCheck } from '../../../bindings/CashCheck';
import type { FieldError } from '../../../bindings/FieldError';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { createEntryStore } from '../../../shared/grid';
import { displayName, useLanguage } from '../../../shared/i18n';
import { useEntrySaveFlow, useFocusFirstField, useRefusalIntoView } from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  CashConfirmDialog,
  CashNote,
  DateInput,
  EntryPartsCard,
  EntrySources,
  ErrorNotice,
  ErrorState,
  Field,
  FreeTextInput,
  SaveChangesPrompt,
  SegmentedControl,
  fieldError,
  showToast,
  useErrorText,
} from '../../../shared/ui';
import {
  GoodsLinesGrid,
  LINE,
  PURCHASE_FIELD_PATHS,
  PurchaseEntryFieldsForm,
  PurchaseEntryNumbersCard,
  filledLines,
  lineCellErrors,
  linesText,
  purchaseNumberText,
  purchasesEntryPreview,
  resetLines,
  useSavePurchaseEntry,
  usePurchaseEntryPreview,
  type EntryLineNumbers,
  type PurchaseRow,
} from '../../purchases';
import { tripsCostPreview } from '../api';
import {
  COST_FIELD_PATHS,
  SALE_FIELD_PATHS,
  buildEntryCost,
  buildEntryPurchase,
  buildEntrySale,
  changeType,
  entryIsFilled,
  newEntryDraft,
  nextEntryDraft,
} from '../draft';
import { useCostPreview, useOtherSalePreview, useSaveCost, useSaveOtherSale } from '../hooks';
import { ENTRY_TYPES, type EntryDraft, type EntryType, type TripRow } from '../types';
import { CostFieldsForm } from './CostFieldsForm';
import { CostNumbersCard } from './CostNumbersCard';
import { OtherSaleFieldsForm } from './OtherSaleFieldsForm';
import { OtherSaleNumbersCard } from './OtherSaleNumbersCard';

/** Codes that only say a field is empty. Held back until the user tries to save. */
const INCOMPLETE: ReadonlySet<string> = new Set(['ERR_REQUIRED', 'ERR_REASON_REQUIRED']);

const FIELD_PATHS: Record<EntryType, readonly string[]> = {
  purchase: PURCHASE_FIELD_PATHS,
  cost: COST_FIELD_PATHS,
  other_sale: SALE_FIELD_PATHS,
};

type PerRow<T> = Readonly<Record<string, Readonly<Record<string, T>>>>;

/** What the screen needs of the preview of the type that is open, whichever type it is. */
interface ActiveView {
  errors: readonly FieldError[];
  warnings: readonly FieldError[];
  cash: CashCheck | undefined;
  current: boolean;
  failed: boolean;
  retry: () => void;
  failure: unknown;
}

interface TripEntryEditorProps {
  trip: TripRow;
  /** The trip's purchases, for the purchase picker of a sale in another shop. */
  purchases: readonly PurchaseRow[];
  /** Today, `YYYY-MM-DD`. */
  today: string;
  /** A purchase was saved: the screen shows it in the list. */
  onPurchaseSaved: (purchaseId: number) => void;
}

export function TripEntryEditor({ trip, purchases, today, onPurchaseSaved }: TripEntryEditorProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const errorText = useErrorText();
  const language = useLanguage();
  const grouping = useGrouping();
  const [draft, setDraft] = useState<EntryDraft>(() => newEntryDraft('purchase', today));
  // The item lines of a purchase entry, kept above the grid so a language switch loses nothing.
  const [lines] = useState(() => createEntryStore());
  const rows = lines((state) => state.rows);
  const [tried, setTried] = useState(false);
  const [askNew, setAskNew] = useState(false);
  const formRef = useFocusFirstField<HTMLFormElement>();
  const rootRef = useRef<HTMLDivElement>(null);
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);

  const change = (patch: Partial<EntryDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
    setTried(false);
  };
  const filled = entryIsFilled(draft, rows);

  // The input of the type that is open, and nothing for the others, so only one preview is asked.
  const purchaseInput = useMemo(
    () =>
      filled && draft.type === 'purchase'
        ? buildEntryPurchase(draft, rows, trip.id, trip.partners, false)
        : null,
    [draft, filled, rows, trip.id, trip.partners],
  );
  const costInput = useMemo(
    () =>
      filled && draft.type === 'cost' ? buildEntryCost(draft, trip.id, trip.partners, false) : null,
    [draft, filled, trip.id, trip.partners],
  );
  const saleInput = useMemo(
    () => (filled && draft.type === 'other_sale' ? buildEntrySale(draft, trip.id) : null),
    [draft, filled, trip.id],
  );
  const purchaseView = usePurchaseEntryPreview(purchaseInput);
  const costView = useCostPreview(costInput);
  const saleView = useOtherSalePreview(saleInput);
  const purchasePreview = purchaseInput === null ? undefined : purchaseView.query.data;
  const costPreview = costInput === null ? undefined : costView.query.data;
  const salePreview = saleInput === null ? undefined : saleView.query.data;

  const active: ActiveView =
    draft.type === 'purchase'
      ? {
          errors: purchasePreview?.errors ?? [],
          warnings: purchasePreview?.warnings ?? [],
          cash: purchasePreview?.cash,
          current: purchaseView.current,
          failed: purchaseView.query.isError && purchaseInput !== null,
          retry: () => void purchaseView.query.refetch(),
          failure: purchaseView.query.error,
        }
      : draft.type === 'cost'
        ? {
            errors: costPreview?.errors ?? [],
            warnings: costPreview?.warnings ?? [],
            cash: costPreview?.cash,
            current: costView.current,
            failed: costView.query.isError && costInput !== null,
            retry: () => void costView.query.refetch(),
            failure: costView.query.error,
          }
        : {
            errors: salePreview?.errors ?? [],
            warnings: salePreview?.warnings ?? [],
            cash: undefined,
            current: saleView.current,
            failed: saleView.query.isError && saleInput !== null,
            retry: () => void saleView.query.refetch(),
            failure: saleView.query.error,
          };

  // What the preview found, in the shape of a refused save, so the fields can show it. Problems
  // that only say "this is empty" wait until the user tries to save, so a field does not turn red
  // before he has reached it.
  const previewError: UiError | null = useMemo(() => {
    const shown = active.errors.filter((item) => tried || !INCOMPLETE.has(item.code));
    return shown.length === 0
      ? null
      : {
          ok: false,
          errors: shown.map((item) => ({
            code: item.code,
            field: item.field,
            values: item.values,
          })),
          ref: '',
        };
  }, [active.errors, tried]);

  const draftText = useMemo(() => JSON.stringify({ draft, lines: linesText(rows) }), [draft, rows]);
  const afterSave = () => {
    setDraft((previous) => nextEntryDraft(previous));
    resetLines(lines);
    setTried(false);
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLElement>('input')?.focus();
    });
  };

  const savePurchase = useSavePurchaseEntry();
  const purchaseFlow = useEntrySaveFlow({
    draftText,
    build: (confirm) => buildEntryPurchase(draft, rows, trip.id, trip.partners, confirm),
    save: (input) => savePurchase.mutateAsync(input),
    freshCash: (input) => purchasesEntryPreview(input).then((preview) => preview.cash),
    cash: purchasePreview?.cash,
    current: purchaseView.current,
    onSaved: (done) => {
      const first = done.saved[0];
      const last = done.saved[done.saved.length - 1];
      if (first === undefined || last === undefined) return;
      showToast(
        'success',
        done.saved.length === 1
          ? l10n.getString('trips-saved-purchase', {
              number: purchaseNumberText(first.number, say),
            })
          : l10n.getString('trips-saved-purchases', {
              count: String(done.saved.length),
              first: purchaseNumberText(first.number, say),
              last: purchaseNumberText(last.number, say),
            }),
      );
      onPurchaseSaved(first.id);
      afterSave();
    },
  });
  const saveCost = useSaveCost();
  const costFlow = useEntrySaveFlow({
    draftText,
    build: (confirm) => buildEntryCost(draft, trip.id, trip.partners, confirm),
    save: (input) => saveCost.mutateAsync(input),
    freshCash: (input) => tripsCostPreview(input).then((preview) => preview.cash),
    cash: costPreview?.cash,
    current: costView.current,
    onSaved: (saved) => {
      showToast('success', l10n.getString('trips-saved-cost', { number: String(saved.number) }));
      afterSave();
    },
  });
  const saveSale = useSaveOtherSale();
  const saleFlow = useEntrySaveFlow({
    draftText,
    build: () => buildEntrySale(draft, trip.id),
    save: (input) => saveSale.mutateAsync(input),
    // Money only comes in with a sale in another shop, so cash is never asked about.
    freshCash: null,
    cash: undefined,
    current: saleView.current,
    onSaved: (saved) => {
      showToast(
        'success',
        l10n.getString(saved.trip_closed ? 'trips-saved-sale-closed' : 'trips-saved-sale', {
          number: String(saved.number),
        }),
      );
      afterSave();
    },
  });
  const flow =
    draft.type === 'purchase' ? purchaseFlow : draft.type === 'cost' ? costFlow : saleFlow;

  const error = flow.refusal ?? previewError;
  useRefusalIntoView(rootRef, flow.refusal);

  const save = async (): Promise<boolean> => {
    setTried(true);
    return flow.save();
  };
  const saveNow = () => {
    void save();
  };
  const navigation = useFormNavigation(saveNow);
  useShortcut(SHORTCUTS.save, saveNow);

  const startNew = () => {
    setDraft((previous) => newEntryDraft(previous.type, today));
    resetLines(lines);
    setTried(false);
    window.requestAnimationFrame(() => {
      formRef.current?.querySelector<HTMLElement>('input')?.focus();
    });
  };
  const requestNew = () => {
    if (filled) setAskNew(true);
    else startNew();
  };
  useShortcut(SHORTCUTS.newEntry, requestNew, { enabled: !askNew });
  useEscape('leave', () => {
    void navigate({ to: '/trips' });
  });

  // The lines the entry sends, in order: the backend counts these, so its problems for line i
  // belong to the i-th filled row. Its numbers come back with the key of the row (PRD FR-12.41).
  const sent = useMemo(() => filledLines(rows), [rows]);
  const previewOf = useMemo(
    () => new Map((purchasePreview?.lines ?? []).map((line) => [line.key, line])),
    [purchasePreview?.lines],
  );
  const lineNumbers: EntryLineNumbers[] = sent.map((row, index) => {
    const item = row.picked?.[LINE.item];
    const packing = row.picked?.[LINE.packing];
    return {
      key: row.rowId,
      name:
        [item, packing]
          .flatMap((hit) => (hit === undefined ? [] : [displayName(hit, language)]))
          .join(' ') || String(index + 1),
      numbers: previewOf.get(row.rowId)?.numbers ?? null,
    };
  });
  // What the backend worked out for each line, put in its row: the value of the line and the نگ
  // left for our shop.
  const calc: PerRow<string> = useMemo(() => {
    const cells: Record<string, Record<string, string>> = {};
    for (const row of sent) {
      const line = previewOf.get(row.rowId);
      if (line === undefined) continue;
      const cell: Record<string, string> = {};
      if (line.numbers !== null) {
        cell[LINE.value] = formatPaisa(line.numbers.value, { grouping });
      }
      if (line.qty_our !== null) cell[LINE.ourQty] = String(line.qty_our);
      cells[row.rowId] = cell;
    }
    return cells;
  }, [grouping, previewOf, sent]);
  // How the value of each line was worked out, behind the ⓘ of its cell.
  const workings: PerRow<Working> = useMemo(() => {
    const cells: Record<string, Record<string, Working>> = {};
    for (const row of sent) {
      const numbers = previewOf.get(row.rowId)?.numbers ?? null;
      if (numbers !== null) cells[row.rowId] = { [LINE.value]: numbers.workings.value };
    }
    return cells;
  }, [previewOf, sent]);
  const cellErrors: PerRow<string> = useMemo(
    () => lineCellErrors(error?.errors ?? [], sent, errorText),
    [error, errorText, sent],
  );
  // The words of a mistake on a line stay in the list under the form too: the cell is marked, and
  // colour is never the only signal (NFR-USE-12).
  const fieldsShown = FIELD_PATHS[draft.type];

  const typeOptions = ENTRY_TYPES.map((type) => ({
    value: type,
    label: l10n.getString(`trips-type-${type.replaceAll('_', '-')}`),
  }));
  const payment = draft.type !== 'other_sale';
  // Any purchase with goods left can be sold in another shop: the part set for another shop goes
  // first and the backend moves the rest from our shop's part (PRD FR-12.41, D1).
  const withStock = purchases.filter((purchase) => purchase.qty_left > 0);

  const numbers =
    draft.type === 'purchase' ? (
      <>
        <PurchaseEntryNumbersCard
          lines={lineNumbers}
          total={purchasePreview?.paid_now_total ?? null}
          empty={!filled}
        />
        <EntryPartsCard
          parts={purchasePreview?.parts ?? null}
          title={l10n.getString('trips-parts-title')}
        />
      </>
    ) : draft.type === 'cost' ? (
      <>
        <CostNumbersCard numbers={costPreview?.numbers ?? null} empty={!filled} />
        <EntryPartsCard
          parts={costPreview?.numbers?.parts ?? null}
          title={l10n.getString('trips-parts-title')}
        />
      </>
    ) : (
      <OtherSaleNumbersCard numbers={salePreview?.numbers ?? null} empty={!filled} />
    );

  return (
    <div ref={rootRef} className="flex flex-col gap-3">
      <LeaveGuard dirty={filled} onSave={save} />
      <div className="flex flex-wrap items-center justify-between gap-3">
        <SegmentedControl<EntryType>
          groupLabel={l10n.getString('trips-type-label')}
          options={typeOptions}
          value={draft.type}
          onChange={(type) => {
            setDraft((current) => changeType(current, type));
            resetLines(lines);
            setTried(false);
          }}
        />
        <div className="flex flex-wrap items-center gap-2">
          <Button shortcut={SHORTCUTS.newEntry} onClick={requestNew}>
            <Plus aria-hidden="true" className="size-icon-md" />
            <Localized id="trips-entry-new" />
          </Button>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            busy={flow.busy}
            disabled={!filled}
            onClick={saveNow}
          >
            <Localized id="layout-save-save" />
          </Button>
        </div>
      </div>
      <form
        ref={formRef}
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex flex-col gap-4"
      >
        {error === null ? null : <ErrorNotice error={error} fieldsShown={fieldsShown} />}
        <div className="grid grid-cols-1 gap-4 normal:grid-cols-5">
          <div className="grid grid-cols-2 gap-4 rounded-3 border border-border bg-surface p-3 normal:col-span-3">
            <Field label={<Localized id="trips-field-date" />} error={fieldError(error, 'date')}>
              {(props) => (
                <DateInput
                  {...props}
                  value={draft.date}
                  onChange={(date) => {
                    change({ date });
                  }}
                />
              )}
            </Field>
            {draft.type === 'purchase' ? null : <div />}
            {draft.type === 'purchase' ? (
              <PurchaseEntryFieldsForm
                fields={draft.purchase}
                onChange={(patch) => {
                  change({ purchase: { ...draft.purchase, ...patch } });
                }}
                error={error}
                purchaseDate={draft.date}
              />
            ) : draft.type === 'cost' ? (
              <CostFieldsForm
                fields={draft.cost}
                amount={draft.amount}
                error={error}
                onChange={({ reason, amount }) => {
                  change({
                    ...(reason === undefined ? {} : { cost: { reason } }),
                    ...(amount === undefined ? {} : { amount }),
                  });
                }}
              />
            ) : (
              <OtherSaleFieldsForm
                fields={draft.sale}
                amount={draft.amount}
                purchases={withStock}
                error={error}
                onChange={({ shop, purchaseId, qty, amount }) => {
                  change({
                    sale: {
                      shop: shop ?? draft.sale.shop,
                      purchaseId: purchaseId ?? draft.sale.purchaseId,
                      qty: qty ?? draft.sale.qty,
                    },
                    ...(amount === undefined ? {} : { amount }),
                  });
                }}
              />
            )}
            {draft.type === 'cost' ? null : (
              <div className="col-span-2">
                <Field
                  label={<Localized id="trips-field-note" />}
                  error={fieldError(error, 'note')}
                >
                  {(props) => (
                    <FreeTextInput
                      {...props}
                      value={draft.note}
                      onChange={(note) => {
                        change({ note });
                      }}
                    />
                  )}
                </Field>
              </div>
            )}
            {payment ? (
              <div className="col-span-2 flex flex-col gap-1">
                <Localized id="trips-sources-title">
                  <h3 className="type-label text-ink-muted" />
                </Localized>
                <Localized id="trips-sources-hint">
                  <p className="type-body text-ink-muted" />
                </Localized>
                <EntrySources
                  partners={trip.partners}
                  value={draft.sources}
                  onChange={(partnerId, source) => {
                    change({ sources: { ...draft.sources, [partnerId]: source } });
                  }}
                />
              </div>
            ) : null}
          </div>
          <div className="flex flex-col gap-3 normal:col-span-2">
            {numbers}
            {active.cash === undefined ? null : <CashNote cash={active.cash} />}
            {active.warnings.length === 0 ? null : (
              <ul className="flex flex-col gap-1">
                {active.warnings.map((item) => (
                  <li
                    key={`${item.code}-${item.field}`}
                    role="status"
                    className="flex items-start gap-2 type-body text-warn"
                  >
                    <TriangleAlert aria-hidden="true" className="mt-1 size-icon-sm shrink-0" />
                    <span>{errorText(item)}</span>
                  </li>
                ))}
              </ul>
            )}
            {active.failed ? (
              <ErrorState
                title={<Localized id="trips-preview-failed" />}
                error={uiErrorOf(active.failure)}
                onRetry={active.retry}
              />
            ) : null}
          </div>
        </div>
        {draft.type === 'purchase' ? (
          // The grid owns its keys (PRD 7.2); the form's own Enter and arrows leave it alone.
          <div data-nav-owns-keys>
            <GoodsLinesGrid
              kind="purchase"
              store={lines}
              calc={calc}
              cellErrors={cellErrors}
              workings={workings}
            />
          </div>
        ) : null}
      </form>
      {flow.cashAfter === null ? null : (
        <CashConfirmDialog
          open={flow.ask}
          onOpenChange={(open) => {
            if (flow.onCashOpenChange(open)) {
              // After a "no" the cursor is on the first money field (PRD FR-6.7).
              window.requestAnimationFrame(() => {
                formRef.current
                  ?.querySelector<HTMLInputElement>('input[inputmode="numeric"]')
                  ?.focus();
              });
            }
          }}
          cashAfter={flow.cashAfter}
          onConfirm={flow.confirmCash}
          onCloseFocus={(event) => {
            // Focus goes where the screen puts it, not back to the button.
            event.preventDefault();
          }}
        />
      )}
      <SaveChangesPrompt
        open={askNew}
        onOpenChange={setAskNew}
        onSave={() => {
          void save().then((saved) => {
            if (saved) setAskNew(false);
          });
        }}
        onLeave={() => {
          setAskNew(false);
          startNew();
        }}
      />
    </div>
  );
}
