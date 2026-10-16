// The field for Urdu text: names, notes, narrations, reasons (PRD FR-7.21, FR-7.22, Code
// Standards F41). It types Urdu three ways, switched with Alt+U and shown as a badge in the field:
//
// - رومن (default): the user types a word in English letters, up to three Urdu spellings show
//   under the field and the first is highlighted. Space, Enter or leaving the field takes the
//   highlighted one. Space stays in the field; Enter then moves on. ↑ ↓ move the highlight. Esc
//   closes the list and keeps the English letters. Backspace straight after a conversion brings
//   back the English letters of that word. Picking another spelling than the first is learned.
// - اردو: the phonetic keyboard (shared/urdu/phonetic_keyboard.ts) types Urdu letters, with a key
//   map that opens with one click.
// - EN: no conversion.
//
// Letters from a Windows Urdu keyboard pass through unchanged in every mode. Pickers, search boxes
// and code, number and phone fields never use this component (F41): they search raw text.
// The field is always written right to left, whatever the app language.

import { useLocalization } from '@fluent/react';
import { Keyboard } from 'lucide-react';
import {
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type ChangeEvent,
  type KeyboardEvent,
} from 'react';
import type { Kind } from '../../bindings/Kind';
import { SHORTCUTS, focusNextField, matchesShortcut, useEscape } from '../keyboard';
import { translitLearn } from '../urdu/api';
import { KeyMap } from '../urdu/KeyMap';
import { mapPhoneticKey } from '../urdu/phonetic_keyboard';
import { trailingLatinRun, wordsOf } from '../urdu/text';
import { wordsStillIn } from '../urdu/spelling';
import type { Source } from '../../bindings/Source';
import { MODE_BADGE_ID, MODE_FULL_ID, nextTypingMode, type TypingMode } from '../urdu/typing-mode';
import { useSuggestions } from '../urdu/useSuggestions';
import { controlClasses } from './control-classes';
import type { FieldControlProps } from './Field';
import { SpellingMark } from './SpellingMark';

/** A word the user turned into Urdu: the English letters he typed and the spelling he took. */
export interface ConvertedWord {
  roman: string;
  urdu: string;
  /** Where the spelling came from. `rules` is a spelling to check (PRD FR-7.66). */
  source: Source;
}

interface RomanUrduInputProps extends FieldControlProps {
  value: string;
  onChange: (text: string) => void;
  /** The typing mode. Leave out for a field that keeps its own (starts in رومن). */
  mode?: TypingMode | undefined;
  onModeChange?: ((mode: TypingMode) => void) | undefined;
  placeholder?: string | undefined;
  /** What a learned pair is about (name, place, item or word). Default: name. */
  kind?: Kind | undefined;
  /** Called when a word has been turned into Urdu, so a name pair can fill its English name. */
  onWordConverted?: ((word: ConvertedWord) => void) | undefined;
  /** The English word being typed right now, or '' (it is not Urdu yet, so it is not an error). */
  onPendingChange?: ((word: string) => void) | undefined;
  /**
   * Words of the text that came from the spelling rules alone and are not confirmed. They show in
   * amber under the field with "check the spelling" (PRD FR-7.66). The screen keeps the list in
   * its own state, so a language switch loses nothing. Leave out for free text, which is not
   * checked.
   */
  marked?: readonly string[] | undefined;
  /**
   * The direction the field is written in. Default `rtl`: Urdu is always written right to left,
   * whatever the app language. Free text typed in English in the English app is `ltr`
   * (FreeTextInput).
   */
  direction?: 'rtl' | 'ltr' | undefined;
}

interface Conversion {
  roman: string;
  urdu: string;
  /** What followed the word: a space when Space took it, else nothing. */
  trailing: string;
}

