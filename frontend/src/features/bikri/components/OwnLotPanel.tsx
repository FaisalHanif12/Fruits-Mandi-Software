// A lot of trip goods (PRD 12.3, FR-12.26). When the seller of a lot is اپنا مال, the goods come
// from one purchase of an open trip with stock left. This panel sits under the lots grid for the
// selected lot and holds: the mark "Trip goods", the choice of the purchase with its trip, what
// the purchase has (bought, sold, written off, left, cost left), and, once the backend can work
// the lot out, what the lot gives: the commission (firm income), the seller credit that goes to
// the trip's partners with each one's part (C25), the cost of the goods sold (C21) and the result.
// Every number is the backend's, with the working behind its ⓘ (Code Standards F42).
//
// - Choosing a purchase fills the item and packing of the lot from it and the quantity with what
//   the purchase has left, like values typed, in the visible cells (NFR-USE-27, Code Standards F43).
//   The item and packing follow the purchase, so they are shown and not typed on this lot.
// - The lot must sell exactly the quantity it takes from the purchase (ERR_OWN_LOT_NOT_FULLY_SOLD,
//   shown at the quantity cell); the rest of the purchase stays in stock for another بکری.
// - A saved lot (an edit) keeps its purchase: the choice is shown and not changed.
// - Cash paid and the advance tick are hidden on this lot, and freight starts at 0 (FR-12.26).
//
// The value is text on the lot's row (the id of the purchase), so a language switch keeps it (F39).

import { Localized } from '@fluent/react';
import { PackageOpen, TriangleAlert } from 'lucide-react';
import { useMemo } from 'react';
import type { LotPreview } from '../../../bindings/LotPreview';
import type { RecordRef } from '../../../bindings/RecordRef';
import type { UiError } from '../../../bindings/UiError';
import type { EntryRow, PickedRecord } from '../../../shared/grid';
import { displayName, useLanguage } from '../../../shared/i18n';
import { isNegativePaisa } from '../../../shared/format';
import {
  CalcInfo,
  Card,
  Fact,
  Field,
  Num,
  Select,
  ShareAmounts,
  fieldError,
} from '../../../shared/ui';
import {
  purchaseNumberText,
  usePurchase,
  usePurchasesList,
  type PurchaseRow,
} from '../../purchases';
import { tripNumberText } from '../../trips';
import { useLocalization } from '@fluent/react';
import { useBikriDraft } from '../draft-store';
import { LOT } from '../types';

interface OwnLotPanelProps {
  row: EntryRow;
  lotIndex: number;
  error: UiError | null;
  /** The backend's answer for this lot, or undefined while it cannot be worked out. */
  lot: LotPreview | undefined;
}

/** How many tries (one for each frame, about 16 ms) the purchase choice gets to appear. */
const FOCUS_TRIES = 40;

/** Puts focus on the purchase choice of a lot, if it is on screen. */
export function focusOwnPurchase(rowId: string): boolean {
  const trigger = document.querySelector<HTMLElement>(
    `[data-own-lot-for="${CSS.escape(rowId)}"] button[role="combobox"]`,
  );
  trigger?.focus();
  return trigger !== null && document.activeElement === trigger;
}

/**
 * Like `focusOwnPurchase`, for the moment right after اپنا مال was chosen as the seller: the panel
 * is drawn a moment later, so it tries again for each frame until it is there.
 */
export function focusOwnPurchaseSoon(rowId: string): void {
  let tries = 0;
  const attempt = () => {
    if (focusOwnPurchase(rowId)) return;
    tries += 1;
    if (tries < FOCUS_TRIES) window.requestAnimationFrame(attempt);
  };
  window.setTimeout(attempt, 0);
}

function pickedOfRef(ref: RecordRef): PickedRecord {
  return { id: ref.id, code: ref.code, name_ur: ref.name_ur, name_en: ref.name_en };
}

