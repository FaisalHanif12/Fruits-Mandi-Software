# Entry grid: code sketches for the shared setup

These sketches show the shape of `frontend/src/shared/grid/`. They are written once, in build step 4, then reused by every entry grid. Check API names against the AG Grid version pinned in `pnpm-lock.yaml`. AG Grid renames options between major versions, and its docs are the final word.

## Grid wrapper

```tsx
// shared/grid/EntryGrid.tsx
export function EntryGrid<Row extends EntryRow>({ columns, rows, onCommit, focus }: EntryGridProps<Row>) {
  const { language } = useLanguage();
  const apiRef = useRef<GridApi<Row> | null>(null);

  return (
    <AgGridReact<Row>
      key={language}                       // enableRtl is set only at creation, so rebuild on switch
      enableRtl={language === 'ur'}
      rowData={rows}
      columnDefs={toColumnDefs(columns)}   // maps our column types to colDefs
      columnTypes={entryColumnTypes}
      getRowId={(p) => p.data.rowId}
      singleClickEdit={false}
      stopEditingWhenCellsLoseFocus
      defaultColDef={{ suppressKeyboardEvent: suppressGridKeys, sortable: false }}
      navigateToNextCell={arrowNavigation}  // arrows follow what the user sees
      tabToNextCell={tabNavigation}
      onCellKeyDown={handleEntryKeys}       // Enter order, Ctrl+D, Ctrl+Delete, ↓ on last row
      onCellValueChanged={(e) => onCommit(e.data.rowId, e.colDef.field, e.newValue)}
      onGridReady={(e) => { apiRef.current = e.api; restoreFocus(e.api, focus); }}
      pinnedBottomRowData={/* totals from the preview */}
    />
  );
}
```

`EntryGrid` is the only file that imports `AgGridReact` for entry screens.

## Keys the grid itself must not handle

```ts
// Returning true tells AG Grid to ignore the key, so our handler decides.
export function suppressGridKeys(p: SuppressKeyboardEventParams): boolean {
  const k = p.event.key;
  if (k === 'F2') return true;                                   // F2 is "New" in this app
  if (k === 'Enter') return true;                                // our Enter order, not AG Grid's
  if (p.event.ctrlKey && (k === 'd' || k === 'Delete')) return true;
  if (p.editing && (k.startsWith('Arrow'))) return true;          // our cell editors handle arrows (below)
  return false;
}
```

## Enter: next editable cell in reading order

```ts
export function nextEnterStop(api: GridApi, from: CellPosition): CellPosition {
  const cols = api.getAllDisplayedColumns().filter(isEnterStopColumn); // enterStop: true, calc excluded
  let rowIndex = from.rowIndex;
  let i = cols.findIndex((c) => c.getColId() === from.column.getColId());
  for (;;) {
    i += 1;
    if (i >= cols.length) { i = 0; rowIndex += 1; ensureRowExists(rowIndex); } // store.addRow if needed
    const node = api.getDisplayedRowAtIndex(rowIndex);
    const col = cols[i];
    if (node && col && col.isCellEditable(node)) return { rowIndex, column: col, rowPinned: null };
  }
}
```

"Reading order" is the order of the columns as the user sees them. `getAllDisplayedColumns()` already reflects the right-to-left layout. Do not reverse it by hand.

Editability is asked per cell (`column.isCellEditable(node)`), because a buyer rate or an override cell can be closed on one row and open on another. Editable cells the app filled (remaining qty, seller memory) are still stops (PRD FR-7.14).

## Moving and starting an edit

```ts
api.setFocusedCell(pos.rowIndex, pos.column);
// Only typing starts an edit. Enter and F2 never do.
// When a printable key arrives on a focused cell that is not editing:
api.startEditingCell({ rowIndex: pos.rowIndex, colKey: pos.column, key: event.key });
```

## Arrow keys while editing

While a cell is editing, AG Grid does not navigate on arrow keys, so the shared cell editors handle them:

- `↑` or `↓`: finish the edit (`api.stopEditing()`), then move up or down with the same navigation as when not editing.
- `←` or `→`: move the caret. When `caretAtVisualEdge` is true, finish the edit and move to the neighbouring cell.

```ts
function caretAtVisualEdge(p: SuppressKeyboardEventParams): boolean {
  const input = p.event.target as HTMLInputElement;
  const rtl = getComputedStyle(input).direction === 'rtl';
  const atStart = input.selectionStart === 0 && input.selectionEnd === 0;
  const atEnd = input.selectionStart === input.value.length;
  // The visual left edge is the end of the text in RTL and the start in LTR.
  const wantsLeft = p.event.key === 'ArrowLeft';
  return wantsLeft ? (rtl ? atEnd : atStart) : (rtl ? atStart : atEnd);
}
```

Number inputs are always `direction: ltr`, even in the Urdu layout, so their caret logic is the LTR case.

## Account cell editor

- A popup editor (`cellEditorPopup: true`) that wraps the shared `AccountPicker`, so every grid gets search, short codes, recent-first and the party card.
- Give any popup element rendered outside the editor the class `ag-custom-component-popup`, so the grid does not stop editing when it gets focus.
- While its list is open, the picker owns `↑` `↓` `Enter` and `Esc`.

## Column type map (shape, in `shared/grid/column_types.ts`)

```ts
export const entryColumnTypes: Record<EntryColumnType, ColDef> = {
  money:   { editable: true,  cellClass: 'cell-num', cellEditor: TextCellEditor },
  qty:     { editable: true,  cellClass: 'cell-num', cellEditor: TextCellEditor },
  rate:    { editable: true,  cellClass: 'cell-num', cellEditor: RateCellEditor },     // placeholder hint
  account: { editable: true,  cellEditor: AccountCellEditor, cellEditorPopup: true },
  date:    { editable: true,  cellClass: 'cell-num', cellEditor: DateCellEditor },
  text:    { editable: true,  cellEditor: TextCellEditor },
  calc:    { editable: false, cellClass: 'cell-num cell-calc', cellRenderer: CalcCell }, // shows CalcInfo
};
```

```css
/* tokens.css / grid.css: numbers on the physical right edge in both directions */
.cell-num { direction: ltr; text-align: end; font-variant-numeric: tabular-nums; }
```
