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
// - Add it here (plan 41, PRD FR-7.69): an entry picker ends its open list with one row, "+ New
//   expense head", pinned under the rows. When text was typed, the row names it: "+ «Electricity»
//   as a new expense head". A click on it, ↓ then Enter, or F4 in the field opens the short form of
//   that kind over the screen (shared/ui/add-record.ts). The record that is saved is put in the
//   field at once. An empty list says so in words ("No expense heads yet"), above the row. A
//   filter box has no such row.
// - Keys (PRD FR-7.16): with the list open ↑ ↓ move through the matches, Enter picks the
//   highlighted one (the best match is highlighted at once) and moves on, Esc closes the list.
//   ↓ on the last match moves to the add row, and Enter there adds. Enter on text that matches
//   nothing (and is not a code) adds it too. With the list closed ↑ ↓ move between fields like in
//   any other field. Alt+↓ opens the full list without typing, the recent records first.
// - Esc order (FR-7.18): close the list, else bring back the chosen name, else leave the screen.
// - The chosen record belongs to the screen (`value`). The field shows its name for the current
//   language, so a language switch keeps the value and changes only the words (FR-7.33).
// - A record that is inactive is never offered. Leaving the field never picks anything by
//   itself. Emptying the field and leaving clears the value.
// - Long lists are windowed: only the rows in view are drawn (NFR-SPD-18).

