// Every keyboard shortcut of PRD 7.3, declared here, once (Code Standards F29). Buttons show
// `label` next to their name (PRD FR-7.4). A screen never writes its own key check (F27).
//
// Keys are matched by `event.code` (the physical key), not `event.key`, so Alt+L works the same
// with an Urdu keyboard layout and on a Mac, where Alt changes the typed character.

/** Where a key works. `anywhere` keys work in the whole window. */
export type ShortcutScope = 'anywhere' | 'entry' | 'grid' | 'field' | 'picker';

/** One physical key with its modifiers. Modifiers not listed must be up. */
export interface ShortcutKey {
  /** The physical key, for example 'KeyL' or 'F10'. */
  readonly code: string;
  readonly alt?: boolean;
  readonly ctrl?: boolean;
  /** `'any'`: Shift may be up or down (for example Ctrl and the plus key, which needs Shift). */
  readonly shift?: boolean | 'any';
}

export interface Shortcut extends ShortcutKey {
  /** How the key is written on buttons and hints. The same in Urdu and English. */
  readonly label: string;
  readonly scope: ShortcutScope;
  /** Other keys that do the same, for example Ctrl+S for save and the number pad plus. */
  readonly also?: readonly ShortcutKey[];
}

/** PRD 7.3, one entry per row of the table, plus the keys of FR-7.16 and FR-7.19. */
export const SHORTCUTS = {
  /** New entry on the current screen. Inside a grid, F2 does not start cell editing. */
  newEntry: { code: 'F2', label: 'F2', scope: 'entry' },
  /** Find. */
  find: { code: 'F3', label: 'F3', scope: 'entry' },
  /** In a picker: add a new account without leaving the screen (FR-7.42). */
  addAccount: { code: 'F4', label: 'F4', scope: 'picker' },
  /** Print. */
  print: { code: 'F9', label: 'F9', scope: 'anywhere' },
  /** Save on entry screens. Ctrl+S does the same. */
  save: { code: 'F10', label: 'F10', scope: 'entry', also: [{ code: 'KeyS', ctrl: true }] },
  /** Save, print the ticked documents and start a new entry (FR-7.57). */
  saveAndNext: { code: 'Enter', ctrl: true, label: 'Ctrl+Enter', scope: 'entry' },
  /** In a grid, before save: remove the current row. */
  removeRow: { code: 'Delete', ctrl: true, label: 'Ctrl+Delete', scope: 'grid' },
  /** In a text field: typing mode (FR-7.21). */
  typingMode: { code: 'KeyU', alt: true, label: 'Alt+U', scope: 'field' },
  /** Switch between اردو and English, anywhere (PRD FR-7.30). */
  languageToggle: { code: 'KeyL', alt: true, label: 'Alt+L', scope: 'anywhere' },
  /** Global search. */
  globalSearch: { code: 'KeyK', ctrl: true, label: 'Ctrl+K', scope: 'anywhere' },
  /** On a calculated amount: how was this worked out? (FR-7.43). */
  explain: { code: 'KeyI', ctrl: true, label: 'Ctrl+I', scope: 'field' },
  /** In a sale-line cell: copy the value from the line above (FR-7.54). */
  copyAbove: { code: 'KeyD', ctrl: true, label: 'Ctrl+D', scope: 'grid' },
  /** While the saved message shows: undo the save (FR-7.47). */
  undoSave: { code: 'KeyZ', alt: true, label: 'Alt+Z', scope: 'anywhere' },
  /** Text size bigger. The plus key needs Shift on most keyboards, so Shift may be down. */
  textBigger: {
    code: 'Equal',
    ctrl: true,
    shift: 'any',
    label: 'Ctrl +',
    scope: 'anywhere',
    also: [{ code: 'NumpadAdd', ctrl: true }],
  },
  textSmaller: {
    code: 'Minus',
    ctrl: true,
    label: 'Ctrl −',
    scope: 'anywhere',
    also: [{ code: 'NumpadSubtract', ctrl: true }],
  },
  textNormal: {
    code: 'Digit0',
    ctrl: true,
    label: 'Ctrl 0',
    scope: 'anywhere',
    also: [{ code: 'Numpad0', ctrl: true }],
  },
  /** New بکری. */
  newBikri: { code: 'KeyB', alt: true, label: 'Alt+B', scope: 'anywhere' },
  /** New وصولی. */
  newReceipt: { code: 'KeyV', alt: true, label: 'Alt+V', scope: 'anywhere' },
  /** New ادائیگی. */
  newPayment: { code: 'KeyA', alt: true, label: 'Alt+A', scope: 'anywhere' },
  /** New پیشگی. */
  newAdvance: { code: 'KeyP', alt: true, label: 'Alt+P', scope: 'anywhere' },
  /** New دکان خرچہ. */
  newExpense: { code: 'KeyK', alt: true, label: 'Alt+K', scope: 'anywhere' },
  /** Enter finishes a small form: sign in, next step (PRD Section 8.1, FR-7.22). */
  confirm: { code: 'Enter', label: 'Enter', scope: 'field' },
  /** Close a list or dialog, else cancel the typing, else go back (FR-7.18). */
  back: { code: 'Escape', label: 'Esc', scope: 'anywhere' },
  /** In a picker: open the full list without typing (FR-7.16). */
  openList: { code: 'ArrowDown', alt: true, label: 'Alt+↓', scope: 'picker' },
  /** In a date field: next day (FR-7.19). */
  dateNextDay: {
    code: 'Equal',
    shift: 'any',
    label: '+',
    scope: 'field',
    also: [{ code: 'NumpadAdd' }],
  },
  /** In a date field: previous day (FR-7.19). */
  datePreviousDay: {
    code: 'Minus',
    label: '−',
    scope: 'field',
    also: [{ code: 'NumpadSubtract' }],
  },
} as const satisfies Record<string, Shortcut>;

/** Every shortcut, for tests and for a future help screen. */
export const ALL_SHORTCUTS: readonly Shortcut[] = Object.values(SHORTCUTS);

function keyMatches(event: KeyboardEvent, key: ShortcutKey): boolean {
  const shift = key.shift ?? false;
  return (
    event.code === key.code &&
    event.altKey === (key.alt ?? false) &&
    event.ctrlKey === (key.ctrl ?? false) &&
    (shift === 'any' || event.shiftKey === shift) &&
    !event.metaKey
  );
}

/** True when the key event is exactly this shortcut (no extra modifier keys). */
export function matchesShortcut(event: KeyboardEvent, shortcut: Shortcut): boolean {
  return keyMatches(event, shortcut) || (shortcut.also ?? []).some((key) => keyMatches(event, key));
}
