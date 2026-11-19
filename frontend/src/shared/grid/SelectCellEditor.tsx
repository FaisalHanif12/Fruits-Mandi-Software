// The editor of a `select` cell (entry-grid skill): a small fixed list of words in a popup under
// the cell, for a value that is optional and has only a few names, such as the quality of a sale
// line (اول, دوم, سوم, or nothing). No typing and no search: the words are fixed (PRD FR-12.42).
//
// - A click on the cell, Space or Alt+↓ opens it (gridKeys.ts); the list opens with the value of
//   the cell chosen.
// - ↑ ↓ move in the list, Home and End go to its ends, Enter or Space choose the word and close the
//   list, and the focus stays on the cell. Enter does not go on to another cell, because a select
//   cell is not an Enter stop; Enter on the cell with the list closed moves on like on any cell.
// - ← → and Tab leave the cell like in a picker cell: Tab keeps the word that was chosen and takes
//   the step Enter takes. Esc closes the list and keeps the old value (the Esc order, FR-7.18).
// - What is chosen goes straight to the row store (F39), never only inside the grid.

import type { SuppressKeyboardEventParams } from 'ag-grid-community';
import type { CustomCellEditorProps } from 'ag-grid-react';
import { Check } from 'lucide-react';
import { useEffect, useRef, useState, type KeyboardEvent } from 'react';
import type { EntryStore } from './entryStore';
import type { EntryRow } from './types';

type KeyParams = SuppressKeyboardEventParams<EntryRow>;

/** One word of the list: what is kept, and the words as the app language says them. */
export interface SelectOption {
  readonly value: string;
  readonly label: string;
}

/** What the grid hands to the editor of a select column. */
export interface SelectEditorParams {
  options: readonly SelectOption[];
  store: EntryStore;
  field: string;
  /** The grid's key handler. Asked when a key arrives, because the grid may have a new one. */
  getKeyHandler: () => (params: KeyParams, fromEditor?: boolean) => boolean;
}

type EditorProps = CustomCellEditorProps<EntryRow, unknown, unknown> & SelectEditorParams;

/** The id of one word of the list, so the list can say which word is highlighted. */
function optionId(rowId: string, field: string, index: number): string {
  return `select-${rowId}-${field}-${String(index)}`;
}

export function SelectCellEditor(props: EditorProps) {
  const { options, store, field, getKeyHandler, data, api, node, column, colDef, context } = props;
  const rowId = data.rowId;
  const list = useRef<HTMLUListElement>(null);
  // The word the cell holds now (it is marked in the list), read from the row store.
  const kept = store((state) => state.rows.find((row) => row.rowId === rowId)?.typed[field] ?? '');
  // The list opens with the word of the cell highlighted.
  const [active, setActive] = useState(() =>
    Math.max(
      options.findIndex((option) => option.value === kept),
      0,
    ),
  );

  // The list has the focus as soon as it opens, so its keys are answered here.
  useEffect(() => {
    list.current?.focus();
  }, []);

  const keyParams = (event: KeyboardEvent, editing: boolean): KeyParams => ({
    event: event.nativeEvent,
    editing,
    api,
    context,
    node,
    data,
    column,
    colDef,
  });

  const close = () => {
    api.stopEditing(true);
    // The cell keeps the focus: the user is still on it, with the new word in it.
    if (node.rowIndex !== null) api.setFocusedCell(node.rowIndex, field);
  };

  const choose = (index: number) => {
    const option = options[index];
    if (option === undefined) return;
    store.getState().updateTyped(rowId, field, option.value);
    close();
  };

  const onKeyDown = (event: KeyboardEvent<HTMLUListElement>) => {
    if (event.ctrlKey || event.metaKey) {
      // Ctrl shortcuts of the grid (copy from above, remove the row) still reach it.
      getKeyHandler()(keyParams(event, true), true);
      return;
    }
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        setActive((at) => Math.min(at + 1, options.length - 1));
        return;
      case 'ArrowUp':
        event.preventDefault();
        setActive((at) => Math.max(at - 1, 0));
        return;
      case 'Home':
        event.preventDefault();
        setActive(0);
        return;
      case 'End':
        event.preventDefault();
        setActive(options.length - 1);
        return;
      case 'Enter':
      case ' ':
        event.preventDefault();
        event.stopPropagation();
        choose(active);
        return;
      case 'Tab': {
        // The word under the highlight stays only if it was chosen; Tab leaves with what the cell has.
        event.preventDefault();
        api.stopEditing(true);
        getKeyHandler()(
          {
            ...keyParams(event, false),
            event: new KeyboardEvent('keydown', { key: 'Enter', cancelable: true }),
          },
          true,
        );
        return;
      }
      case 'ArrowLeft':
      case 'ArrowRight':
        // To the neighbouring cell, as in a picker cell.
        api.stopEditing(true);
        getKeyHandler()(keyParams(event, false), true);
        return;
      default:
        // Esc is answered by the Esc order; any other key does nothing in a fixed list.
        if (event.key !== 'Escape') event.preventDefault();
    }
  };

  const activeId = optionId(rowId, field, active);
  return (
    // The class keeps the grid from ending the edit when the focus is in this popup.
    <ul
      ref={list}
      role="listbox"
      tabIndex={-1}
      aria-label={colDef.headerName}
      {...{ 'aria-activedescendant': activeId }}
      onKeyDown={onKeyDown}
      className="ag-custom-component-popup min-w-picker-cell rounded-2 border border-border bg-surface p-1 shadow-overlay outline-none"
    >
      {options.map((option, index) => (
        <li
          key={option.value}
          id={optionId(rowId, field, index)}
          role="option"
          aria-selected={index === active}
          onMouseEnter={() => {
            setActive(index);
          }}
          onClick={() => {
            choose(index);
          }}
          className={`flex h-control cursor-default items-center justify-between gap-3 rounded-1 px-3 type-field outline-none ${index === active ? 'bg-primary-soft text-primary' : 'text-ink'}`}
        >
          {option.label}
          {option.value === kept ? (
            <Check aria-hidden="true" className="size-icon-sm text-primary" />
          ) : null}
        </li>
      ))}
    </ul>
  );
}
