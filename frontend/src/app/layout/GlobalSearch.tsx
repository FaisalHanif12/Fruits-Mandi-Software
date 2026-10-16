// The search box of the top bar (PRD FR-5.4, FR-5.6): it finds an account by code, short code, Urdu
// or English name, father or firm, city or phone, in either script, and opens it. `Ctrl+K` focuses
// it from anywhere. ↑ ↓ move through the results and Enter opens the highlighted one; Esc clears
// the box. A result is an account, or (plan 13) a بکری found by its number or by the start of its
// truck number; vouchers come with their module. Like every search, it takes the text as typed and
// never converts it to Urdu (F41).

import { Localized, useLocalization } from '@fluent/react';
import { useCombobox } from 'downshift';
import { Search } from 'lucide-react';
import { useId, useState, type KeyboardEvent } from 'react';
import {
  useGlobalSearch,
  type AccountSearchHit,
  type BikriSearchHit,
  type GlobalHit,
} from '../../shared/accounts';
import { uiErrorOf } from '../../shared/api';
import { displayName, otherScriptLang, otherScriptName, useLanguage } from '../../shared/i18n';
import { SHORTCUTS, keepKeyForApp, useEscape, useShortcut } from '../../shared/keyboard';
import { isoToDisplayDate } from '../../shared/format';
import type { Language } from '../../i18n';
import { Code, ErrorMessage, KeyChip } from '../../shared/ui';

interface GlobalSearchProps {
  /** A result was chosen: open that account. */
  onOpenAccount: (id: number) => void;
  /** A بکری was chosen: open the sale book on that id (read-only until plan 16). */
  onOpenBikri: (id: number) => void;
}

/** The text a result is known by in the box. */
function hitText(hit: GlobalHit, language: Language): string {
  return hit.kind === 'bikri' ? String(hit.number) : displayName(hit, language);
}

/** An account result: its name in the app language, its code, the other script and the city. */
function AccountResult({ hit, language }: { hit: AccountSearchHit; language: Language }) {
  return (
    <>
      <span className="flex items-baseline justify-between gap-3">
        <span className="type-field truncate">{displayName(hit, language)}</span>
        <Code>{hit.code}</Code>
      </span>
      <span className="flex items-baseline gap-2 truncate type-body text-ink-muted">
        <span lang={otherScriptLang(language)} dir="auto" className="truncate">
          {otherScriptName(hit, language)}
        </span>
        <span className="truncate">
          {displayName({ name_ur: hit.extra_ur, name_en: hit.extra_en }, language)}
        </span>
      </span>
    </>
  );
}

/** A بکری result: "بکری 12", its day, its truck number and, if cancelled, the word (FR-5.4). */
function BikriResult({ hit }: { hit: BikriSearchHit }) {
  return (
    <>
      <span className="flex items-baseline justify-between gap-3">
        <Localized id="bikri-hit-title" vars={{ number: String(hit.number) }}>
          <span className="type-field truncate" />
        </Localized>
        <Code>{isoToDisplayDate(hit.entry_date)}</Code>
      </span>
      <span className="flex items-baseline gap-2 truncate type-body text-ink-muted">
        {hit.truck_no === null ? null : <Code>{hit.truck_no}</Code>}
        {hit.status === 'cancelled' ? (
          <Localized id="bikri-hit-cancelled">
            <span className="text-due" />
          </Localized>
        ) : null}
      </span>
    </>
  );
}

