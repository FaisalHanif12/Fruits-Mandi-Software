// The picker behind every picker of the app: the master pickers (Picker) and the account picker
// (AccountPicker) are thin wrappers that give it a source of records (PRD FR-7.10, FR-7.16,
// FR-7.35, FR-7.40, FR-7.41, Code Standards F7, F40). Built on Downshift `useCombobox`.
//
// - A mouse click on the field opens the list of every record of the kind, recent first, then by
//   code (plan 32, R7). Arriving by keyboard (Enter, Tab, arrows) does not open it: the arrows
//   keep moving between fields, and typing or Alt+↓ opens the list (PRD 7.2).
// - Typing searches (shared/search). Both scripts are searched and a small spelling mistake is
//   forgiven, by the backend. The picker never converts English letters to Urdu (F41): the text
//   goes to the search exactly as typed.
// - A list that is long is read page by page and draws only the rows in view (B31, F34).
// - When nothing is found, or nothing has been added yet, the list shows a button that opens the
//   Index form of that kind above the screen (plan 32, R8, shared/ui/add-record.ts).
// - Keys (PRD FR-7.16): with the list open ↑ ↓ move through the matches, Enter picks the
//   highlighted one (the best match is highlighted at once) and moves on, Esc closes the list.
//   With the list closed ↑ ↓ move between fields like in any other field. Alt+↓ opens the full
//   list without typing, the recent records first. F4 adds a new record when the screen allows.
// - Esc order (FR-7.18): close the list, else bring back the chosen name, else leave the screen.
// - The chosen record belongs to the screen (`value`). The field shows its name for the current
//   language, so a language switch keeps the value and changes only the words (FR-7.33).
// - A record that is inactive is never offered. Leaving the field never picks anything by
//   itself. Emptying the field and leaving clears the value.
// - Long lists are windowed: only the rows in view are drawn (NFR-SPD-18).

import { Localized, useLocalization } from '@fluent/react';
import { useCombobox, type UseComboboxStateChange } from 'downshift';
import { ChevronDown } from 'lucide-react';
import {
  useEffect,
  useId,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
  type ReactNode,
  type UIEvent,
} from 'react';
import { uiErrorOf } from '../api';
import { tokenPx } from '../hooks/tokenPx';
import { displayName, otherScriptLang, otherScriptName, useLanguage, type Named } from '../i18n';
import { SHORTCUTS, focusNextField, keepKeyForApp, matchesShortcut, useEscape } from '../keyboard';
import {
  PICKER_LOAD_AHEAD_ROWS,
  useFetchHits,
  usePickerHits,
  useTouchRecord,
  type PickerSource,
} from '../search';
import { usePickerFlag, usePickerText } from './usePickerText';
import { Button } from './Button';
import { Code } from './Code';
import { controlClasses } from './control-classes';
import { ErrorMessage } from './ErrorMessage';
import type { FieldControlProps } from './Field';

/** A record the screen already holds, for example the city of a saved truck stand. */
export interface PickerValue extends Named {
  id: number;
  code?: string | undefined;
}

/** What a picker row needs of a record: both names, a code, a line to tell two records apart. */
export interface PickerHit extends Named {
  id: number;
  code: string;
  extra_ur: string;
  extra_en: string;
  is_active: boolean;
  /**
   * A person in the dead amount (PRD FR-10.10). He is inactive but the receipt picker and the
   * ادھار رقم واپسی picker still offer him, marked "ڈوبی". Only an account hit has it.
   */
  is_dead?: boolean | undefined;
}

/** A row the picker may offer: an active record, or a person in the dead amount. */
const isOffered = (hit: PickerHit) => hit.is_active || hit.is_dead === true;