import { Localized, useLocalization } from '@fluent/react';
import { useCombobox, type UseComboboxStateChange } from 'downshift';
import { ChevronDown, Plus } from 'lucide-react';
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
import { Code } from './Code';
import { KeyChip } from './KeyChip';
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
  /**
   * The word for the kind of record this field takes, the key of the add row's and the empty
   * list's Fluent messages (`search-add-row`, `search-empty-kind`), for example `expense-head`.
   * Without it the field offers no add row: a filter box has nothing to add (plan 41, R5).
   */
  addNoun?: string | undefined;
  /**
   * Opens the short form of the kind (plan 41). The row, `F4` and `Enter` on text that matches
   * nothing call it with the typed text. `done` is called when the form closes: with the row of
   * the record that was saved, which the field then takes, or with null when the user left
   * without saving, and the field is as it was.
   */
  onAddRecord?: ((text: string, done: (added: H | null) => void) => void) | undefined;
  /**
   * A screen that opens its own form and puts the new record in the field itself (the person who
   * can borrow, a separate khata). Called with the typed text from the row and from `F4`. Used
   * only where `onAddRecord` is not given.
   */
  onAddNew?: ((text: string) => void) | undefined;
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
/** Only digits are a code or a short code (PRD FR-7.39). */
const CODE_TEXT = /^\d+$/;
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
  addNoun,
  onAddRecord,
  onAddNew,
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
  // The add row is the last stop of the list: ↓ on the last match moves to it (plan 41, R1). It is
  // not one of the matches, so Downshift does not know it and the matches keep their old meaning.
  const [addActive, setAddActive] = useState(false);
  const addRowId = `${listId}-add`;
  const canAdd =
    addNoun !== undefined &&
    readOnly !== true &&
    (onAddRecord !== undefined || onAddNew !== undefined);
  // The add row ends the open list. It waits for the first answer, so an empty list is known to
  // be empty, and for the last page, so it is the end of the list (plan 41, R1).
  const addShown = open && canAdd && !isPending && !isError && !hasMore;

  const shown = typed ?? (value === null ? '' : displayName(value, language));

  const closeList = () => {
    setOpen(false);
    setAddActive(false);
    // A closed list forgets how far it was scrolled; the next one opens at the top.
    setScrollTop(0);
  };

  const pick = (hit: H) => {
    onChange(hit);
    touch.mutate(hit.id);
    setTypedNow(null);
    closeList();
  };

  const {
    highlightedIndex,
    setHighlightedIndex,
    getInputProps,
    getItemProps,
    getMenuProps,
    getToggleButtonProps,
  } = useCombobox<H>({
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
          highlightedIndex: (state.isOpen ? state.highlightedIndex : changes.highlightedIndex) ?? 0,
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
    if (canAdd && matchesShortcut(event.nativeEvent, SHORTCUTS.addAccount)) {
      event.preventDefault();
      event.stopPropagation();
      keepKeyForApp(event);
      startAdd(typed ?? '', event.currentTarget);
      return;
    }
    if (event.altKey || event.ctrlKey || event.metaKey) return;
    if (open && addShown && event.key === 'ArrowDown') {
      // ↓ on the last match (or on an empty list) moves to the add row, and stays there.
      if (addActive || hits.length === 0 || highlightedIndex >= hits.length - 1) {
        event.preventDefault();
        keepKeyForApp(event);
        setHighlightedIndex(-1);
        setAddActive(true);
        return;
      }
    }
    if (open && addActive && event.key === 'ArrowUp') {
      event.preventDefault();
      keepKeyForApp(event);
      setAddActive(false);
      setHighlightedIndex(hits.length - 1);
      return;
    }
    if (open && addActive && event.key === 'Enter') {
      event.preventDefault();
      keepKeyForApp(event);
      startAdd(typed ?? '', event.currentTarget);
      return;
    }
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
              enterNoMatch(text, field);
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
        enterNoMatch(typed ?? '', event.currentTarget);
        return;
      }
      pick(hit);
      // Enter picks and goes on to the next field, as it does in every field (FR-7.14).
      advance(event.currentTarget);
    }
  };

  // Adds a record from the list: the short form over the screen, or the screen's own form. When
  // the form closes with a saved record, the field takes it and the cursor stays in the field, with
  // the name selected, so Enter carries on (plan 41, R4). Without one, the field is as it was.
  function startAdd(text: string, field: HTMLInputElement | null | undefined) {
    closeList();
    if (onAddRecord === undefined) {
      onAddNew?.(text);
      return;
    }
    adding.set(true);
    onAddRecord(text, (added) => {
      // Radix gives the focus back to the field after the form has closed; take it from there.
      window.setTimeout(() => {
        adding.set(false);
        if (!field?.isConnected) return;
        if (added === null) {
          setTypedNow(text);
          field.focus();
          setOpen(true);
          return;
        }
        pick(added);
        field.focus();
        // The screen shows the new name after it has taken the pick; select it then.
        window.setTimeout(() => {
          if (field.isConnected) field.select();
        }, 0);
      }, 0);
    });
  }

  // Enter on text that matches nothing. A code is the account picker's to resolve (FR-7.39); other
  // text is a new record, and the add form opens with it (plan 41, R1).
  function enterNoMatch(text: string, field: HTMLInputElement) {
    if (canAdd && text.trim() !== '' && !CODE_TEXT.test(text.trim())) startAdd(text, field);
    else onEnterNoMatch?.(text);
  }

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

  // What the add row names: the text typed, trimmed. Empty while the field is not being typed in.
  const addTyped = (typed ?? '').trim();

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
        setAddActive(false);
        onTyping?.();
      }
    },
  });

  return (
    <div ref={boxRef} className="relative">
      {/* A name wider than the box is cut with an ellipsis and read in full on hover (plan 38). */}
      <input
        {...inputProps}
        {...(addActive ? { 'aria-activedescendant': addRowId } : {})}
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
                    {...getItemProps({
                      item: hit,
                      index,
                      // The mouse over a match takes the highlight from the add row.
                      onMouseMove: () => {
                        setAddActive(false);
                      },
                    })}
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
          <div role="status" className="px-3 py-2">
            {(typed ?? '').trim() !== '' ? (
              <Localized id="search-no-match">
                <p className="type-body text-ink-muted" />
              </Localized>
            ) : addNoun === undefined ? (
              <Localized id="search-empty">
                <p className="type-body text-ink-muted" />
              </Localized>
            ) : (
              // The list says in words what it holds none of, for example "No expense heads yet".
              <Localized id="search-empty-kind" vars={{ noun: addNoun }}>
                <p className="type-body text-ink-muted" />
              </Localized>
            )}
          </div>
        ) : null}
        {addShown ? (
          <div
            className="mt-1 border-t border-border pt-1"
            // A press on the row must not take focus from the field, or the list would close
            // before the click lands and the typed text would be lost.
            onMouseDown={(event) => {
              event.preventDefault();
            }}
          >
            <button
              type="button"
              id={addRowId}
              tabIndex={-1}
              aria-current={addActive}
              onClick={() => {
                startAdd(typed ?? '', boxRef.current?.querySelector('input'));
              }}
              onMouseMove={() => {
                if (addActive) return;
                setHighlightedIndex(-1);
                setAddActive(true);
              }}
              className={`flex min-h-control w-full items-center gap-2 rounded-1 px-3 py-1 text-start type-field text-primary ${addActive ? 'bg-primary-soft' : ''}`}
            >
              <Plus aria-hidden="true" className="size-icon-md shrink-0" />
              <Localized
                id={addTyped === '' ? 'search-add-row' : 'search-add-row-named'}
                vars={{ noun: addNoun, text: addTyped }}
              >
                <span className="min-w-0 flex-1 text-balance break-words" />
              </Localized>
              <KeyChip shortcut={SHORTCUTS.addAccount} tone="on-light" />
            </button>
          </div>
        ) : null}
        {footer}
      </div>
    </div>
  );
}
