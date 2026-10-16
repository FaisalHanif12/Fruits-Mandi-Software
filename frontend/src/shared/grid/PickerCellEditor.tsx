// The editor of a `picker` cell (entry-grid skill, "Account cell editor"): the shared picker
// inside the cell, so every grid gets the same search, short codes, recent-first list and F4 as
// the pickers of the forms (PRD FR-7.10, FR-7.38 to FR-7.42, Code Standards F40).
//
// - It opens as a popup under the cell. A letter, digit or code typed on the cell starts it with
//   that key already typed (PRD FR-7.14); Alt+↓ opens it with the full list (FR-7.16).
// - What is chosen goes straight to the row store (never only inside the grid, F39). The editor
//   then closes the cell without a second save (`stopEditing(true)`, which cancels the grid's own
//   value step).
// - Enter in the open list picks and goes on to the next Enter stop, the same step as Enter on a
//   cell (FR-7.14). With the list closed, Enter, the arrows and the other grid keys are answered
//   by the grid's own key handler, so the keys are the same in every cell (F27).
// - The party card is not drawn here: the screen draws it beside the grid, from `onCardAccount`.

import type { SuppressKeyboardEventParams } from 'ag-grid-community';
import type { CustomCellEditorProps } from 'ag-grid-react';
import { useEffect, useRef, useState, type KeyboardEvent } from 'react';
import { AccountPicker } from '../ui/AccountPicker';
import { Picker } from '../ui/Picker';
import type { EntryStore } from './entryStore';
import type { EntryRow, PickedRecord, PickerSpec } from './types';

type KeyParams = SuppressKeyboardEventParams<EntryRow>;

/** What the grid hands to the editor of a picker column. */
export interface PickerEditorParams {
  spec: PickerSpec;
  store: EntryStore;
  field: string;
  /** The grid's key handler. Asked when a key arrives, because the grid may have a new one. */
  getKeyHandler: () => (params: KeyParams, fromEditor?: boolean) => boolean;
  /** The account the screen's party card would show now (null for none). */
  onCardAccount?: ((accountId: number | null) => void) | undefined;
}

type EditorProps = CustomCellEditorProps<EntryRow, unknown, unknown> & PickerEditorParams;

/** Keys the editor leaves to the picker or to the browser. */
function isTypingKey(event: KeyboardEvent): boolean {
  return event.key.length === 1 && !event.ctrlKey && !event.altKey && !event.metaKey;
}

export function PickerCellEditor(props: EditorProps) {
  const { spec, store, field, getKeyHandler, onCardAccount, data, api, node, column, colDef } =
    props;
  const { context, eventKey } = props;
  const rowId = data.rowId;
  const wrapper = useRef<HTMLDivElement>(null);
  const value = store((state) => state.rows.find((row) => row.rowId === rowId)?.picked?.[field]);
  // A key that opened the cell is the first letter; Alt+↓ opened it for the full list.
  const [start] = useState(() => ({
    text: eventKey !== null && eventKey.length === 1 ? eventKey : undefined,
    open: store.getState().editHint === 'open-list',
  }));

  useEffect(() => {
    store.getState().setEditHint(null);
    const input = wrapper.current?.querySelector('input');
    if (input === null || input === undefined) return;
    input.focus();
    const end = input.value.length;
    input.setSelectionRange(end, end);
  }, [store]);

  // The card follows the cell while it is open; when it closes, the screen goes back to the row.
  useEffect(
    () => () => {
      onCardAccount?.(null);
    },
    [onCardAccount],
  );

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

  const commit = (hit: PickedRecord | null) => {
    store
      .getState()
      .updatePicked(
        rowId,
        field,
        hit === null
          ? null
          : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
      );
  };

  // Enter has picked: close the cell, then take the step Enter takes on any cell.
  const goOn = () => {
    api.stopEditing(true);
    getKeyHandler()({
      event: new KeyboardEvent('keydown', { key: 'Enter', cancelable: true }),
      editing: false,
      api,
      context,
      node,
      data,
      column,
      colDef,
    });
  };

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const target = event.target;
    // An open list owns ↑ ↓ and Enter (PRD FR-7.16); Esc is answered by the Esc order (FR-7.18).
    if (target instanceof HTMLElement && target.getAttribute('aria-expanded') === 'true') return;
    if (event.key === 'Escape' || isTypingKey(event)) return;
    if (event.key === 'Tab') {
      event.preventDefault();
      goOn();
      return;
    }
    // Alt+↓ and F4 belong to the picker itself.
    if (event.altKey || event.key === 'F4') return;
    getKeyHandler()(keyParams(event, true), true);
  };

  return (
    // The class keeps the grid from ending the edit when focus moves into a part of the picker
    // that is drawn outside this element, such as the dialog of F4 (entry-grid skill).
    <div
      ref={wrapper}
      onKeyDown={onKeyDown}
      className="ag-custom-component-popup min-w-picker-cell rounded-2 border border-border bg-surface p-1 shadow-overlay"
    >
      {spec.kind === 'account' ? (
        <AccountPicker
          scope={spec.scope}
          value={value ?? null}
          showCard={false}
          initialText={start.text}
          startOpen={start.open}
          onChange={commit}
          onEnterPicked={goOn}
          onCardAccount={onCardAccount}
        />
      ) : (
        <Picker
          scope={spec.scope}
          value={value ?? null}
          initialText={start.text}
          startOpen={start.open}
          onChange={commit}
          onEnterPicked={goOn}
        />
      )}
    </div>
  );
}