export interface PickerBaseProps<H extends PickerHit> extends FieldControlProps {
  /** Where the records come from (a master list, the accounts). */
  source: PickerSource<H>;
  value: PickerValue | null;
  /** The record chosen, or null when the field was emptied. */
  onChange: (hit: H | null) => void;
  placeholder?: string | undefined;
  /** Called with the typed text on F4 and from the button of an empty result. Leave out to hide both. */
  onAddNew?: ((text: string) => void) | undefined;
  /**
   * The button of an empty result opens the Index form with this (plan 32, R8). `done` is called
   * when that form closes, saved or not; the picker then takes the focus back with the same text
   * and opens its list. Without it the button uses `onAddNew`, and F4 is not changed by it.
   */
  onAddRecord?: ((text: string, done: () => void) => void) | undefined;
  /** False hides the button of an empty result (a filter has nothing to add). F4 is not changed. */
  showAdd?: boolean | undefined;
  /** The Fluent id of the button's words, for example `search-add-seller`. Default: "Add new". */
  addLabelId?: string | undefined;
  /** Something at the end of each row, for example a balance. */
  renderEnd?: ((hit: H) => ReactNode) | undefined;
  /** The record the list highlights now, or null: a screen can preview it (the party card). */
  onHighlight?: ((hit: H | null) => void) | undefined;
  /** The user left the field with this text typed and nothing picked (a short code, FR-7.39). */
  onLeave?: ((typed: string) => void) | undefined;
  /** Enter was pressed with this text typed and no match in the list. */
  onEnterNoMatch?: ((typed: string) => void) | undefined;
  /**
   * Enter was pressed while the list is an older answer than the text in the field (a fast
   * typist). Return true if the whole text is taken from here (the account picker sends a code to
   * the backend); return false and the picker waits for the answer to the whole text and picks its
   * best match then. Either way the older list is never picked from.
   */
  resolveOnEnter?: ((typed: string) => boolean) | undefined;
  /** The user typed in the field: a message about an earlier try can go. */
  onTyping?: (() => void) | undefined;
  /** Shown under the list's rows, inside the popup. */
  footer?: ReactNode;
  /**
   * The field opens with this text already typed and its list open: a picker cell of a grid that
   * was started by a key (the key is the first letter, PRD FR-7.14).
   */
  initialText?: string | undefined;
  /** The field opens with its full list open, nothing typed (Alt+↓ on a grid cell, FR-7.16). */
  startOpen?: boolean | undefined;
  /**
   * Enter picked a record. Called instead of moving on to the next field of the form. A picker
   * inside a grid cell uses it to finish the cell and go on to the next Enter stop (FR-7.14).
   */
  onEnterPicked?: (() => void) | undefined;
}

/** The sizes the style sheet gives the list, used when it cannot be read (tests have no CSS). */
const FALLBACK_ROW_PX = 72;
const FALLBACK_LIST_PX = 360;
/** Rows drawn above and below the view, so scrolling never shows a gap. */
const OVERSCAN_ROWS = 3;

/** Row height and list height as drawn, from the same tokens the classes use. */
function readListSizes() {
  return {
    rowPx: tokenPx('--picker-row-h', FALLBACK_ROW_PX),
    listPx: tokenPx('--picker-list-h', FALLBACK_LIST_PX),
  };
}

