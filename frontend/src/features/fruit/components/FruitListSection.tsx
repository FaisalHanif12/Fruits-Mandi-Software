// The list under the پھل form (PRD FR-16.12, FR-19.11): the entries, with the partner (or the
// guests), the item and the days as filters, today's by default. ↑ ↓ and Page Up and Page Down move
// between rows, Enter (or a double click) opens the row to edit it while its day is open, and the
// buttons above the list edit or cancel the row that has focus. A cancelled entry stays in the list,
// marked in words. The list loads page by page; the quantity and value under it are the backend's
// totals over the whole filter, never a sum of the rows on screen. Four states: loading, an error
// with a retry, nothing to list, and the rows. The print and Excel of this list come with R37 (D5).
//
// While the partner section is locked (PRD FR-17.11) the backend answers with `partners_locked`: the rows and
// totals are the fruit for the shop's guests only, and the code box stands where the partners' fruit would be
// (plan 80 R4-1). A request for one partner's fruit is refused with ERR_PARTNER_LOCKED, which the list's error
// state shows as the same box, never as a failure with a retry.

import { Localized, useLocalization } from '@fluent/react';
import { Apple, Printer, SearchX } from 'lucide-react';
import { useMemo, useState } from 'react';
import { pickedOf } from '../../../shared/accounts';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { PagedList } from '../../../shared/grid';
import { useDebouncedValue, useListWindow, type ListWindow } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  Button,
  Card,
  EmptyState,
  Field,
  FilterBar,
  FormSpan,
  ListWindowBar,
  PartnerLockedBox,
  Picker,
  Select,
  ShowClosedTick,
  type PickerValue,
  EntryActions,
} from '../../../shared/ui';
import { usePartnerList } from '../../partners';
import { useOpenReport } from '../../reports';
import { FRUIT_COLUMNS } from '../list-columns';
import { useFruitList } from '../hooks';
import { FOR_GUESTS, type FruitListInput, type FruitRow } from '../types';

/** The value of the "for" filter that means everyone. */
const EVERYONE = 'everyone';
/** A pause so a burst of key presses in the search box asks once. */
const SEARCH_DELAY_MS = 150;

/** What the two buttons show before a row is chosen: both off, nothing to say. */
const NO_ENTRY = {
  can_edit: false,
  can_cancel: false,
  locked_reason: null,
  needs_reopen: false,
} as const;

interface FruitListSectionProps {
  /** Today, `YYYY-MM-DD`: the list starts with today's entries. */
  today: string;
  /**
   * The day the list starts with, `YYYY-MM-DD`: the day of the entry the address opened, so that
   * entry is in the list. Default: today.
   */
  day?: string | undefined;
  /** The entry that has focus in the list, or null. */
  selectedId: number | null;
  onSelect: (row: FruitRow | null) => void;
  onEdit: (row: FruitRow) => void;
  onCancel: (row: FruitRow) => void;
}

