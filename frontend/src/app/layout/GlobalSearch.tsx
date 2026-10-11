// The search box of the top bar (PRD FR-5.4, FR-5.6): it finds an account by code, short code, Urdu
// or English name, father or firm, city or phone, in either script, and opens it. `Ctrl+K` focuses
// it from anywhere. ↑ ↓ move through the results and Enter opens the highlighted one; Esc clears
// the box. Accounts for now; the other kinds (بکری numbers, trucks, vouchers) come with their
// modules. Like every search, it takes the text as typed and never converts it to Urdu (F41).

import { Localized, useLocalization } from '@fluent/react';
import { useCombobox } from 'downshift';
import { Search } from 'lucide-react';
import { useId, useState } from 'react';
import { useGlobalSearch, type GlobalHit } from '../../shared/accounts';
import { uiErrorOf } from '../../shared/api';
import { displayName, otherScriptName, useLanguage } from '../../shared/i18n';
import { SHORTCUTS, useShortcut } from '../../shared/keyboard';
import { Code, ErrorMessage, KeyChip } from '../../shared/ui';

interface GlobalSearchProps {
  /** A result was chosen: open that account. */
  onOpenAccount: (id: number) => void;
}

export function GlobalSearch({ onOpenAccount }: GlobalSearchProps) {
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

  const { highlightedIndex, getInputProps, getItemProps, getMenuProps } = useCombobox<GlobalHit>({
    items: hits,
    menuId: listId,
    itemToString: (hit) => (hit === null ? '' : displayName(hit, language)),
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
      onOpenAccount(selectedItem.id);
      setText('');
      inputEl?.blur();
    },
  });

  const error = query.isError ? uiErrorOf(query.error) : null;
  return (
    <div className="relative max-w-xl flex-1">
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
            onKeyDown: (event: React.KeyboardEvent<HTMLInputElement>) => {
              // Esc clears the box (PRD FR-7.18); the list closes with it.
              if (event.key === 'Escape' && text !== '') {
                event.preventDefault();
                event.stopPropagation();
                setText('');
              }
            },
          })}
          className="type-field min-w-0 flex-1 bg-transparent text-ink placeholder:text-ink-placeholder"
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
                  key={hit.id}
                  {...getItemProps({ item: hit, index })}
                  className={`flex min-h-control cursor-pointer flex-col justify-center rounded-1 px-3 ${index === highlightedIndex ? 'bg-primary-soft text-primary' : 'text-ink'}`}
                >
                  <span className="flex items-baseline justify-between gap-3">
                    <span className="type-field truncate">{displayName(hit, language)}</span>
                    <Code>{hit.code}</Code>
                  </span>
                  <span className="flex items-baseline gap-2 truncate type-body text-ink-muted">
                    <span
                      lang={language === 'ur' ? 'en' : 'ur'}
                      dir={language === 'ur' ? 'ltr' : 'rtl'}
                      className="truncate"
                    >
                      {otherScriptName(hit, language)}
                    </span>
                    <span className="truncate">
                      {displayName({ name_ur: hit.extra_ur, name_en: hit.extra_en }, language)}
                    </span>
                  </span>
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
