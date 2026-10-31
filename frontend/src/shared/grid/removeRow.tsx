// A visible button to remove a line from any entry grid (plan 43, PRD FR-7.3, FR-7.4, NFR-USE-7,
// NFR-USE-8). Ctrl+Delete was the only way, and a key is a shortcut, never the only way. Every grid that
// is made with `EntryGrid` gets this last column with no code of its own: a small trash button at the
// end edge of each row (the left in Urdu, the right in English), with the words "Remove this line" and
// the key Ctrl+Delete as its tooltip and accessible name.
//
// - The button and the key follow ONE rule and take ONE path (`useRemoveRow`): a grid whose rows are
//   fixed has no button, a row that `canRemoveRow` refuses (a saved lot of an edit) has none, and a row
//   that has more under it than itself (a lot with sale lines) first asks what will go (plan 43 R3).
// - After the removal, focus goes to the same column of the row that took its place, or of the one above
//   when it was the last (R4). The store is the one source of truth, so the live summary refreshes and
//   a language switch keeps the rest (Code Standards F39).
// - The column is not a stop of Enter: `EntryGrid` never gives it to the Enter order (FR-7.14). The
//   button is reached with Tab, and works with Enter and Space like any button (R5).

import { Localized, useLocalization } from '@fluent/react';
import type { ColDef, GridApi } from 'ag-grid-community';
import type { CustomCellRendererProps } from 'ag-grid-react';
import { Trash2 } from 'lucide-react';
import { useCallback, useRef, useState, type ReactNode, type RefObject } from 'react';
import { SHORTCUTS } from '../keyboard';
import { ConfirmDialog } from '../ui/ConfirmDialog';
import { isEditable } from './enterOrder';
import type { EntryStore } from './entryStore';
import type { EntryColumn, EntryRow } from './types';

/** The id of the remove column. It is not one of the grid's own columns, so no key and no Enter finds it. */
export const REMOVE_COLUMN_ID = '__remove';

/** The width of the remove column: a click target of 32 px and its room (NFR-USE-23). */
const REMOVE_COLUMN_WIDTH = 44;

/** What the screen says is under a row that is about to go: one Fluent sentence with its numbers. */
export interface RemoveQuestion {
  /** The Fluent message of the one sentence that says what will go. */
  messageId: string;
  /** The Fluent message of the title of the question. Default: "Remove the line?". */
  titleId?: string;
  vars?: Record<string, string | number>;
}

interface RemoveButtonProps {
  rowId: string;
  onRemove: (rowId: string) => void;
}

/** The trash button of one row. Its key is in its tooltip (FR-7.4), after its name. */
function RemoveRowButton({ rowId, onRemove }: RemoveButtonProps) {
  const { l10n } = useLocalization();
  const label = l10n.getString('common-remove-line');
  return (
    <button
      type="button"
      className="row-remove-button"
      aria-label={label}
      title={`${label} (${SHORTCUTS.removeRow.label})`}
      onClick={() => {
        onRemove(rowId);
      }}
    >
      <Trash2 aria-hidden="true" className="size-icon-sm" />
    </button>
  );
}

/** The heading of the remove column: no visible word, so the column stays narrow; a screen reader reads it. */
function RemoveHeader() {
  return (
    <span className="sr-only">
      <Localized id="common-remove-line" />
    </span>
  );
}

interface RemoveColumnEnv {
  /** Is the row one this grid lets the user remove? A row that is not has no button. */
  canRemove: (row: EntryRow) => boolean;
  onRemove: (rowId: string) => void;
}

/** The last column of an entry grid: one trash button for each row that can be removed. */
export function removeColumnDef(env: RemoveColumnEnv): ColDef<EntryRow> {
  return {
    colId: REMOVE_COLUMN_ID,
    headerName: '',
    headerComponent: RemoveHeader,
    width: REMOVE_COLUMN_WIDTH,
    minWidth: REMOVE_COLUMN_WIDTH,
    maxWidth: REMOVE_COLUMN_WIDTH,
    resizable: false,
    sortable: false,
    suppressMovable: true,
    editable: false,
    // A small column: its button and its heading are centred (plan 39 R3).
    cellClass: ['cell-remove', 'cell-center'],
    headerClass: ['header-center'],
    cellRenderer: (params: CustomCellRendererProps<EntryRow>) =>
      params.data === undefined || !env.canRemove(params.data) ? null : (
        <RemoveRowButton rowId={params.data.rowId} onRemove={env.onRemove} />
      ),
  };
}

