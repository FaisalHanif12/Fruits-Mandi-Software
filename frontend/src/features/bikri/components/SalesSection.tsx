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
import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import type { UiError } from '../../../bindings/UiError';
import type { CardChukta } from '../../../shared/accounts';
import { EntryGrid, type EntryRow, type GridInterceptors } from '../../../shared/grid';
import { useLatest, useLayoutStep } from '../../../shared/hooks';
import { SegmentedControl } from '../../../shared/ui';
import { useOverrideSettings } from '../../settings';
import { allLinesColumns, saleColumns } from '../columns';
import { useBikriDraft } from '../draft-store';
import { useKeepCardInView } from '../hooks/useKeepCardInView';
import { lotLabelVars, useLotWho } from '../lot-name';
import type { GridView } from '../grid-views';
import { LOT, SALE, type BikriPreview } from '../types';
import { LineDetails } from './LineDetails';
import { MissingNotes, QtyNotes } from './QtyNotes';

/** A grid with more lines than this fills a fixed height and scrolls inside; shorter ones are as tall as their lines. */
const SHORT_GRID_ROWS = 30;

interface SalesSectionProps {
  lotKey: string;
  lotIndex: number;
  /** The whole-sale view: the lines of every lot in one grid. */
  whole: boolean;
  view: GridView;
  onCardAccount: (accountId: number | null, chukta?: CardChukta | null) => void;
  /** The party card drawn under the grid (compact and normal windows). Null: it is drawn elsewhere. */
  card?: ReactNode;
  /**
   * A later sale or the edit of one has a single lot, so there is nothing to switch between:
   * the heading says what the lines are and the lot / whole switch is not drawn (plan 16).
   */
  single?: { title: ReactNode } | undefined;
  /** Cells the user changed in a posting open for edit (plan 16). */
  cellChanged?: Record<string, Record<string, string>> | undefined;
  /** The cells an earlier edit changed, with the words of that edit as their tip (plan 44). */
  cellMarks?: Record<string, Record<string, string>> | undefined;
  /** Can Ctrl+Delete remove this line? Default: yes. */
  canRemoveRow?: ((row: EntryRow) => boolean) | undefined;
}

