// The editor of a `picker` cell (entry-grid skill, "Account cell editor"): the shared picker
// inside the cell, so every grid gets the same search, short codes, recent-first list and F4 as
// the pickers of the forms (PRD FR-7.10, FR-7.38 to FR-7.42, Code Standards F40).
//
// - It opens as a popup under the cell as soon as the cell gets the focus, by a click or by a key
//   (EntryGrid, PRD FR-7.16, plan 46). A letter, digit or code typed on the cell starts it with
//   that key already typed (PRD FR-7.14); Alt+↓ opens the full list again (FR-7.16).
// - What is chosen goes straight to the row store (never only inside the grid, F39). The editor
//   then closes the cell without a second save (`stopEditing(true)`, which cancels the grid's own
//   value step).
// - Enter in the open list picks and goes on to the next Enter stop, the same step as Enter on a
//   cell (FR-7.14). Enter with text typed and the list closed (Esc) is the picker's too: it picks
//   the first row of the answer to that text, and the cell goes on after it (plan 61). While the list is open ↑ ↓ choose in it, ← → and Tab leave the cell, and
//   Ctrl shortcuts (copy from above, remove the row) reach the grid. With the list closed, Enter,
//   the arrows and the other grid keys are answered by the grid's own key handler, so the keys are
//   the same in every cell (F27).
// - The party card is not drawn here: the screen draws it beside the grid, from `onCardAccount`.

import type { SuppressKeyboardEventParams } from 'ag-grid-community';
import type { CustomCellEditorProps } from 'ag-grid-react';
import { useEffect, useRef, useState, type KeyboardEvent } from 'react';
import { pickedOf, type CardChukta, type PickedSource } from '../accounts';
import { AccountPicker } from '../ui/AccountPicker';
import { Picker } from '../ui/Picker';
import type { EntryStore } from './entryStore';
import { leftTextKey } from './leftText';
import type { EntryRow, PickerSpec } from './types';

type KeyParams = SuppressKeyboardEventParams<EntryRow>;

/** What the grid hands to the editor of a picker column. */
export interface PickerEditorParams {
  spec: PickerSpec;
  store: EntryStore;
  field: string;
  /** The grid's key handler. Asked when a key arrives, because the grid may have a new one. */
  getKeyHandler: () => (params: KeyParams, fromEditor?: boolean) => boolean;
  /**
   * The account the screen's party card would show now (null for none), and for a چکتا buyer who
   * he is: his names and phone (PRD FR-13.6).
   */
  onCardAccount?: ((accountId: number | null, chukta?: CardChukta | null) => void) | undefined;
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
  // The text in the field as the last key left it, until a record is picked or the edit is cancelled.
  // When the cell is left with text in it and nothing picked, it is kept on the row (leftText.ts).
  const typedNow = useRef('');
  const value = store((state) => state.rows.find((row) => row.rowId === rowId)?.picked?.[field]);
  // A key that opened the cell is the first letter; Alt+↓ opened it for the full list.
  const [start] = useState(() => ({
    text: eventKey !== null && eventKey.length === 1 ? eventKey : undefined,
    open: store.getState().editHint === 'open-list',
  }));

  useEffect(() => {
    // A key that opened the cell is the first letter of the text.
    typedNow.current = start.text ?? '';
    store.getState().setEditHint(null);
    const input = wrapper.current?.querySelector('input');
    if (input === null || input === undefined) return;
    input.focus();
    // Opened by a key, the key is the first letter and the cursor goes after it. Opened by a click
    // or by Alt+↓, a name already in the cell is selected, so typing replaces it (PRD FR-7.15).
    if (start.text === undefined) {
      input.select();
    } else {
      const end = input.value.length;
      input.setSelectionRange(end, end);
    }
  }, [store, start.text]);

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

  const commit = (hit: PickedSource | null) => {
    typedNow.current = '';
    const state = store.getState();
    state.updatePicked(rowId, field, hit === null ? null : pickedOf(hit));
    // A record is chosen, so the text that was left before is no longer waiting for one.
    const key = leftTextKey(field);
    if ((state.rows.find((row) => row.rowId === rowId)?.typed[key] ?? '') !== '') {
      state.updateTyped(rowId, key, '');
    }
  };

  // The cell closes with text typed and nothing picked: keep the text, so the line can say so.
  useEffect(
    () => () => {
      const text = typedNow.current.trim();
      if (text === '') return;
      const state = store.getState();
      if (state.rows.find((row) => row.rowId === rowId)?.picked?.[field] !== undefined) return;
      state.updateTyped(rowId, leftTextKey(field), text);
    },
    [store, rowId, field],
  );

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
    // The list is open whenever the cell is, so ← → and Tab still leave the cell and the Ctrl
    // shortcuts of the grid still work (plan 46).
    const listOpen =
      target instanceof HTMLElement && target.getAttribute('aria-expanded') === 'true';
    if (
      listOpen &&
      !event.ctrlKey &&
      event.key !== 'ArrowLeft' &&
      event.key !== 'ArrowRight' &&
      event.key !== 'Tab'
    ) {
      return;
    }
    if (event.key === 'Escape') {
      // Esc cancels the typing in the cell: nothing is kept (PRD FR-7.18).
      typedNow.current = '';
      return;
    }
    if (isTypingKey(event)) return;
    // An Enter the picker has taken is not the grid's: it is resolving the text in the field (the
    // list was closed, or is an older answer), and the cell goes on only after it has picked
    // (plan 61, PRD FR-7.14). Without this the cell would end with the text and nothing picked.
    if (event.key === 'Enter' && event.defaultPrevented) return;
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
          withChukta={spec.withChukta === true}
          value={value ?? null}
          showCard={false}
          initialText={start.text}
          startOpen={start.open}
          onChange={commit}
          onEnterPicked={goOn}
          onCardAccount={onCardAccount}
          onTyped={(text) => {
            typedNow.current = text;
          }}
        />
      ) : (
        <Picker
          scope={spec.scope}
          value={value ?? null}
          initialText={start.text}
          startOpen={start.open}
          onChange={commit}
          onEnterPicked={goOn}
          onTyped={(text) => {
            typedNow.current = text;
          }}
        />
      )}
    </div>
  );
}