export function GlobalSearch({ onOpenAccount, onOpenBikri }: GlobalSearchProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const listId = useId();
  // The input element, kept in state so no ref is read while drawing.
  const [inputEl, setInputEl] = useState<HTMLInputElement | null>(null);
  const [text, setText] = useState('');
  const [focused, setFocused] = useState(false);
  const query = useGlobalSearch(text);
  const typed = text.trim() !== '';
  const hits = typed ? (query.data ?? []) : [];
  const open = focused && typed;

  useShortcut(SHORTCUTS.globalSearch, () => {
    inputEl?.focus();
  });
  // Esc clears the box (PRD FR-7.18). The list closes with it, because it shows only while
  // there is text. The Esc order is the shared one (F27); the box has no key handler of its own.
  useEscape(
    'edit',
    () => {
      setText('');
    },
    () => focused && text !== '',
  );

  const { highlightedIndex, getInputProps, getItemProps, getMenuProps } = useCombobox<GlobalHit>({
    items: hits,
    menuId: listId,
    itemToString: (hit) => (hit === null ? '' : hitText(hit, language)),
    selectedItem: null,
    inputValue: text,
    isOpen: open,
    defaultHighlightedIndex: 0,
    scrollIntoView: () => undefined,
    onInputValueChange: ({ inputValue }) => {
      setText(inputValue);
    },
    onSelectedItemChange: ({ selectedItem }) => {
      if (selectedItem === null) return;
      if (selectedItem.kind === 'bikri') onOpenBikri(selectedItem.id);
      else onOpenAccount(selectedItem.id);
      setText('');
      inputEl?.blur();
    },
  });

  const error = query.isError ? uiErrorOf(query.error) : null;
  return (
    <div className="relative max-w-reading flex-1">
      <div className="flex h-control-compact items-center gap-2 rounded-2 border border-border-control bg-bg px-3">
        <Search aria-hidden="true" className="size-icon-sm shrink-0 text-ink-muted" />
        <input
          {...getInputProps({
            ref: setInputEl,
            type: 'search',
            'aria-label': l10n.getString('layout-search-label'),
            placeholder: l10n.getString('layout-search-placeholder'),
            autoComplete: 'off',
            spellCheck: false,
            onFocus: () => {
              setFocused(true);
            },
            onBlur: () => {
              setFocused(false);
            },
            // Esc is answered by the shared Esc order above, not by Downshift.
            onKeyDown: (event: KeyboardEvent<HTMLInputElement>) => {
              if (event.key === 'Escape') keepKeyForApp(event);
            },
          })}
          className="type-field min-w-0 flex-1 self-stretch truncate bg-transparent text-ink placeholder:text-ink-placeholder"
        />
        <KeyChip shortcut={SHORTCUTS.globalSearch} tone="on-light" />
      </div>
      <div
        className={
          open
            ? 'absolute inset-x-0 top-full z-30 mt-1 rounded-2 border border-border bg-surface p-1 shadow-overlay'
            : 'hidden'
        }
      >
        <ul
          {...getMenuProps({ 'aria-label': l10n.getString('layout-search-results') })}
          className={`max-h-picker-list overflow-y-auto ${hits.length === 0 ? 'hidden' : ''}`}
        >
          {open
            ? hits.map((hit, index) => (
                <li
                  key={`${hit.kind}-${String(hit.id)}`}
                  {...getItemProps({ item: hit, index })}
                  className={`flex min-h-control cursor-pointer flex-col justify-center rounded-1 px-3 ${index === highlightedIndex ? 'bg-primary-soft text-primary' : 'text-ink'}`}
                >
                  {hit.kind === 'bikri' ? (
                    <BikriResult hit={hit} />
                  ) : (
                    <AccountResult hit={hit} language={language} />
                  )}
                </li>
              ))
            : null}
        </ul>
        {open && query.isPending ? (
          <Localized id="layout-search-searching">
            <p role="status" className="px-3 py-2 type-body text-ink-muted" />
          </Localized>
        ) : null}
        {open && error !== null ? (
          <div role="alert" className="flex flex-col gap-1 px-3 py-2 text-due">
            <Localized id="layout-search-failed">
              <p className="type-body" />
            </Localized>
            {error.errors.map((item, index) => (
              <ErrorMessage key={`${item.code}-${String(index)}`} item={item}>
                <p className="type-body" />
              </ErrorMessage>
            ))}
          </div>
        ) : null}
        {open && !query.isPending && error === null && hits.length === 0 ? (
          <Localized id="layout-search-none">
            <p role="status" className="px-3 py-2 type-body text-ink-muted" />
          </Localized>
        ) : null}
      </div>
    </div>
  );
}