export function RomanUrduInput({
  id,
  'aria-describedby': describedBy,
  invalid,
  readOnly,
  value,
  onChange,
  mode: modeProp,
  onModeChange,
  placeholder,
  kind = 'name',
  onWordConverted,
  onPendingChange,
  marked,
  direction = 'rtl',
}: RomanUrduInputProps) {
  const { l10n } = useLocalization();
  const inputRef = useRef<HTMLInputElement>(null);
  const listId = useId();
  const [ownMode, setOwnMode] = useState<TypingMode>('roman');
  const mode = modeProp ?? ownMode;
  const [focused, setFocused] = useState(false);
  const [dismissedFor, setDismissedFor] = useState('');
  const [highlight, setHighlight] = useState(0);
  const [keyMapOpen, setKeyMapOpen] = useState(false);
  const [conversion, setConversion] = useState<Conversion | null>(null);
  const caretAfterInsert = useRef<number | null>(null);
  // The words turned into Urdu in this field, in order, so a later correction can be learned.
  const converted = useRef<{ roman: string; urdu: string }[]>([]);

  // The English word at the end of the text, while the field has focus in رومن mode.
  const token = mode === 'roman' && focused && readOnly !== true ? trailingLatinRun(value) : '';
  const suggestions = useSuggestions(token, dismissedFor !== token);
  const open = token !== '' && dismissedFor !== token && suggestions.length > 0;
  const activeIndex = Math.min(highlight, Math.max(suggestions.length - 1, 0));

  // Esc closes the list and keeps the English letters (PRD FR-7.18, FR-7.22).
  useEscape(
    'overlay',
    () => {
      setDismissedFor(token);
    },
    open,
  );
  useLayoutEffect(() => {
    onPendingChange?.(token);
  });
  useLayoutEffect(() => {
    const caret = caretAfterInsert.current;
    if (caret === null) return;
    caretAfterInsert.current = null;
    inputRef.current?.setSelectionRange(caret, caret);
  });

  const setMode = (next: TypingMode) => {
    setOwnMode(next);
    onModeChange?.(next);
    setKeyMapOpen(false);
    setConversion(null);
  };

  /** Puts `text` at the cursor, replacing a selection, and keeps the cursor after it. */
  const insert = (text: string) => {
    const input = inputRef.current;
    const start = input?.selectionStart ?? value.length;
    const end = input?.selectionEnd ?? value.length;
    caretAfterInsert.current = start + text.length;
    onChange(value.slice(0, start) + text + value.slice(end));
  };

  /** Takes suggestion `index` for the English word at the end, with `trailing` after it. */
  const accept = (index: number, trailing: string): boolean => {
    const suggestion = suggestions[index];
    if (suggestion === undefined || token === '') return false;
    const chosen = suggestion.text;
    const roman = token.toLowerCase();
    // The pair is told first, so a name pair that fills its English name from this change already
    // knows the English letters of this word.
    onWordConverted?.({ roman, urdu: chosen, source: suggestion.source });
    onChange(value.slice(0, value.length - token.length) + chosen + trailing);
    setConversion({ roman: token, urdu: chosen, trailing });
    converted.current = [...converted.current, { roman, urdu: chosen }];
    // Another spelling than the first is remembered for next time (PRD FR-7.26). Learning is a
    // help: if it cannot be saved, typing goes on and nothing is shown.
    if (index !== 0) void translitLearn({ roman, urdu: chosen, kind }).catch(() => undefined);
    return true;
  };

  /**
   * When he leaves the field, a word he turned into Urdu and then changed by hand is a correction:
   * the pair (the English letters, his spelling) is remembered (PRD FR-7.26). Only when the words
   * are still the same number, so each one can be matched to its English letters.
   */
  const learnCorrections = () => {
    const words = wordsOf(value);
    const list = converted.current;
    if (list.length === 0 || words.length !== list.length) return;
    converted.current = list.map((word, index) => {
      const now = words[index];
      if (now === undefined || now === word.urdu || /[A-Za-z]/.test(now)) return word;
      void translitLearn({ roman: word.roman, urdu: now, kind }).catch(() => undefined);
      return { roman: word.roman, urdu: now };
    });
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (readOnly === true) return;
    if (matchesShortcut(event.nativeEvent, SHORTCUTS.typingMode)) {
      event.preventDefault();
      event.stopPropagation();
      setMode(nextTypingMode(mode));
      return;
    }
    if (event.ctrlKey || event.altKey || event.metaKey) return;
    if (mode === 'keyboard') {
      const letter = mapPhoneticKey(event.key);
      if (letter !== null) {
        event.preventDefault();
        insert(letter);
      }
      return;
    }
    if (mode !== 'roman') return;

    const input = event.currentTarget;
    // Backspace straight after a conversion brings back the English letters of that word.
    if (event.key === 'Backspace' && conversion !== null) {
      const tail = conversion.urdu + conversion.trailing;
      const atEnd =
        input.selectionStart === input.selectionEnd && input.selectionEnd === value.length;
      if (atEnd && value.endsWith(tail)) {
        event.preventDefault();
        onChange(value.slice(0, value.length - tail.length) + conversion.roman);
        setConversion(null);
        converted.current = converted.current.slice(0, -1);
        return;
      }
    }
    if (!open) return;
    switch (event.key) {
      case 'ArrowDown':
      case 'ArrowUp': {
        event.preventDefault();
        event.stopPropagation();
        const step = event.key === 'ArrowDown' ? 1 : -1;
        setHighlight((activeIndex + step + suggestions.length) % suggestions.length);
        break;
      }
      case ' ':
        // Space takes the word and stays in the field for the next one.
        event.preventDefault();
        accept(activeIndex, ' ');
        break;
      case 'Enter':
        // Enter takes the word, then moves on as it always does.
        event.preventDefault();
        event.stopPropagation();
        accept(activeIndex, '');
        focusNextField(input);
        break;
      case 'Tab':
        accept(activeIndex, '');
        break;
      default:
        break;
    }
  };

  const onInputChange = (event: ChangeEvent<HTMLInputElement>) => {
    setConversion(null);
    setHighlight(0);
    onChange(event.target.value);
  };

  const fullName = l10n.getString(MODE_FULL_ID[mode]);
  const markedWords = marked === undefined ? [] : wordsStillIn(value, marked);
  return (
    // Urdu text is written right to left, so the badge is on the left. English free text in the
    // English app is the exception (`direction`).
    <div dir={direction}>
      <div className="relative">
        <input
          ref={inputRef}
          id={id}
          type="text"
          role="combobox"
          lang={direction === 'rtl' ? 'ur' : 'en'}
          value={value}
          placeholder={placeholder}
          readOnly={readOnly}
          aria-invalid={invalid}
          aria-describedby={describedBy}
          aria-expanded={open}
          aria-controls={listId}
          aria-autocomplete="list"
          aria-activedescendant={open ? `${listId}-${String(activeIndex)}` : undefined}
          autoComplete="off"
          spellCheck={false}
          onChange={onInputChange}
          onKeyDown={onKeyDown}
          onFocus={() => {
            setFocused(true);
          }}
          onBlur={() => {
            // Leaving the field takes the highlighted spelling (PRD FR-7.22).
            if (open) accept(activeIndex, '');
            else learnCorrections();
            setFocused(false);
          }}
          className={`${controlClasses({ invalid, readOnly })} type-field ${mode === 'keyboard' ? 'pe-3 ps-field-badge-keys' : 'pe-3 ps-field-badge'}`}
        />
        <div className="absolute inset-y-0 start-2 flex items-center gap-1">
          {mode === 'keyboard' ? (
            <button
              type="button"
              tabIndex={-1}
              aria-label={l10n.getString('translit-keymap-open')}
              aria-pressed={keyMapOpen}
              title={l10n.getString('translit-keymap-open')}
              onMouseDown={(event) => {
                event.preventDefault();
              }}
              onClick={() => {
                setKeyMapOpen((current) => !current);
                inputRef.current?.focus();
              }}
              className="flex size-control-compact items-center justify-center rounded-1 text-ink-muted"
            >
              <Keyboard aria-hidden="true" className="size-icon-md" />
            </button>
          ) : null}
          <button
            type="button"
            tabIndex={-1}
            aria-label={`${l10n.getString('translit-mode-label')}: ${fullName}`}
            title={fullName}
            onMouseDown={(event) => {
              event.preventDefault();
            }}
            onClick={() => {
              setMode(nextTypingMode(mode));
              inputRef.current?.focus();
            }}
            disabled={readOnly === true}
            className="min-h-6 type-chip rounded-1 bg-surface-sunken px-2 text-ink"
          >
            {l10n.getString(MODE_BADGE_ID[mode])}
          </button>
        </div>
        {open ? (
          <ul
            id={listId}
            role="listbox"
            aria-label={l10n.getString('translit-suggestions-label')}
            className="absolute inset-x-0 top-full z-20 mt-1 flex flex-col gap-1 rounded-2 border border-border bg-surface p-1 shadow-overlay"
          >
            {suggestions.map((item, index) => (
              <li
                key={item.text}
                id={`${listId}-${String(index)}`}
                role="option"
                aria-selected={index === activeIndex}
                lang="ur"
                onMouseDown={(event) => {
                  // Taking a spelling with the mouse keeps the focus in the field.
                  event.preventDefault();
                  accept(index, '');
                }}
                className={`type-field cursor-pointer rounded-1 px-3 ${index === activeIndex ? 'bg-primary-soft text-primary' : 'text-ink'}`}
              >
                {item.text}
              </li>
            ))}
          </ul>
        ) : null}
        {keyMapOpen && mode === 'keyboard' ? (
          <KeyMap
            onPick={(letter) => {
              insert(letter);
              inputRef.current?.focus();
            }}
            onClose={() => {
              setKeyMapOpen(false);
            }}
          />
        ) : null}
      </div>
      {markedWords.length === 0 ? null : (
        <div className="mt-2">
          <SpellingMark words={markedWords} lang="ur" />
        </div>
      )}
    </div>
  );
}
