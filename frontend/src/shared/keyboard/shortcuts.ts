// Every keyboard shortcut is declared here, once (Code Standards F29, PRD 7.3).
// Buttons show `label` next to their name (PRD FR-7.4).
//
// Keys are matched by `event.code` (the physical key), not `event.key`, so Alt+L works the same
// with an Urdu keyboard layout and on a Mac, where Alt changes the typed character.

export interface Shortcut {
  /** The physical key, for example 'KeyL' or 'F10'. */
  readonly code: string;
  readonly alt?: boolean;
  readonly ctrl?: boolean;
  readonly shift?: boolean;
  /** How the key is written on buttons and hints. The same in Urdu and English. */
  readonly label: string;
}

export const SHORTCUTS = {
  /** Switch between اردو and English, anywhere (PRD FR-7.30). */
  languageToggle: { code: 'KeyL', alt: true, label: 'Alt+L' },
} as const satisfies Record<string, Shortcut>;

/** True when the key event is exactly this shortcut (no extra modifier keys). */
export function matchesShortcut(event: KeyboardEvent, shortcut: Shortcut): boolean {
  return (
    event.code === shortcut.code &&
    event.altKey === (shortcut.alt ?? false) &&
    event.ctrlKey === (shortcut.ctrl ?? false) &&
    event.shiftKey === (shortcut.shift ?? false) &&
    !event.metaKey
  );
}
