// The filter bar of a list (plan 38 G3, R2): one tidy row with a search box, the filters used every
// day, and a button "More filters" that opens the rare ones. The open or shut state is remembered
// for the screen, and the button says how many of the hidden filters are set, so a hidden filter
// never filters in secret. Everything behind "More filters" is reached by Tab and Enter like any
// other field (PRD 7.2). The row wraps by itself; the rare filters open in a `FormGrid` under it.

import { Localized, useLocalization } from '@fluent/react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import type { ReactNode } from 'react';
import { useRememberedFlag } from '../hooks';
import { Button } from './Button';
import { Field } from './Field';
import { FormGrid } from './FormGrid';
import { Num } from './Num';
import { TextInput } from './TextInput';

interface FilterBarSearch {
  value: string;
  onChange: (text: string) => void;
  /** The Fluent message of the grey hint in the box (default: a code, a name or a number). */
  placeholderId?: string;
  /** The Fluent message of the label of the box (default: "Search"). */
  labelId?: string;
}

interface FilterBarProps {
  /** The search box of the list (code, name or number). */
  search?: FilterBarSearch;
  /** The filters used every day, as cells of a `FormGrid` (fields, a `PeriodBox`). */
  children?: ReactNode;
  /** The rare filters, behind "More filters". */
  more?: ReactNode;
  /** How many of the filters behind "More filters" are set. */
  moreSet?: number;
  /** Keeps "More filters" open, for example when a filter behind it has a problem to show. */
  moreOpen?: boolean;
  /** Names the screen, so the open or shut state of "More filters" is kept for it. */
  screen: string;
  /** What sits at the end of the bar: a tick such as "show closed too". */
  extra?: ReactNode;
}

export function FilterBar({
  search,
  children,
  more,
  moreSet = 0,
  moreOpen = false,
  screen,
  extra,
}: FilterBarProps) {
  const { l10n } = useLocalization();
  const [kept, setOpen] = useRememberedFlag(`${screen}.more_filters`, false);
  // A hidden filter that is set must never filter in secret: the part stays open while one is set.
  const open = kept || moreSet > 0 || moreOpen;
  // The button of the rare filters, with how many of them are set.
  const moreButton = (
    <div className="flex min-h-control min-w-0 items-center gap-2">
      <Button
        onClick={() => {
          setOpen(!open);
        }}
        accessibleName={l10n.getString(open ? 'common-filters-less' : 'common-filters-more')}
      >
        {open ? (
          <ChevronUp aria-hidden="true" className="size-icon-md" />
        ) : (
          <ChevronDown aria-hidden="true" className="size-icon-md" />
        )}
        <Localized id={open ? 'common-filters-less' : 'common-filters-more'} />
      </Button>
      {moreSet > 0 ? (
        <Localized
          id="common-filters-set"
          vars={{ count: String(moreSet) }}
          elems={{ num: <Num /> }}
        >
          <span className="type-body text-ink-muted" />
        </Localized>
      ) : null}
    </div>
  );
  return (
    <div className="flex flex-col gap-3">
      {/* One row that wraps by itself: the search box takes what is left, each filter has a fixed
          width (`w-filter`), the button of the rare filters and the ticks follow. At 1024 px a bar
          of search, period and one more filter is one row (plan 38 G3, tech lead). */}
      <div className="flex flex-wrap items-end gap-x-3 gap-y-2">
        {search === undefined ? null : (
          <div className="min-w-0 flex-1 basis-mid">
            <Field label={<Localized id={search.labelId ?? 'common-search-label'} />}>
              {(control) => (
                <TextInput
                  {...control}
                  value={search.value}
                  onChange={search.onChange}
                  placeholder={l10n.getString(search.placeholderId ?? 'common-search-placeholder')}
                />
              )}
            </Field>
          </div>
        )}
        <div className="contents *:w-filter">{children}</div>
        {more === undefined ? null : moreButton}
        {extra === undefined ? null : (
          <div className="flex min-h-control flex-wrap items-center gap-x-4 gap-y-1">{extra}</div>
        )}
      </div>
      {more !== undefined && open ? <FormGrid columns={4}>{more}</FormGrid> : null}
    </div>
  );
}