interface UseRemoveRowOptions {
  store: EntryStore;
  columns: readonly EntryColumn[];
  apiRef: RefObject<GridApi<EntryRow> | null>;
  /** The rows are fixed (one for each account): nothing is removed. */
  fixedRows: boolean;
  canRemoveRow: ((row: EntryRow) => boolean) | undefined;
  removeQuestion: ((row: EntryRow) => RemoveQuestion | null) | undefined;
}

interface Asking {
  rowId: string;
  field: string;
  question: RemoveQuestion;
}

/**
 * The one way a row is removed, by the key or by the button. It returns `request(rowId, field?)`, which
 * removes the row now or, when the screen says something is under it, asks first, and the dialog to draw
 * once. `field` is the column that has focus now (the key knows it); the button has none, so the column
 * that had focus is used, or else the first column the row lets the user type in.
 */
export function useRemoveRow({
  store,
  columns,
  apiRef,
  fixedRows,
  canRemoveRow,
  removeQuestion,
}: UseRemoveRowOptions): {
  canRemove: (row: EntryRow) => boolean;
  request: (rowId: string, field?: string) => void;
  dialog: ReactNode;
} {
  const [asking, setAsking] = useState<Asking | null>(null);
  // True when the dialog closes because the row went: focus is then the grid's, not the dialog's.
  const removed = useRef(false);

  const canRemove = useCallback(
    (row: EntryRow) => !fixedRows && (canRemoveRow === undefined || canRemoveRow(row)),
    [fixedRows, canRemoveRow],
  );

  const remove = useCallback(
    (rowId: string, field: string) => {
      const { rows } = store.getState();
      const index = rows.findIndex((row) => row.rowId === rowId);
      if (index < 0) return;
      apiRef.current?.stopEditing(true);
      const remaining = rows.length - 1;
      store.getState().removeRow(rowId);
      // The row that took its place, or the one above when it was the last (plan 43 R4).
      store
        .getState()
        .setPendingFocus({ rowIndex: Math.min(index, Math.max(remaining - 1, 0)), field });
    },
    [store, apiRef],
  );

  const request = useCallback(
    (rowId: string, field?: string) => {
      const row = store.getState().rows.find((candidate) => candidate.rowId === rowId);
      if (row === undefined || !canRemove(row)) return;
      // The column to focus afterwards: the one the user was on, when he can type in it, else the first
      // cell of the row he can type in.
      const focused = store.getState().focus?.field;
      const target =
        field ??
        columns.find((column) => column.field === focused && isEditable(column, row))?.field ??
        columns.find((column) => isEditable(column, row))?.field ??
        columns[0]?.field ??
        '';
      const question = removeQuestion?.(row) ?? null;
      if (question === null) {
        remove(rowId, target);
        return;
      }
      removed.current = false;
      setAsking({ rowId, field: target, question });
    },
    [store, columns, canRemove, removeQuestion, remove],
  );

  const dialog =
    asking === null ? null : (
      <ConfirmDialog
        open
        onOpenChange={(open) => {
          if (!open) setAsking(null);
        }}
        title={<Localized id={asking.question.titleId ?? 'common-remove-line-title'} />}
        description={<Localized id={asking.question.messageId} vars={asking.question.vars ?? {}} />}
        confirmLabel={<Localized id="common-remove-line-confirm" />}
        tone="danger"
        onConfirm={() => {
          removed.current = true;
          remove(asking.rowId, asking.field);
        }}
        // After the row went, the grid takes the focus (its pending focus); on "back" it goes back.
        onCloseFocus={(event) => {
          if (removed.current) event.preventDefault();
        }}
      />
    );

  return { canRemove, request, dialog };
}