export function OwnLotPanel({ row, lotIndex, error, lot }: OwnLotPanelProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const stores = useBikriDraft();
  const chosenText = row.typed[LOT.purchase] ?? '';
  const chosenId =
    Number.isSafeInteger(Number(chosenText)) && Number(chosenText) > 0 ? Number(chosenText) : null;
  // A saved lot of an edit keeps its purchase.
  const locked = (row.typed[LOT.lotId] ?? '') !== '';

  // The purchases with goods left, and the one chosen (which may have nothing left: an edit).
  const inStock = usePurchasesList({
    trip_id: null,
    from: null,
    to: null,
    item_id: null,
    state: 'in_stock',
    offset: 0,
    limit: 0,
  });
  const chosen = usePurchase(chosenId);
  const options = useMemo(() => {
    const rows = inStock.data?.rows ?? [];
    const found = chosen.data;
    return found !== undefined && !rows.some((candidate) => candidate.id === found.id)
      ? [found, ...rows]
      : rows;
  }, [inStock.data, chosen.data]);
  const current = options.find((candidate) => candidate.id === chosenId) ?? null;

  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);
  const label = (purchase: PurchaseRow) =>
    l10n.getString('bikri-own-option', {
      number: purchaseNumberText(purchase.number, say),
      trip: tripNumberText(purchase.trip.number, say),
      item: displayName(purchase.item, language),
      packing: displayName(purchase.packing, language),
      qty_left: String(purchase.qty_left),
    });

  const choose = (value: string) => {
    const purchase = options.find((candidate) => String(candidate.id) === value);
    if (purchase === undefined) return;
    stores.lots.getState().patchRow(row.rowId, {
      typed: { [LOT.purchase]: value, [LOT.qty]: String(purchase.qty_left) },
      picked: {
        [LOT.item]: pickedOfRef(purchase.item),
        [LOT.packing]: pickedOfRef(purchase.packing),
      },
    });
  };
  // The quantity is filled, so it is the next stop (PRD FR-7.14). It is asked for once the list has
  // closed: the list would otherwise give focus back to the purchase choice afterwards.
  const focusQty = () => {
    stores.lots.getState().setPendingFocus({ rowIndex: lotIndex, field: LOT.qty });
  };

  return (
    <Card
      compact
      title={
        <span className="inline-flex items-center gap-2">
          <PackageOpen aria-hidden="true" className="size-icon-md" />
          <Localized id="bikri-own-title" />
        </span>
      }
    >
      <div
        data-own-lot-for={row.rowId}
        className="grid grid-cols-1 gap-x-4 gap-y-3 normal:grid-cols-2"
      >
        <Field
          label={<Localized id="bikri-own-purchase" />}
          hint={<Localized id="bikri-own-purchase-hint" />}
          readOnly={locked}
          error={fieldError(error, `lots[${String(lotIndex)}].purchase_id`)}
        >
          {(control) => (
            <Select
              {...control}
              value={current === null ? undefined : String(current.id)}
              onChange={choose}
              afterChoose={focusQty}
              options={options.map((purchase) => ({
                value: String(purchase.id),
                label: label(purchase),
              }))}
            />
          )}
        </Field>
        {inStock.isError && options.length === 0 ? (
          <Localized id="bikri-own-load-failed">
            <p className="type-body text-due" role="alert" />
          </Localized>
        ) : !inStock.isPending && options.length === 0 ? (
          <Localized id="bikri-own-none">
            <p className="type-body text-ink-muted" role="status" />
          </Localized>
        ) : null}
      </div>
      {current === null ? null : (
        <dl className="grid grid-cols-3 gap-x-4 gap-y-3 normal:grid-cols-6">
          <Fact label={<Localized id="bikri-stock-own-trip" />}>
            {tripNumberText(current.trip.number, say)}
          </Fact>
          <Fact label={<Localized id="purchases-col-qty" />}>
            <Num>{String(current.qty)}</Num>
          </Fact>
          <Fact label={<Localized id="purchases-lot-qty-sold" />}>
            <Num>{String(current.qty_sold)}</Num>
          </Fact>
          <Fact label={<Localized id="purchases-detail-written-off" />}>
            <Num>{String(current.qty_written_off)}</Num>
          </Fact>
          <Fact label={<Localized id="purchases-col-left" />}>
            <Num>{String(current.qty_left)}</Num>
          </Fact>
          <Fact label={<Localized id="purchases-col-cost-left" />}>
            <Num paisa={current.cost_left_paisa} />
          </Fact>
        </dl>
      )}
      <OwnResult lot={lot} />
    </Card>
  );
}

/** What the lot gives, from the backend's preview (PRD FR-12.26, C25, C21, FR-12.27). */
function OwnResult({ lot }: { lot: LotPreview | undefined }) {
  const { l10n } = useLocalization();
  const own = lot?.own ?? null;
  if (own === null) {
    return (
      <Localized id="bikri-own-numbers-pending">
        <p className="type-body text-ink-muted" />
      </Localized>
    );
  }
  const loss = isNegativePaisa(own.result_paisa);
  const totals = lot?.totals ?? null;
  return (
    <div className="flex flex-col gap-1">
      <dl className="flex flex-col">
        {totals === null ? null : (
          <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
            <Localized id="bikri-own-commission">
              <dt />
            </Localized>
            <dd className="flex items-center gap-1">
              <Num paisa={totals.seller_comm} />
              <CalcInfo working={totals.workings.seller_comm} inCell />
            </dd>
          </div>
        )}
        <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-label text-ink">
          <Localized id="bikri-own-credit">
            <dt />
          </Localized>
          <dd>
            <Num paisa={own.credit_paisa} />
          </dd>
        </div>
      </dl>
      <ShareAmounts credits={own.credits} label={l10n.getString('bikri-own-credit')} />
      <dl className="flex flex-col">
        <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
          <Localized id="purchases-detail-cost-sold">
            <dt />
          </Localized>
          <dd className="flex items-center gap-1">
            <Num paisa={own.cost_paisa} />
            <CalcInfo working={own.cost_working} inCell />
          </dd>
        </div>
        <div
          className={`flex items-center justify-between gap-3 border-b border-border py-1 type-label ${loss ? 'text-due' : 'text-ink'}`}
        >
          <Localized id={loss ? 'glossary-tijarati-nuqsan' : 'glossary-tijarati-munafa'}>
            <dt />
          </Localized>
          <dd className="flex items-center gap-1">
            <Num paisa={own.result_paisa} />
            <CalcInfo working={own.result_working} inCell />
          </dd>
        </div>
      </dl>
      {loss ? (
        <p className="flex items-center gap-2 type-body text-due">
          <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
          <Localized id="purchases-loss-note">
            <span />
          </Localized>
        </p>
      ) : null}
    </div>
  );
}
