// The list under the پھل form (PRD FR-16.12, FR-19.11): the entries, with the partner (or the
// guests), the item and the days as filters, today's by default. ↑ ↓ and Page Up and Page Down move
// between rows, Enter (or a double click) opens the row to edit it while its day is open, and the
// buttons above the list edit or cancel the row that has focus. A cancelled entry stays in the list,
// marked in words. The list loads page by page; the quantity and value under it are the backend's
// totals over the whole filter, never a sum of the rows on screen. Four states: loading, an error
// with a retry, nothing to list, and the rows. The print and Excel of this list come with R37 (D5).

import { Localized, useLocalization } from '@fluent/react';
import { Ban, Apple, Pencil, Printer, SearchX } from 'lucide-react';
import { useMemo, useState } from 'react';
import { pickedOf } from '../../../shared/accounts';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { PagedList } from '../../../shared/grid';
import { useDebouncedValue, usePeriod, type PeriodControl } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  Button,
  Card,
  EmptyState,
  Field,
  FilterBar,
  FormSpan,
  PeriodBox,
  Picker,
  Select,
  ShowClosedTick,
  type PickerValue,
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
  // The list opens on today and the Owner's last period is kept (plan 38 R4). A list the address
  // opened on one entry starts on that entry's day, until he chooses another period.
  const saved = usePeriod('fruit.list', 'today', today);
  const startText = day === undefined ? null : isoToDisplayDate(day);
  const [dayOnly, setDayOnly] = useState(startText !== null && day !== today);
  const period: PeriodControl =
    dayOnly && startText !== null
      ? {
          state: { kind: 'custom', from: startText, to: startText },
          range: { from: startText, to: startText },
          ready: true,
          change: (next) => {
            setDayOnly(false);
            saved.change(next);
          },
        }
      : saved;
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
    from: period.range.from === '' ? null : period.range.from,
    to: period.range.to === '' ? null : period.range.to,
    status: null,
    show_finished: showFinished,
    search: searchText === '' ? null : searchText,
    // A month heading is worth its row only when the list covers more than one day.
    group_by_month: period.state.kind !== 'today' && !dayOnly,
  };
  const list = useFruitList(input);
  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const columns = useMemo(() => FRUIT_COLUMNS, []);
  const selected = rows?.find((row) => row.id === selectedId) ?? null;
  const onlyToday =
    period.state.kind === 'today' &&
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
            period.change({ kind: 'today', from: '', to: '' });
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
    <Card compact title={<Localized id="fruit-list-title" />}>
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
          {period.ready ? <PeriodBox period={period} /> : null}
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
        <Button
          disabled={selected?.can_change !== true}
          onClick={() => {
            if (selected !== null) onEdit(selected);
          }}
        >
          <Pencil aria-hidden="true" className="size-icon-md" />
          <Localized id="fruit-edit" />
        </Button>
        <Button
          variant="danger"
          disabled={selected?.can_change !== true}
          onClick={() => {
            if (selected !== null) onCancel(selected);
          }}
        >
          <Ban aria-hidden="true" className="size-icon-md" />
          <Localized id="fruit-cancel" />
        </Button>
        {/* The fruit register (R37, PRD FR-16.12) with the same days, partner or guests and item. */}
        {openReport === null ? null : (
          <Button
            onClick={() => {
              openReport('R37', {
                ...(period.range.from === '' ? {} : { from: period.range.from }),
                ...(period.range.to === '' ? {} : { to: period.range.to }),
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
        sort={{ value: list.sort, onChange: list.setSort }}
        months={
          months === undefined
            ? undefined
            : { months, monthOf: (row) => row.entry_date.slice(0, 7) }
        }
        empty={empty}
        onOpen={(row) => {
          if (row.can_change) onEdit(row);
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
    </Card>
  );
}