export function FruitListSection({
  today,
  day,
  selectedId,
  onSelect,
  onEdit,
  onCancel,
}: FruitListSectionProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const partners = usePartnerList();
  const openReport = useOpenReport();
  // The list shows one month at a time, the latest first (PRD FR-19.11, plan 76 point 8). A list the address
  // opened on one entry starts on that entry's day, until he moves the window.
  const monthWindow = useListWindow({ kind: 'month', source: { kind: 'fruit' }, today });
  const startText = day === undefined ? null : isoToDisplayDate(day);
  const [dayOnly, setDayOnly] = useState(startText !== null && day !== today);
  const listWindow: ListWindow = {
    ...monthWindow,
    move: (direction) => {
      setDayOnly(false);
      monthWindow.move(direction);
    },
    goTo: (value) => {
      setDayOnly(false);
      monthWindow.goTo(value);
    },
    latest: () => {
      setDayOnly(false);
      monthWindow.latest();
    },
    onPageKey: (direction) => {
      setDayOnly(false);
      monthWindow.onPageKey(direction);
    },
  };
  const range =
    dayOnly && startText !== null ? { from: startText, to: startText } : monthWindow.range;
  const [forWhom, setForWhom] = useState(EVERYONE);
  const [item, setItem] = useState<PickerValue | null>(null);
  const [search, setSearch] = useState('');
  const [showFinished, setShowFinished] = useState(false);
  const searchText = useDebouncedValue(search.trim(), SEARCH_DELAY_MS);
  const partnerId = forWhom === EVERYONE || forWhom === FOR_GUESTS ? null : Number(forWhom);
  const input: Omit<FruitListInput, 'offset' | 'limit' | 'sort'> = {
    partner_id: partnerId,
    guests_only: forWhom === FOR_GUESTS,
    item_id: item?.id ?? null,
    source: null,
    from: range.from === '' ? null : range.from,
    to: range.to === '' ? null : range.to,
    status: null,
    show_finished: showFinished,
    search: searchText === '' ? null : searchText,
    // A window is one month: the bar under the list says which, so no month heading is drawn.
    group_by_month: false,
  };
  const list = useFruitList(input);
  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const columns = useMemo(() => FRUIT_COLUMNS, []);
  const selected = rows?.find((row) => row.id === selectedId) ?? null;
  const onlyToday =
    listWindow.isLatest &&
    !dayOnly &&
    forWhom === EVERYONE &&
    item === null &&
    searchText === '' &&
    !showFinished;
  const months = first === undefined || first.months.length === 0 ? undefined : first.months;

  const empty = onlyToday ? (
    <EmptyState
      icon={Apple}
      title={<Localized id="fruit-list-empty" />}
      help={<Localized id="fruit-list-empty-help" />}
    />
  ) : (
    <EmptyState
      icon={SearchX}
      title={<Localized id="fruit-list-no-match" />}
      help={<Localized id="fruit-list-no-match-help" />}
      action={
        <Button
          onClick={() => {
            listWindow.latest();
            setForWhom(EVERYONE);
            setItem(null);
            setSearch('');
            setShowFinished(false);
          }}
        >
          <Localized id="fruit-list-clear" />
        </Button>
      }
    />
  );

  return (
    <Card compact icon={Apple} title={<Localized id="fruit-list-title" />}>
      <div {...formNavigationProps}>
        <FilterBar
          screen="fruit.list"
          search={{ value: search, onChange: setSearch }}
          moreSet={item === null ? 0 : 1}
          more={
            <FormSpan span="two">
              <Field label={<Localized id="fruit-filter-item" />}>
                {(control) => (
                  <Picker
                    {...control}
                    canAdd={false}
                    allChoice
                    scope="item"
                    value={item}
                    placeholder={l10n.getString('fruit-filter-item-all')}
                    onChange={(hit) => {
                      setItem(hit === null ? null : pickedOf(hit));
                    }}
                  />
                )}
              </Field>
            </FormSpan>
          }
          extra={<ShowClosedTick checked={showFinished} onChange={setShowFinished} />}
        >
          <Field label={<Localized id="fruit-filter-for" />}>
            {(control) => (
              <Select
                {...control}
                value={forWhom}
                onChange={setForWhom}
                options={[
                  { value: EVERYONE, label: l10n.getString('fruit-filter-for-all') },
                  ...(partners.data ?? []).map((partner) => ({
                    value: String(partner.id),
                    label: displayName(partner, language),
                  })),
                  { value: FOR_GUESTS, label: l10n.getString('fruit-for-guests') },
                ]}
              />
            )}
          </Field>
        </FilterBar>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <EntryActions
          flags={selected ?? NO_ENTRY}
          onEdit={() => {
            if (selected !== null) onEdit(selected);
          }}
          onCancel={() => {
            if (selected !== null) onCancel(selected);
          }}
        />
        {/* The fruit register (R37, PRD FR-16.12) with the same days, partner or guests and item. */}
        {openReport === null ? null : (
          <Button
            onClick={() => {
              openReport('R37', {
                ...(range.from === '' ? {} : { from: range.from }),
                ...(range.to === '' ? {} : { to: range.to }),
                ...(partnerId === null ? {} : { partner: partnerId }),
                ...(forWhom === FOR_GUESTS ? { guests: true } : {}),
                ...(item === null ? {} : { item }),
              });
            }}
          >
            <Printer aria-hidden="true" className="size-icon-md" />
            <Localized id="fruit-print" />
          </Button>
        )}
        <Localized id="fruit-list-select-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      {first?.partners_locked === true ? <PartnerLockedBox /> : null}
      <PagedList<FruitRow>
        columns={columns}
        rows={rows}
        getRowId={(row) => String(row.id)}
        labelId="fruit-list-label"
        failedId="fruit-list-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        onPageKey={listWindow.onPageKey}
        sort={{ value: list.sort, onChange: list.setSort }}
        months={
          months === undefined
            ? undefined
            : { months, monthOf: (row) => row.entry_date.slice(0, 7) }
        }
        empty={empty}
        onOpen={(row) => {
          if (row.can_edit) onEdit(row);
          else onSelect(row);
        }}
        onFocusRow={(rowId) => {
          onSelect(rows?.find((row) => String(row.id) === rowId) ?? null);
        }}
        focusRowId={selectedId === null ? null : String(selectedId)}
        isInactive={(row) => row.status === 'cancelled'}
        totals={
          first === undefined
            ? undefined
            : (context) => ({
                labelField: 'for',
                values: {
                  for: context.text('glossary-meezan'),
                  qty: String(first.totals.qty),
                  value: context.money(first.totals.value_paisa),
                },
              })
        }
      />
      <ListWindowBar window={listWindow} />
    </Card>
  );
}
