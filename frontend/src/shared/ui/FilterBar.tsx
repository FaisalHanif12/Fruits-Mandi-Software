// The filter bar of a list (plan 38 G3, R2): one tidy row with a search box, the filters used every
// day, and a button "More filters" that opens the rare ones. The open or shut state is remembered
// for the screen, and the button says how many of the hidden filters are set, so a hidden filter
// never filters in secret. Everything behind "More filters" is reached by Tab and Enter like any
// other field (PRD 7.2). Cells go in a `FormGrid`, so the row follows the free width.

import { Localized, useLocalization } from '@fluent/react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import type { ReactNode } from 'react';
import { useRememberedFlag } from '../hooks';
import { Button } from './Button';
import { Field } from './Field';
import { FormGrid, FormSpan } from './FormGrid';
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
  /** Names the screen, so the open or shut state of "More filters" is kept for it. */
  screen: string;
  /** What sits at the end of the bar: a tick such as "show closed too". */
  extra?: ReactNode;
}

export function FilterBar({ search, children, more, moreSet = 0, screen, extra }: FilterBarProps) {
  const { l10n } = useLocalization();
  const [kept, setOpen] = useRememberedFlag(`${screen}.more_filters`, false);
  // A hidden filter that is set must never filter in secret: the part stays open while one is set.
  const open = kept || moreSet > 0;
  return (
    <div className="flex flex-col gap-3">
      <FormGrid columns={4} alignEnd>
        {search === undefined ? null : (
          <FormSpan span="two">
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
          </FormSpan>
        )}
        {children}
        {more === undefined ? null : (
          <div className="flex min-w-0 items-center gap-2">
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
        )}
      </FormGrid>
      {more !== undefined && open ? <FormGrid columns={4}>{more}</FormGrid> : null}
      {extra === undefined ? null : (
        <div className="flex flex-wrap items-center gap-4">{extra}</div>
      )}
    </div>
  );
}
