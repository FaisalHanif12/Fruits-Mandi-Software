// The sale lines of the selected lot (PRD 12.4, FR-12.1). The grid holds buyer, quantity and rate
// (three fields, three Enters, NFR-USE-6, D3-Q1) and the amounts the backend works out, each with
// its ⓘ. Under it are the details of the line that has focus and the party card of the buyer.
// A switch on the heading shows the lines of ALL lots in one grid with a lot column instead (the
// whole بکری, FR-12.1). Both grids are editable and use the same draft: the whole view reads the
// stores of the lots through `stores.all` and hands every change back to the lot that owns the
// row. The switch is the same one as the summary's, so the two always show the same part.
//
// - A new line's quantity starts as what the lot has left, the rate stays empty with the last
//   rate as grey text (PRD FR-7.52, FR-7.53); Ctrl+D copies the cell above (FR-7.54).
// - Enter on the rate goes to the next line's buyer (the grid makes the line). ↑ on the first line
//   goes back up to the lot (FR-7.17).
// - Typed lines live in the store of the lot, so a language switch loses nothing (F39).

import { Localized, useLocalization } from '@fluent/react';
import { useMemo, type ReactNode } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { EntryGrid, type GridInterceptors } from '../../../shared/grid';
import { useLayoutStep } from '../../../shared/hooks';
import { SegmentedControl } from '../../../shared/ui';
import { allLinesColumns, saleColumns } from '../columns';
import { useBikriDraft } from '../draft-store';
import { focusPukhtaFieldsSoon } from './PukhtaFields';
import { isPukhtaBuyer } from '../draft';
import type { GridView } from '../grid-views';
import { LOT, SALE, type BikriPreview } from '../types';
import { LineDetails } from './LineDetails';

/** A grid with more lines than this fills a fixed height and scrolls inside; shorter ones are as tall as their lines. */
const SHORT_GRID_ROWS = 30;

interface SalesSectionProps {
  lotKey: string;
  lotIndex: number;
  /** The whole-sale view: the lines of every lot in one grid. */
  whole: boolean;
  view: GridView;
  onCardAccount: (accountId: number | null) => void;
  /** The party card drawn under the grid (compact and normal windows). Null: it is drawn elsewhere. */
  card?: ReactNode;
}

export function SalesSection({
  lotKey,
  lotIndex,
  whole,
  view,
  onCardAccount,
  card,
}: SalesSectionProps) {
  const { l10n } = useLocalization();
  const stores = useBikriDraft();
  const step = useLayoutStep();
  const columns = useMemo(() => (whole ? allLinesColumns(step) : saleColumns(step)), [step, whole]);
  // The lines of the selected lot, or the lines of every lot (shown from the stores of the lots).
  const store = whole ? stores.all : stores.salesFor(lotKey);
  const rows = store((state) => state.rows);

  /** Changes the view and keeps the cursor on the same cell of the same line. */
  const switchView = (next: boolean) => {
    const focus = store.getState().focus;
    stores.view.getState().setWhole(next);
    if (focus === null) return;
    for (const lot of stores.lots.getState().rows) {
      const index = stores
        .salesFor(lot.rowId)
        .getState()
        .rows.findIndex((row) => row.rowId === focus.rowId);
      if (index >= 0) {
        stores.focusSaleLine(lot.rowId, index, focus.field);
        return;
      }
    }
  };

  const interceptors = useMemo<GridInterceptors>(
    () => ({
      // ↑ on the first line goes back up to its lot.
      onAboveFirst: () => {
        const index = stores.lots.getState().rows.findIndex((row) => row.rowId === lotKey);
        stores.lots.getState().setPendingFocus({ rowIndex: Math.max(index, 0), field: LOT.seller });
        return true;
      },
      // A one-time buyer: Enter goes on to his name and phone first (PRD FR-13.2).
      onEnter: (cell) => {
        const row = stores.all.getState().rows.find((candidate) => candidate.rowId === cell.rowId);
        if (cell.field !== SALE.buyer || row === undefined || !isPukhtaBuyer(row)) return false;
        focusPukhtaFieldsSoon(cell.rowId);
        return true;
      },
    }),
    [stores, lotKey],
  );

  return (
    <section aria-labelledby="bikri-sales-heading" className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        {whole ? (
          <Localized id="bikri-sales-title-all">
            <h2 id="bikri-sales-heading" className="type-section text-ink" />
          </Localized>
        ) : (
          <Localized id="bikri-sales-title" vars={{ lot: String(lotIndex + 1) }}>
            <h2 id="bikri-sales-heading" className="type-section text-ink" />
          </Localized>
        )}
        <SegmentedControl
          groupLabel={l10n.getString('bikri-sales-view-label')}
          value={whole ? 'whole' : 'lot'}
          onChange={(next) => {
            switchView(next === 'whole');
          }}
          options={[
            {
              value: 'lot',
              label: l10n.getString('bikri-summary-lot', { lot: String(lotIndex + 1) }),
            },
            { value: 'whole', label: l10n.getString('bikri-summary-whole') },
          ]}
        />
      </div>
      <div className={rows.length > SHORT_GRID_ROWS ? 'h-tree-view' : undefined}>
        <EntryGrid
          key={whole ? 'whole' : lotKey}
          columns={columns}
          store={store}
          labelId="bikri-sales-label"
          calc={view.calc}
          workings={view.workings}
          cellErrors={view.errors}
          cellWarnings={view.warnings}
          interceptors={interceptors}
          onCardAccount={onCardAccount}
          copyAbove
          fit
          heightMode={rows.length > SHORT_GRID_ROWS ? 'fill' : 'auto'}
        />
      </div>
      {card}
    </section>
  );
}

interface ActiveLineProps {
  lotKey: string;
  lotIndex: number;
  preview: BikriPreview | undefined;
  error: UiError | null;
  usesWeight: boolean;
}

/** The details of the sale line that has focus, under the summary (PRD 12.4, FR-11.10). */
export function ActiveLine({ lotKey, lotIndex, preview, error, usesWeight }: ActiveLineProps) {
  const stores = useBikriDraft();
  const store = stores.salesFor(lotKey);
  const rows = store((state) => state.rows);
  const focus = store((state) => state.focus);
  const lotRow = stores.lots((state) => state.rows).find((row) => row.rowId === lotKey);
  const activeIndex = Math.max(
    rows.findIndex((row) => row.rowId === focus?.rowId),
    0,
  );
  const active = rows[activeIndex];
  if (active === undefined) return null;
  const lotPreview = preview?.lots.find((lot) => lot.key === lotKey);
  return (
    <LineDetails
      store={store}
      row={active}
      seller={lotRow?.picked?.[LOT.seller]}
      item={lotRow?.picked?.[LOT.item]}
      lotIndex={lotIndex}
      lineIndex={activeIndex}
      line={lotPreview?.lines.find((line) => line.key === active.rowId)}
      allowOverride={preview?.allow_line_override ?? false}
      usesWeight={usesWeight}
      error={error}
      onDone={() => {
        stores.focusSaleLine(lotKey, activeIndex, SALE.qty);
      }}
    />
  );
}
