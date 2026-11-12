// The lots of the بکری (PRD 12.3): one row for each seller's goods of one item. The grid holds the
// cells typed on most lots, in the order of Enter: seller, item, packing, quantity received,
// freight, labour, munshiana, نقد. The rest of the lot is behind "مزید" (LotMore).
//
// - Enter on the last cell goes on to the sale lines of that lot (PRD FR-12.1); ↓ on the last lot
//   adds a lot; the round "+" under the grid does the same (it is reached by ↓ from the last row's trash
//   cell, and stops at 50 lots, NFR-SCL-3). Ctrl+Delete, or the button at the end
//   of its row, removes a lot, after asking when it has sale lines (plan 43).
// - The lot that has focus is the selected lot: its sale lines are below.
// - The seller of a new lot, and the item, packing and مارکہ of a seller, are filled by the app
//   and are Enter stops like the rest (useDraftFills).

import { Localized } from '@fluent/react';
import { useEffect, useMemo, type ReactNode } from 'react';
import { groupDigits, useGrouping } from '../../../shared/format';
import {
  AddRowButton,
  EntryGrid,
  addRowAndFocus,
  useAddRowLink,
  type EntryRow,
  type GridInterceptors,
  type RemoveQuestion,
} from '../../../shared/grid';
import { useLayoutStep } from '../../../shared/hooks';
import { lotColumns } from '../columns';
import { isGardenLot, isOwnLot } from '../draft';
import { useBikriDraft } from '../draft-store';
import type { GridView } from '../grid-views';
import { MAX_LOTS } from '../limits';
import { LOT, SALE } from '../types';
import { LotMoreToggle } from './LotMore';
import { focusOwnPurchaseSoon } from './OwnLotPanel';

/**
 * The last cell of a lot row that Enter stops at. An own goods lot and a garden's lot have no نقد
 * cell to type in.
 */
function lastStopOf(row: EntryRow | undefined): string {
  return row !== undefined && (isOwnLot(row) || isGardenLot(row)) ? LOT.munshiana : LOT.cash;
}

interface LotsSectionProps {
  view: GridView;
  onCardAccount: (accountId: number | null) => void;
  /** The "more" panel of the selected lot cannot be closed now (a problem is on it). */
  moreForced: boolean;
  /** Cells the user changed in a posting open for edit (plan 16), marked in the grid. */
  cellChanged?: Record<string, Record<string, string>> | undefined;
  /** The cells an earlier edit changed, with the words of that edit as their tip (plan 44). */
  cellMarks?: Record<string, Record<string, string>> | undefined;
  /** Can Ctrl+Delete remove this lot? A saved lot of an edit cannot be removed. */
  canRemoveRow?: ((row: EntryRow) => boolean) | undefined;
  /** A note between the title and the grid, for example where some of the lots came from. */
  note?: ReactNode;
}

export function LotsSection({
  view,
  onCardAccount,
  moreForced,
  cellChanged,
  cellMarks,
  canRemoveRow,
  note,
}: LotsSectionProps) {
  const stores = useBikriDraft();
  const step = useLayoutStep();
  // The arrows reach the trash cell of each lot, and from the last one the "+" under the grid.
  const addLink = useAddRowLink();
  const columns = useMemo(() => lotColumns(step), [step]);
  const rows = stores.lots((state) => state.rows);
  const focus = stores.lots((state) => state.focus);
  const selected = stores.view((state) => state.selectedLot);

  // The lot that has focus is the selected lot.
  useEffect(() => {
    if (focus !== null) stores.view.getState().select(focus.rowId);
  }, [focus, stores]);
  // A removed lot cannot stay selected.
  useEffect(() => {
    if (selected === null || !rows.some((row) => row.rowId === selected)) {
      stores.view.getState().select(rows[0]?.rowId ?? null);
    }
  }, [rows, selected, stores]);

  // A lot with sale lines takes them with it: the user is asked first, in one sentence (plan 43 R3). A
  // blank line (no buyer and no rate) is not counted: it is only an empty row (PRD FR-7.52).
  const removeQuestion = useMemo(
    () =>
      (row: EntryRow): RemoveQuestion | null => {
        const count = stores
          .salesFor(row.rowId)
          .getState()
          .rows.filter(
            (line) =>
              line.picked?.[SALE.buyer] !== undefined ||
              (line.typed[SALE.rate] ?? '').trim() !== '',
          ).length;
        return count === 0
          ? null
          : {
              messageId: 'bikri-lot-remove-question',
              titleId: 'bikri-lot-remove-title',
              vars: { count },
            };
      },
    [stores],
  );

  const interceptors = useMemo<GridInterceptors>(
    () => ({
      // Enter on the last cell of a lot goes on to its sale lines (PRD FR-12.1).
      onEnter: (cell) => {
        const row = stores.lots.getState().rows.find((candidate) => candidate.rowId === cell.rowId);
        // An own goods lot: Enter goes on to the choice of its purchase until it has one (FR-12.26).
        if (
          cell.field === LOT.seller &&
          row !== undefined &&
          isOwnLot(row) &&
          (row.typed[LOT.purchase] ?? '') === '' &&
          (row.typed[LOT.lotId] ?? '') === ''
        ) {
          stores.view.getState().select(cell.rowId);
          focusOwnPurchaseSoon(cell.rowId);
          return true;
        }
        if (cell.field !== lastStopOf(row)) return false;
        stores.view.getState().select(cell.rowId);
        stores.focusSaleLine(cell.rowId, 0, SALE.buyer);
        return true;
      },
    }),
    [stores],
  );

  // A new lot goes through the grid's one new-row path, the same as ↓ on the last lot (PRD FR-7.17).
  const addLot = () => {
    addRowAndFocus(stores.lots, LOT.seller);
  };
  // At the limit of lots in one بکری (NFR-SCL-3) no lot is offered, and the words say why.
  const grouping = useGrouping();
  const lotsFull = rows.length >= MAX_LOTS;
  const limit = lotsFull
    ? {
        messageId: 'bikri-lots-limit',
        vars: { max: groupDigits(String(MAX_LOTS), grouping) },
      }
    : undefined;

  return (
    <section aria-labelledby="bikri-lots-heading" className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Localized id="bikri-lots-title">
          <h2 id="bikri-lots-heading" className="type-section text-ink" />
        </Localized>
        <div className="flex flex-wrap gap-2">
          <LotMoreToggle
            lotIndex={Math.max(
              rows.findIndex((row) => row.rowId === selected),
              0,
            )}
            forced={moreForced}
          />
        </div>
      </div>
      {note}
      <div className="flex flex-col gap-1">
        <EntryGrid
          columns={columns}
          store={stores.lots}
          labelId="bikri-lots-label"
          calc={view.calc}
          workings={view.workings}
          cellErrors={view.errors}
          cellWarnings={view.warnings}
          {...(cellChanged === undefined ? {} : { cellChanged })}
          {...(cellMarks === undefined ? {} : { cellMarks })}
          {...(canRemoveRow === undefined ? {} : { canRemoveRow })}
          removeQuestion={removeQuestion}
          interceptors={interceptors}
          onCardAccount={onCardAccount}
          addLink={addLink}
          fit
          heightMode="auto"
        />
        <AddRowButton labelId="bikri-lots-add-row" onAdd={addLot} limit={limit} link={addLink} />
      </div>
    </section>
  );
}
