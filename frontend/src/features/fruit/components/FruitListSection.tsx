// The list under the پھل form (PRD FR-16.12, FR-19.11): the entries, with the partner (or the
// guests), the item and the days as filters, today's by default. ↑ ↓ and Page Up and Page Down move
// between rows, Enter (or a double click) opens the row to edit it while its day is open, and the
// buttons above the list edit or cancel the row that has focus. A cancelled entry stays in the list,
// marked in words. The list loads page by page; the quantity and value under it are the backend's
// totals over the whole filter, never a sum of the rows on screen. Four states: loading, an error
// with a retry, nothing to list, and the rows. The print and Excel of this list come with R37 (D5).

import { Localized, useLocalization } from '@fluent/react';
import { Ban, Apple, Pencil, SearchX } from 'lucide-react';
import { useMemo, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { PagedList } from '../../../shared/grid';
import { displayName, useLanguage } from '../../../shared/i18n';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  Button,
  Card,
  DateInput,
  EmptyState,
  Field,
  Picker,
  Select,
  type PickerValue,
} from '../../../shared/ui';
import { usePartnerList } from '../../partners';
import { FRUIT_COLUMNS } from '../list-columns';
import { useFruitList } from '../hooks';
import { FOR_GUESTS, type FruitListInput, type FruitRow } from '../types';

/** The value of the "for" filter that means everyone. */
const EVERYONE = 'everyone';

interface FruitListSectionProps {
  /** Today, `YYYY-MM-DD`: the list starts with today's entries. */
  today: string;
  /** The entry that has focus in the list, or null. */
  selectedId: number | null;
  onSelect: (row: FruitRow | null) => void;
  onEdit: (row: FruitRow) => void;
  onCancel: (row: FruitRow) => void;
}

/** The first day of the month and of the year of `DD-MM-YYYY` text, for the period buttons. */
function monthStart(today: string): string {
  return `01-${today.slice(3)}`;
}
function yearStart(today: string): string {
  return `01-01-${today.slice(6)}`;
}

export function FruitListSection({
  today,
  selectedId,
  onSelect,
  onEdit,
  onCancel,
}: FruitListSectionProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const partners = usePartnerList();
  const todayText = isoToDisplayDate(today);
  const [from, setFrom] = useState(todayText);
  const [to, setTo] = useState(todayText);
  const [forWhom, setForWhom] = useState(EVERYONE);
  const [item, setItem] = useState<PickerValue | null>(null);
  const partnerId = forWhom === EVERYONE || forWhom === FOR_GUESTS ? null : Number(forWhom);
  const input: Omit<FruitListInput, 'offset' | 'limit'> = {
    partner_id: partnerId,
    guests_only: forWhom === FOR_GUESTS,
    item_id: item?.id ?? null,
    source: null,
    from: from.trim() === '' ? null : from.trim(),
    to: to.trim() === '' ? null : to.trim(),
    status: null,
  };
  const list = useFruitList(input);
  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const columns = useMemo(() => FRUIT_COLUMNS, []);
  const selected = rows?.find((row) => row.id === selectedId) ?? null;
  const onlyToday =
    from.trim() === todayText && to.trim() === todayText && forWhom === EVERYONE && item === null;
  const period = (nextFrom: string, nextTo: string) => {
    setFrom(nextFrom);
    setTo(nextTo);
  };

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
            period(todayText, todayText);
            setForWhom(EVERYONE);
            setItem(null);
          }}
        >
          <Localized id="fruit-list-clear" />
        </Button>
      }
    />
  );

  return (
    <Card compact title={<Localized id="fruit-list-title" />}>
      <div {...formNavigationProps} className="grid grid-cols-2 items-end gap-4 normal:grid-cols-4">
        <Field label={<Localized id="fruit-filter-from" />}>
          {(control) => <DateInput {...control} value={from} onChange={setFrom} />}
        </Field>
        <Field label={<Localized id="fruit-filter-to" />}>
          {(control) => <DateInput {...control} value={to} onChange={setTo} />}
        </Field>
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
        <Field label={<Localized id="fruit-filter-item" />}>
          {(control) => (
            <Picker
              {...control}
              scope="item"
              value={item}
              placeholder={l10n.getString('fruit-filter-item-all')}
              onChange={(hit) => {
                setItem(
                  hit === null
                    ? null
                    : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
                );
              }}
            />
          )}
        </Field>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Button
          onClick={() => {
            period(todayText, todayText);
          }}
        >
          <Localized id="fruit-period-today" />
        </Button>
        <Button
          onClick={() => {
            period(monthStart(todayText), todayText);
          }}
        >
          <Localized id="fruit-period-month" />
        </Button>
        <Button
          onClick={() => {
            period(yearStart(todayText), todayText);
          }}
        >
          <Localized id="fruit-period-year" />
        </Button>
        <Button
          onClick={() => {
            period('', '');
          }}
        >
          <Localized id="fruit-period-all" />
        </Button>
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
        <Localized id="fruit-list-select-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      <PagedList<FruitRow>
        columns={columns}
        rows={rows}
        totalCount={first?.total_count}
        getRowId={(row) => String(row.id)}
        labelId="fruit-list-label"
        failedId="fruit-list-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        hasMore={list.hasNextPage}
        loadingMore={list.isFetchingNextPage}
        onLoadMore={() => {
          void list.fetchNextPage();
        }}
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