export function SalesSection({
  lotKey,
  lotIndex,
  whole,
  view,
  onCardAccount,
  card,
  single,
  cellChanged,
  cellMarks,
  canRemoveRow,
}: SalesSectionProps) {
  const { l10n } = useLocalization();
  const stores = useBikriDraft();
  const step = useLayoutStep();
  const lotWho = useLotWho();
  // The lot is named by its seller and item, not by its number alone (PRD FR-12.6).
  const lotRow = stores.lots((state) => state.rows).find((row) => row.rowId === lotKey);
  const lotLabel = lotLabelVars(lotIndex + 1, lotWho(lotRow));
  const baseColumns = useMemo(
    () => (whole ? allLinesColumns(step) : saleColumns(step)),
    [step, whole],
  );
  // The party card under the grid is kept in view while the user types in the lines (PRD FR-7.38).
  const cardBox = useRef<HTMLDivElement>(null);
  const [region, setRegion] = useState<HTMLElement | null>(null);
  useKeepCardInView(cardBox, region);
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

  // The lines the backend refused for a quantity over the lot, read when a key is pressed or a cell is
  // asked to be typed in.
  const holdsNow = useLatest(view.holds);
  // The quantity text each hold was sent back for. A hold is for that text: the moment the text changes the
  // line is free again (its cells take input and Enter goes on), until the next answer says again.
  const sentBack = useRef(new Map<string, string>());
  const heldFieldOf = useMemo(
    () =>
      (rowId: string): string | null => {
        const held = holdsNow()[rowId];
        if (held === undefined) return null;
        const text =
          store.getState().rows.find((row) => row.rowId === rowId)?.typed[SALE.qty] ?? '';
        const known = sentBack.current.get(rowId);
        return known === undefined || known === text ? held : null;
      },
    [holdsNow, store],
  );
  // The amounts the backend decided for cells it closed (the paid amount of a buyer who must pay in full,
  // PRD FR-12.44), read from the newest answer when a cell is asked to be typed in or Enter looks for the
  // next stop.
  const calcNow = useLatest(view.calc);
  // On a line the backend refused for its quantity only the quantity takes input until it is fixed: the
  // other cells of the line do not start typing (D8 Bug 1, B2). A cell the backend closed does not take
  // input and Enter skips it (plan 65). Read from the newest answer when asked.
  const columns = useMemo(
    () =>
      baseColumns.map((column) => ({
        ...column,
        editable: (row: EntryRow) => {
          const held = heldFieldOf(row.rowId);
          const closedBy = column.fixedFromCalc;
          const closed = closedBy !== undefined && (calcNow()[row.rowId]?.[closedBy] ?? '') !== '';
          return (
            !closed && (held === null || held === column.field) && (column.editable?.(row) ?? true)
          );
        },
      })),
    [baseColumns, heldFieldOf, calcNow],
  );

  // What the backend refused, read when Enter is pressed, so a new answer never rebuilds the keys.
  const errorsNow = useLatest(view.errors);
  const interceptors = useMemo<GridInterceptors>(
    () => ({
      // A refused line keeps the cursor on its quantity until it is fixed (D8 Bug 1, B2).
      heldField: heldFieldOf,
      // ↑ on the first line goes back up to its lot.
      onAboveFirst: () => {
        const index = stores.lots.getState().rows.findIndex((row) => row.rowId === lotKey);
        stores.lots.getState().setPendingFocus({ rowIndex: Math.max(index, 0), field: LOT.seller });
        return true;
      },
      // A line with something refused keeps the cursor until it is fixed. A چکتا buyer needs no
      // stop of his own: he is one chosen record of the buyer box, so Enter goes on as usual
      // (PRD FR-12.5, FR-13.5).
      onEnter: (cell, enter) => {
        // A quantity the backend refused (more than the lot has left, FR-12.6) is not left with Enter
        // until it is fixed: the cursor stays on it. Only when the text was not just changed, because
        // the refusal on view belongs to the old text (D8 Bug 1, B2).
        if (
          cell.field === SALE.qty &&
          enter?.changed !== true &&
          errorsNow()[cell.rowId]?.[SALE.qty] !== undefined
        ) {
          return true;
        }
        return false;
      },
    }),
    [stores, lotKey, errorsNow, heldFieldOf],
  );

  // A quantity the backend refused (more than the lot has left): once for each text it refused, the cursor
  // goes back to that line's quantity with the text selected, whatever cell it had moved to, if it is in
  // the lines here. The screen only reacts to the backend's error; it judges nothing (D8 Bug 1, B2).
  useEffect(() => {
    const holds = view.holds;
    // A text the cursor was sent back for is remembered while the text stays the same, also while the
    // answer for a later change (the rate, say) is on its way and the hold blinks off. Once the text is
    // another, the memory goes, so the same wrong text typed again later sends the cursor back again.
    for (const [rowId, known] of sentBack.current) {
      const now = store.getState().rows.find((row) => row.rowId === rowId)?.typed[SALE.qty];
      if (now !== known) sentBack.current.delete(rowId);
    }
    for (const rowId of Object.keys(holds)) {
      const index = store.getState().rows.findIndex((row) => row.rowId === rowId);
      const row = store.getState().rows[index];
      if (row === undefined) continue;
      const text = row.typed[SALE.qty] ?? '';
      if (sentBack.current.get(rowId) === text) continue;
      sentBack.current.set(rowId, text);
      const active = document.activeElement;
      const here =
        region !== null &&
        active instanceof Element &&
        (region.contains(active) || active.closest('.ag-custom-component-popup') !== null);
      if (!here) continue;
      store.getState().setPendingFocus({ rowIndex: index, field: SALE.qty, edit: true });
    }
  }, [view.holds, store, region]);

  return (
    <section ref={setRegion} aria-labelledby="bikri-sales-heading" className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        {single !== undefined ? (
          <h2 id="bikri-sales-heading" className="type-section text-ink">
            {single.title}
          </h2>
        ) : whole ? (
          <Localized id="bikri-sales-title-all">
            <h2 id="bikri-sales-heading" className="type-section text-ink" />
          </Localized>
        ) : (
          <Localized id="bikri-sales-title" vars={{ ...lotLabel }}>
            <h2 id="bikri-sales-heading" className="type-section text-ink" />
          </Localized>
        )}
        {single !== undefined ? null : (
          <SegmentedControl
            groupLabel={l10n.getString('bikri-sales-view-label')}
            value={whole ? 'whole' : 'lot'}
            onChange={(next) => {
              switchView(next === 'whole');
            }}
            options={[
              {
                value: 'lot',
                label: l10n.getString('bikri-summary-lot', { ...lotLabel }),
              },
              { value: 'whole', label: l10n.getString('bikri-summary-whole') },
            ]}
          />
        )}
      </div>
      <div className={rows.length > SHORT_GRID_ROWS ? 'h-tree-view' : undefined}>
        <EntryGrid
          key={whole ? 'whole' : lotKey}
          columns={columns}
          store={store}
          labelId="bikri-sales-label"
          calc={view.calc}
          workings={view.workings}
          cellManual={view.manual}
          cellErrors={view.errors}
          cellWarnings={view.warnings}
          {...(cellChanged === undefined ? {} : { cellChanged })}
          {...(cellMarks === undefined ? {} : { cellMarks })}
          {...(canRemoveRow === undefined ? {} : { canRemoveRow })}
          interceptors={interceptors}
          onCardAccount={onCardAccount}
          copyAbove
          fit
          heightMode={rows.length > SHORT_GRID_ROWS ? 'fill' : 'auto'}
        />
      </div>
      <QtyNotes notes={view.qtyNotes} whole={whole} />
      <MissingNotes notes={view.missingNotes} whole={whole} />
      {card === null || card === undefined ? null : <div ref={cardBox}>{card}</div>}
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
  // The switch is the Owner's setting, read the same way as in Settings. The preview only knows it
  // once a line has been previewed, so it cannot say "off" for an empty line (plan 46, F2). While
  // the setting is not known yet, nothing is said to be off.
  const overrides = useOverrideSettings();
  const allowOverride = overrides.data?.allow_line_override ?? preview?.allow_line_override ?? null;
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
      allowOverride={allowOverride}
      usesWeight={usesWeight}
      error={error}
      onDone={() => {
        stores.focusSaleLine(lotKey, activeIndex, SALE.qty);
      }}
    />
  );
}
