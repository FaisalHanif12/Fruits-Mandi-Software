// A text field that suggests values used before, such as a truck number (PRD FR-7.49). Unlike a
// picker it takes text that is not in the list: a new truck number is a normal thing to type. So
// typing never picks anything by itself, Enter takes what is typed unless ↑ ↓ moved onto a
// suggestion, and leaving the field keeps the text.
//
// - The suggestions come from the screen (the backend answers while the user types). The field
//   shows them under itself, newest first, each with a short note, such as its last day.
// - ↑ ↓ move through the suggestions while the list is open, Enter takes the highlighted one and
//   moves on, Esc closes the list (PRD FR-7.16, FR-7.18). With the list closed, the arrows move
//   between fields like in any other field.
// - `onCommit` is called with the text when the user is done with it: a suggestion was taken, Enter
//   was pressed, or the field was left. The screen uses it to fill what it remembers (FR-7.49).
// - Codes are Latin and written left to right in both languages (F25).

import { useCombobox } from 'downshift';
import { useRef, useState, type KeyboardEvent } from 'react';
import { focusNextField, keepKeyForApp } from '../keyboard';
import { Code } from './Code';
import { controlClasses } from './control-classes';
import type { FieldControlProps } from './Field';

export interface Suggestion {
  /** The text that goes into the field. */
  value: string;
  /** A short note beside it, for example the date of its last use. Already formatted. */
  note?: string | undefined;
}

interface SuggestInputProps extends FieldControlProps {
  value: string;
  onChange: (text: string) => void;
  suggestions: readonly Suggestion[];
  onCommit?: ((text: string) => void) | undefined;
  placeholder?: string | undefined;
}

/**
 * Remembers the text that was last handed to the screen, so a suggestion taken (which also moves
 * focus away) is not handed over a second time when the field is left.
 */
function useCommitOnce(onCommit: ((text: string) => void) | undefined) {
  const last = useRef<string | null>(null);
  const commit = (text: string) => {
    if (last.current === text) return;
    last.current = text;
    onCommit?.(text);
  };
  const forget = () => {
    last.current = null;
  };
  return { commit, forget };
}

export function SuggestInput({
  id,
  'aria-describedby': describedBy,
  invalid,
  readOnly,
  value,
  onChange,
  suggestions,
  onCommit,
  placeholder,
}: SuggestInputProps) {
  const [open, setOpen] = useState(false);
  const { commit, forget } = useCommitOnce(onCommit);

  const { getInputProps, getItemProps, getMenuProps, highlightedIndex } = useCombobox<Suggestion>({
    items: [...suggestions],
    ...(id === undefined ? {} : { inputId: id }),
    itemToString: (item) => item?.value ?? '',
    selectedItem: null,
    inputValue: value,
    isOpen: open && suggestions.length > 0,
    defaultHighlightedIndex: -1,
    scrollIntoView: () => undefined,
    stateReducer: (state, { type, changes }) => {
      // Leaving the field keeps the text; only a suggestion taken on purpose changes it.
      if (type === useCombobox.stateChangeTypes.InputBlur) {
        return { ...changes, selectedItem: state.selectedItem, inputValue: state.inputValue };
      }
      return changes;
    },
    onIsOpenChange: ({ isOpen }) => {
      setOpen(isOpen);
    },
    onSelectedItemChange: ({ selectedItem }) => {
      if (selectedItem === null) return;
      onChange(selectedItem.value);
      setOpen(false);
      commit(selectedItem.value);
    },
  });

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (readOnly === true || event.altKey || event.ctrlKey || event.metaKey) return;
    const listOpen = open && suggestions.length > 0;
    if (!listOpen) {
      // A closed list is a field like any other: the arrows go on to the form (FR-7.16).
      if (event.key === 'ArrowUp' || event.key === 'ArrowDown' || event.key === 'Escape') {
        keepKeyForApp(event);
      }
      return;
    }
    if (event.key === 'Escape') {
      // Esc closes the list first; the next Esc is the screen's (FR-7.18).
      keepKeyForApp(event);
      event.preventDefault();
      event.stopPropagation();
      setOpen(false);
      return;
    }
    if (event.key === 'Enter' && highlightedIndex < 0) {
      // Nothing was highlighted: Enter takes the text as typed and moves on.
      keepKeyForApp(event);
      event.preventDefault();
      setOpen(false);
      commit(value);
      focusNextField(event.currentTarget);
    }
  };

  const showList = open && suggestions.length > 0;
  return (
    <div className="relative">
      <input
        {...getInputProps({
          type: 'text',
          readOnly,
          placeholder,
          'aria-invalid': invalid,
          'aria-describedby': describedBy,
          // The field has the label of its Field (for/id). Downshift's own label does not exist.
          'aria-labelledby': undefined,
          autoComplete: 'off',
          spellCheck: false,
          dir: 'ltr',
          lang: 'en',
          onKeyDown,
          onChange: (event: { currentTarget: HTMLInputElement }) => {
            forget();
            onChange(event.currentTarget.value);
            setOpen(true);
          },
          onBlur: () => {
            setOpen(false);
            if (value.trim() !== '') commit(value);
          },
        })}
        className={`${controlClasses({ invalid, readOnly })} type-number-field`}
      />
      <div
        className={
          showList
            ? 'absolute start-0 top-full z-20 mt-1 w-max min-w-full rounded-2 border border-border bg-surface p-1 shadow-overlay'
            : 'hidden'
        }
      >
        <ul {...getMenuProps()} className="max-h-picker-list overflow-y-auto">
          {showList
            ? suggestions.map((suggestion, index) => (
                <li
                  key={suggestion.value}
                  {...getItemProps({ item: suggestion, index })}
                  className={`flex h-control cursor-pointer items-center justify-between gap-4 whitespace-nowrap rounded-1 px-3 ${index === highlightedIndex ? 'bg-primary-soft text-primary' : 'text-ink'}`}
                >
                  <Code>{suggestion.value}</Code>
                  {suggestion.note === undefined ? null : (
                    <span className="type-body text-ink-muted">
                      <Code>{suggestion.note}</Code>
                    </span>
                  )}
                </li>
              ))
            : null}
        </ul>
      </div>
    </div>
  );
}
