// The on-screen key map of the phonetic Urdu keyboard (PRD FR-7.21): every key and the Urdu
// letter it types. A click on a key also types the letter, so the map works with the mouse alone
// (NFR-USE-8). It opens with one click on the field's key-map button.

import { Localized, useLocalization } from '@fluent/react';
import { X } from 'lucide-react';
import { KEY_MAP } from './phonetic_keyboard';

interface KeyMapProps {
  onPick: (letter: string) => void;
  onClose: () => void;
}

export function KeyMap({ onPick, onClose }: KeyMapProps) {
  const { l10n } = useLocalization();
  return (
    <div
      role="group"
      aria-label={l10n.getString('translit-keymap-title')}
      className="absolute inset-x-0 top-full z-20 mt-1 flex flex-col gap-3 rounded-2 border border-border bg-surface p-3 shadow-overlay"
    >
      <div className="flex items-center justify-between gap-3">
        <Localized id="translit-keymap-title">
          <p className="type-label" />
        </Localized>
        <button
          type="button"
          tabIndex={-1}
          aria-label={l10n.getString('translit-keymap-close')}
          onMouseDown={(event) => {
            event.preventDefault();
          }}
          onClick={onClose}
          className="flex size-control-compact items-center justify-center rounded-2 text-ink-muted"
        >
          <X aria-hidden="true" className="size-icon-md" />
        </button>
      </div>
      <ul className="flex flex-wrap gap-2" dir="ltr">
        {KEY_MAP.map(({ key, letter }) => (
          <li key={key}>
            <button
              type="button"
              tabIndex={-1}
              // A click types the letter without taking focus from the field.
              onMouseDown={(event) => {
                event.preventDefault();
              }}
              onClick={() => {
                onPick(letter);
              }}
              className="flex min-h-control-compact min-w-control-compact items-center justify-center gap-2 rounded-1 border border-border px-2 py-1"
            >
              <kbd lang="en" className="type-number">
                {key}
              </kbd>
              <span lang="ur" dir="rtl" className="type-field">
                {letter}
              </span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