export function PickerBase<H extends PickerHit>({
  id,
  'aria-describedby': describedBy,
  invalid,
  readOnly,
  source,
  value,
  onChange,
  placeholder,
  onAddNew,
  onAddRecord,
  showAdd = true,
  addLabelId = 'search-add-new',
  renderEnd,
  onHighlight,
  onLeave,
  onEnterNoMatch,
  resolveOnEnter,
  onTyping,
  footer,
  initialText,
  startOpen,
  onEnterPicked,
}: PickerBaseProps<H>) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const listId = useId();
  // The list element, kept in state so the scroll effect can read it without touching a ref in render.
  const [listEl, setListEl] = useState<HTMLUListElement | null>(null);
  const { typed, setTyped: setTypedNow, readTyped } = usePickerText(initialText ?? null);
  const [open, setOpen] = useState(initialText !== undefined || startOpen === true);
  const [focused, setFocused] = useState(false);
  const [scrollTop, setScrollTop] = useState(0);
  const [{ rowPx, listPx }] = useState(readListSizes);

  const touch = useTouchRecord(source);
  const {
    rows,
    isPending,
    isError,
    error: queryError,
    current,
    hasMore,
    loadMore,
  } = usePickerHits(source, typed ?? '', open && readOnly !== true);
  const fetchHits = useFetchHits(source);
  const hits = useMemo(() => rows.filter(isOffered), [rows]);
  const boxRef = useRef<HTMLDivElement | null>(null);
  // True while the Index form of the "add" button is open above the screen. The field loses focus
  // to it, and that must not drop what was typed or send it as a code (plan 32, R8).
  const adding = usePickerFlag();

  const shown = typed ?? (value === null ? '' : displayName(value, language));

  const closeList = () => {
    setOpen(false);
    // A closed list forgets how far it was scrolled; the next one opens at the top.
    setScrollTop(0);
  };

  const pick = (hit: H) => {
    onChange(hit);
    touch.mutate(hit.id);
    setTypedNow(null);
    closeList();
  };

  const { highlightedIndex, getInputProps, getItemProps, getMenuProps, getToggleButtonProps } =
    useCombobox<H>({
      items: hits,
      ...(id === undefined ? {} : { inputId: id }),
      menuId: listId,
      itemToString: (hit) => (hit === null ? '' : displayName(hit, language)),
      // The value belongs to the screen; picking tells it through onChange.
      selectedItem: null,
      inputValue: shown,
      isOpen: open,
      defaultHighlightedIndex: 0,
      // The list scrolls itself (windowed rows), so Downshift's own scrolling is off.
      scrollIntoView: () => undefined,
      stateReducer: (state, { type, changes }) => {
        // Leaving the field never picks the highlighted record by itself.
        if (type === useCombobox.stateChangeTypes.InputBlur) {
          return { ...changes, selectedItem: state.selectedItem, inputValue: state.inputValue };
        }
        // A click on the field opens the list and never closes it again (plan 32, R7): the list
        // closes with Esc, a pick, or leaving the field. The chevron button is the one that toggles.
        if (type === useCombobox.stateChangeTypes.InputClick) {
          return {
            ...changes,
            isOpen: true,
            highlightedIndex:
              (state.isOpen ? state.highlightedIndex : changes.highlightedIndex) ?? 0,
          };
        }
        return changes;
      },
      onIsOpenChange: ({ isOpen }: UseComboboxStateChange<H>) => {
        // A field that cannot be changed (read only) opens no list, whatever is clicked.
        if (isOpen) {
          if (readOnly !== true) setOpen(true);
        } else closeList();
      },
      onSelectedItemChange: ({ selectedItem }: UseComboboxStateChange<H>) => {
        if (selectedItem !== null && selectedItem !== undefined) pick(selectedItem);
      },
    });

  // A screen can show the record the list highlights while the user is still typing.
  const highlightRef = useRef(onHighlight);
  useEffect(() => {
    highlightRef.current = onHighlight;
  });
  const highlighted = open && highlightedIndex >= 0 ? (hits[highlightedIndex] ?? null) : null;
  useEffect(() => {
    highlightRef.current?.(highlighted);
  }, [highlighted]);

  // Esc closes the list first, then brings back the chosen name (FR-7.18). Downshift answers the
  // key in the field itself; this entry lets a dialog around the field know a list is open, so
  // the dialog stays until the next Esc.
  useEscape(
    'overlay',
    () => {
      closeList();
    },
    open,
  );
  useEscape(
    'edit',
    () => {
      setTypedNow(null);
    },
    () => focused && typed !== null && !open,
  );

  // Keep the highlighted row in view while ↑ ↓ move through a long list.
  useLayoutEffect(() => {
    const list = listEl;
    if (!open || list === null || highlightedIndex < 0 || list.clientHeight === 0) return;
    const top = highlightedIndex * rowPx;
    const bottom = top + rowPx;
    if (top < list.scrollTop) list.scrollTo({ top });
    else if (bottom > list.scrollTop + list.clientHeight) {
      list.scrollTo({ top: bottom - list.clientHeight });
    }
  }, [highlightedIndex, listEl, open, rowPx]);

  const visibleRows = Math.ceil(listPx / rowPx) + 1;
  const first = Math.max(Math.floor(scrollTop / rowPx) - OVERSCAN_ROWS, 0);
  const last = Math.min(first + visibleRows + 2 * OVERSCAN_ROWS, hits.length);

  // After Enter has picked: the next field of the form, or what the screen says (a grid cell).
  const advance = (field: HTMLElement) => {
    if (onEnterPicked !== undefined) onEnterPicked();
    else focusNextField(field);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (readOnly === true) return;
    if (matchesShortcut(event.nativeEvent, SHORTCUTS.openList)) {
      // Alt+↓: the full list without typing, recent records first (FR-7.16, FR-7.41).
      event.preventDefault();
      keepKeyForApp(event);
      setOpen(true);
      return;
    }
    if (onAddNew !== undefined && matchesShortcut(event.nativeEvent, SHORTCUTS.addAccount)) {
      event.preventDefault();
      event.stopPropagation();
      keepKeyForApp(event);
      closeList();
      onAddNew(typed ?? '');
      return;
    }
    if (event.altKey || event.ctrlKey || event.metaKey) return;
    if ((event.key === 'ArrowDown' || event.key === 'ArrowUp') && !open) {
      // A closed picker is a field like any other: the arrows go on to the form (FR-7.16).
      keepKeyForApp(event);
      return;
    }
    if (event.key === 'Escape' && !open) {
      // Closed: the Esc order decides (bring back the name, else leave the screen), not Downshift.
      keepKeyForApp(event);
      return;
    }
    if (event.key === 'Enter' && open) {
      keepKeyForApp(event);
      event.preventDefault();
      if (!current) {
        // The list is an older answer than the text in the field: never pick a row from it. The
        // whole text goes on from here, or its own answer is asked for and picked when it comes
        // (PRD FR-7.14, FR-7.39). If he types more, or leaves, in the meantime, nothing is picked.
        const text = (readTyped() ?? '').trim();
        if (resolveOnEnter?.(text) === true) {
          closeList();
          return;
        }
        const field = event.currentTarget;
        void fetchHits(text)
          .then((answer) => {
            if ((readTyped() ?? '').trim() !== text) return;
            const best = answer.find(isOffered);
            if (best === undefined) {
              closeList();
              onEnterNoMatch?.(text);
              return;
            }
            pick(best);
            advance(field);
          })
          .catch(() => undefined);
        return;
      }
      const hit = hits[highlightedIndex];
      if (hit === undefined) {
        closeList();
        onEnterNoMatch?.(typed ?? '');
        return;
      }
      pick(hit);
      // Enter picks and goes on to the next field, as it does in every field (FR-7.14).
      advance(event.currentTarget);
    }
  };

  // The "add" button: the Index form above the screen, or the small form of the screen's own.
  const addFromList = () => {
    const text = typed ?? '';
    closeList();
    if (onAddRecord === undefined) {
      onAddNew?.(text);
      return;
    }
    adding.set(true);
    onAddRecord(text, () => {
      // Radix gives the focus back to the field after the form has closed; take it from there.
      window.setTimeout(() => {
        adding.set(false);
        const field = boxRef.current?.querySelector('input');
        if (!field?.isConnected) return;
        setTypedNow(text);
        field.focus();
        setOpen(true);
      }, 0);
    });
  };

  const onBlur = () => {
    setFocused(false);
    if (adding.get()) {
      closeList();
      return;
    }
    // Emptying the field and leaving clears the value; any other typing is dropped. The text is
    // read as the last key left it: after a pick it is already gone, so nothing is resolved twice.
    const left = readTyped();
    if (left !== null && left.trim() === '' && value !== null) onChange(null);
    else if (left !== null && left.trim() !== '') onLeave?.(left.trim());
    setTypedNow(null);
    closeList();
  };

  const onScroll = (event: UIEvent<HTMLUListElement>) => {
    setScrollTop(event.currentTarget.scrollTop);
  };

  // Near the end of what has been read, the next page is asked for (B31, F34).
  const nearEnd = last >= hits.length - PICKER_LOAD_AHEAD_ROWS;
  useEffect(() => {
    if (open && hasMore && nearEnd) loadMore();
  }, [open, hasMore, nearEnd, loadMore, hits.length]);

  const error = isError ? uiErrorOf(queryError) : null;
  const searching = open && isPending;
  const noMatch = open && !isPending && !isError && hits.length === 0;
  const canAdd = showAdd && (onAddRecord !== undefined || onAddNew !== undefined);

  const inputProps = getInputProps({
    type: 'text',
    readOnly,
    placeholder,
    'aria-invalid': invalid,
    'aria-describedby': describedBy,
    autoComplete: 'off',
    spellCheck: false,
    onKeyDown,
    onClick: (event: { currentTarget: HTMLInputElement }) => {
      // A click on a field that shows a chosen name selects the whole name, so typing
      // replaces it, as it does when the field is entered by keyboard (PRD FR-7.15).
      if (readOnly !== true && typed === null) event.currentTarget.select();
    },
    onFocus: () => {
      setFocused(true);
    },
    onBlur,
    onChange: (event: { currentTarget: HTMLInputElement }) => {
      if (readOnly !== true) {
        setTypedNow(event.currentTarget.value);
        onTyping?.();
      }
    },
  });

  return (
    <div ref={boxRef} className="relative">
      {/* A name wider than the box is cut with an ellipsis and read in full on hover (plan 38). */}
      <input
        {...inputProps}
        title={typeof inputProps.value === 'string' ? inputProps.value : undefined}
        className={`${controlClasses({ invalid, readOnly })} type-field pe-control text-ellipsis`}
      />
      {readOnly === true ? null : (
        <button
          {...getToggleButtonProps({
            tabIndex: -1,
            'aria-label': l10n.getString('search-open-list'),
            title: l10n.getString('search-open-list'),
          })}
          type="button"
          className="absolute inset-y-0 end-1 flex size-control-compact items-center justify-center rounded-1 text-ink-muted"
        >
          <ChevronDown aria-hidden="true" className="size-icon-md" />
        </button>
      )}
      <div
        className={
          open
            ? 'absolute inset-x-0 top-full z-20 mt-1 rounded-2 border border-border bg-surface p-1 shadow-overlay'
            : 'hidden'
        }
      >
        <ul
          {...getMenuProps({
            ref: setListEl,
            'aria-label': l10n.getString('search-list-label'),
            onScroll,
          })}
          className={`max-h-picker-list overflow-y-auto ${hits.length === 0 ? 'hidden' : ''}`}
        >
          {open && first > 0 ? (
            <li role="presentation" aria-hidden="true" style={{ height: first * rowPx }} />
          ) : null}
          {open
            ? hits.slice(first, last).map((hit, offset) => {
                const index = first + offset;
                const extra = displayName(
                  { name_ur: hit.extra_ur, name_en: hit.extra_en },
                  language,
                );
                return (
                  <li
                    key={hit.id}
                    {...getItemProps({ item: hit, index })}
                    aria-setsize={hits.length}
                    aria-posinset={index + 1}
                    className={`flex h-picker-row cursor-pointer flex-col justify-center rounded-1 px-3 ${index === highlightedIndex ? 'bg-primary-soft text-primary' : 'text-ink'}`}
                  >
                    <span className="flex items-baseline justify-between gap-3">
                      <span className="type-field truncate">{displayName(hit, language)}</span>
                      <span className="flex shrink-0 items-baseline gap-2">
                        {hit.is_dead === true ? (
                          <Localized id="dead-tag">
                            <span className="type-chip rounded-1 bg-warn-soft px-2 text-warn" />
                          </Localized>
                        ) : null}
                        <Code>{hit.code}</Code>
                      </span>
                    </span>
                    <span className="flex items-baseline gap-2 truncate type-body text-ink-muted">
                      <span lang={otherScriptLang(language)} dir="auto" className="truncate">
                        {otherScriptName(hit, language)}
                      </span>
                      {extra === '' ? null : <span className="truncate">{extra}</span>}
                      {renderEnd === undefined ? null : (
                        <span className="ms-auto shrink-0">{renderEnd(hit)}</span>
                      )}
                    </span>
                  </li>
                );
              })
            : null}
          {open && last < hits.length ? (
            <li
              role="presentation"
              aria-hidden="true"
              style={{ height: (hits.length - last) * rowPx }}
            />
          ) : null}
        </ul>
        {searching ? (
          <Localized id="search-searching">
            <p role="status" className="px-3 py-2 type-body text-ink-muted" />
          </Localized>
        ) : null}
        {error === null ? null : (
          <div role="alert" className="flex flex-col gap-1 px-3 py-2 text-due">
            <Localized id="search-error">
              <p className="type-body" />
            </Localized>
            {error.errors.map((item, index) => (
              <ErrorMessage key={`${item.code}-${String(index)}`} item={item}>
                <p className="type-body" />
              </ErrorMessage>
            ))}
          </div>
        )}
        {noMatch ? (
          <div
            role="status"
            className="flex flex-col items-start gap-2 px-3 py-2"
            // A press on the button must not take focus from the field, or the list would close
            // before the click lands and the typed text would be lost.
            onMouseDown={(event) => {
              event.preventDefault();
            }}
          >
            <Localized id={(typed ?? '').trim() === '' ? 'search-empty' : 'search-no-match'}>
              <p className="type-body text-ink-muted" />
            </Localized>
            {canAdd ? (
              <Button
                // F4 is shown on the button only where F4 does the same (the small form).
                {...(onAddRecord === undefined ? { shortcut: SHORTCUTS.addAccount } : {})}
                onClick={addFromList}
              >
                <Localized id={addLabelId} />
              </Button>
            ) : null}
          </div>
        ) : null}
        {footer}
      </div>
    </div>
  );
}
