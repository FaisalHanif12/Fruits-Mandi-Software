// The lots of the بکری (PRD 12.3): one row for each seller's goods of one item. The grid holds the
// cells typed on most lots, in the order of Enter: seller, item, packing, quantity received,
// freight, labour, munshiana, نقد. The rest of the lot is behind "مزید" (LotMore).
//
// - Enter on the last cell goes on to the sale lines of that lot (PRD FR-12.1); ↓ on the last lot
//   adds a lot; "لاٹ شامل کریں" does the same with the mouse. Ctrl+Delete removes a lot.
// - The lot that has focus is the selected lot: its sale lines are below.
// - The seller of a new lot, and the item, packing and مارکہ of a seller, are filled by the app
//   and are Enter stops like the rest (useDraftFills).

import { Localized } from '@fluent/react';
import { Plus } from 'lucide-react';
import { useEffect, useMemo } from 'react';
import { EntryGrid, type EntryRow, type GridInterceptors } from '../../../shared/grid';
import { useLayoutStep } from '../../../shared/hooks';
import { Button } from '../../../shared/ui';
import { lotColumns } from '../columns';
import { isPukhtaSeller } from '../draft';
import { useBikriDraft } from '../draft-store';
import type { GridView } from '../grid-views';
import { LOT, SALE } from '../types';
import { LotMoreToggle } from './LotMore';
import { focusPukhtaFieldsSoon } from './PukhtaFields';

/** The last cell of a lot row that Enter stops at. */
const LAST_STOP = LOT.cash;

interface LotsSectionProps {
  view: GridView;
  onCardAccount: (accountId: number | null) => void;
  /** The "more" panel of the selected lot cannot be closed now (a problem is on it). */
  moreForced: boolean;
  /** Cells the user changed in a posting open for edit (plan 16), marked in the grid. */
  cellChanged?: Record<string, Record<string, string>> | undefined;
  /** Can Ctrl+Delete remove this lot? A saved lot of an edit cannot be removed. */
  canRemoveRow?: ((row: EntryRow) => boolean) | undefined;
}

export function LotsSection({
  view,
  onCardAccount,
  moreForced,
  cellChanged,
  canRemoveRow,
}: LotsSectionProps) {
  const stores = useBikriDraft();
  const step = useLayoutStep();
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

  const interceptors = useMemo<GridInterceptors>(
    () => ({
      // Enter on the last cell of a lot goes on to its sale lines (PRD FR-12.1).
      onEnter: (cell) => {
        // A one-time seller: Enter goes on to his name and phone first (PRD FR-13.2).
        const row = stores.lots.getState().rows.find((candidate) => candidate.rowId === cell.rowId);
        if (cell.field === LOT.seller && row !== undefined && isPukhtaSeller(row)) {
          stores.view.getState().select(cell.rowId);
          focusPukhtaFieldsSoon(cell.rowId);
          return true;
        }
        if (cell.field !== LAST_STOP) return false;
        stores.view.getState().select(cell.rowId);
        stores.focusSaleLine(cell.rowId, 0, SALE.buyer);
        return true;
      },
    }),
    [stores],
  );

  const addLot = () => {
    stores.lots.getState().addRow();
    stores.lots.getState().setPendingFocus({ rowIndex: 'last', field: LOT.seller });
  };

  return (
    <section aria-labelledby="bikri-lots-heading" className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Localized id="bikri-lots-title">
          <h2 id="bikri-lots-heading" className="type-section text-ink" />
        </Localized>
        <div className="flex flex-wrap gap-2">
          <Button onClick={addLot}>
            <Plus aria-hidden="true" className="size-icon-md" />
            <Localized id="bikri-lots-add" />
          </Button>
          <LotMoreToggle
            lotIndex={Math.max(
              rows.findIndex((row) => row.rowId === selected),
              0,
            )}
            forced={moreForced}
          />
        </div>
      </div>
      <EntryGrid
        columns={columns}
        store={stores.lots}
        labelId="bikri-lots-label"
        calc={view.calc}
        workings={view.workings}
        cellErrors={view.errors}
        cellWarnings={view.warnings}
        {...(cellChanged === undefined ? {} : { cellChanged })}
        {...(canRemoveRow === undefined ? {} : { canRemoveRow })}
        interceptors={interceptors}
        onCardAccount={onCardAccount}
        fit
        heightMode="auto"
      />
    </section>
  );
}
