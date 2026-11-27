// The saved entries under a form of the روکڑ area (PRD FR-19.11, FR-7.67, FR-7.70, plan 100 F): دکان خرچہ,
// ادھار رقم, الگ کھاتہ and جنرل. Like the وصولی, ادائیگی and پیشگی lists, it shows 30 days at a time, the
// latest first, one page at a time, with the buttons and the box that go to a day. With no person or khata
// chosen on the form it lists every entry; with one chosen, only his. Right after a save the new entry is
// there, because a save asks every list to read again. The row that has focus is the one the Edit and
// Cancel buttons work on; Enter on a row opens it while its day is open. A cancelled entry is left out
// until the tick is on. Four states: loading, an error with a retry, nothing in these days, and the rows.
// The total under the amount column is the backend's.

import { Localized } from '@fluent/react';
import { FileText, ReceiptText, SearchX } from 'lucide-react';
import type { SortInput } from '../../../bindings/SortInput';
import { useState } from 'react';
import type { VoucherType } from '../../../bindings/VoucherType';
import type { WindowSource } from '../../../bindings/WindowSource';
import { uiErrorOf } from '../../../shared/api';
import { useOpenVoucher } from '../../../shared/edit';
import { PagedList, type ListColumn } from '../../../shared/grid';
import { useDebouncedValue, useListWindow } from '../../../shared/hooks';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  Button,
  Card,
  EmptyState,
  FilterBar,
  ListWindowBar,
  ShowClosedTick,
} from '../../../shared/ui';
import { useVouchersList } from '../../vouchers';
import type { VoucherRow } from '../types';
import { VoucherRowActions } from './VoucherRowActions';

/** A pause so a burst of key presses in the search box asks once. */
const SEARCH_DELAY_MS = 150;

/** The newest entry on top (plan 100 F), until the user sorts by a column. */
const NEWEST_FIRST: SortInput = { column: 'entry_date', direction: 'desc', language: null };

interface EntryListSectionProps {
  /** Names the list for its remembered page size and filters. */
  screen: string;
  /** The Fluent id of the card's title. */
  titleId: string;
  /** The kinds the list holds. */
  types: readonly [VoucherType, ...VoucherType[]];
  /** The person, khata or account chosen on the form, or null for every entry. */
  accountId: number | null;
  /** What the list is, for the first day it has data. */
  source: WindowSource;
  columns: readonly ListColumn<VoucherRow>[];
  /** Today, `YYYY-MM-DD`; the list waits until it is known. */
  today: string | undefined;
  /** The column the totals row's word stands in. Default: the account. */
  totalsField?: 'account' | 'narration';
  /** The words of a list with its own: its name, when it cannot be read, and when it has nothing. */
  words?: { labelId: string; failedId: string; emptyId: string; emptyHelpId: string } | undefined;
}

export function EntryListSection({
  screen,
  titleId,
  types,
  accountId,
  source,
  columns,
  today,
  totalsField = 'account',
  words,
}: EntryListSectionProps) {
  const open = useOpenVoucher();
  const listWindow = useListWindow({ kind: 'days', source, today });
  const [search, setSearch] = useState('');
  const [showFinished, setShowFinished] = useState(false);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const searchText = useDebouncedValue(search.trim(), SEARCH_DELAY_MS);
  const [first, ...others] = types;
  const list = useVouchersList(
    {
      voucher_type: others.length === 0 ? first : null,
      ...(others.length === 0 ? {} : { voucher_types: [...types] }),
      from: listWindow.range.from,
      to: listWindow.range.to,
      account_id: accountId,
      show_finished: showFinished,
      search: searchText === '' ? null : searchText,
      group_by_month: false,
    },
    listWindow.ready,
    NEWEST_FIRST,
  );
  const page = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((one) => one.rows);
  const selected = rows?.find((row) => row.id === selectedId) ?? null;
  const filtering = searchText !== '' || showFinished || !listWindow.isLatest;

  return (
    <Card compact icon={ReceiptText} title={<Localized id={titleId} />}>
      <div {...formNavigationProps}>
        <FilterBar
          screen={screen}
          search={{ value: search, onChange: setSearch }}
          extra={<ShowClosedTick checked={showFinished} onChange={setShowFinished} />}
        />
      </div>
      <VoucherRowActions row={selected} />
      <PagedList<VoucherRow>
        columns={columns}
        rows={rows}
        getRowId={(row) => String(row.id)}
        labelId={words?.labelId ?? titleId}
        failedId={words?.failedId ?? 'cashbook-entries-failed'}
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        onPageKey={listWindow.onPageKey}
        sort={{ value: list.sort, onChange: list.setSort }}
        empty={
          filtering ? (
            <EmptyState
              icon={SearchX}
              title={<Localized id="cashbook-entries-no-match" />}
              help={<Localized id="cashbook-entries-no-match-help" />}
              action={
                <Button
                  onClick={() => {
                    listWindow.latest();
                    setSearch('');
                    setShowFinished(false);
                  }}
                >
                  <Localized id="vouchers-list-clear" />
                </Button>
              }
            />
          ) : (
            <EmptyState
              icon={FileText}
              title={<Localized id={words?.emptyId ?? 'cashbook-entries-empty'} />}
              help={<Localized id={words?.emptyHelpId ?? 'cashbook-entries-empty-help'} />}
            />
          )
        }
        onOpen={(row) => {
          if (row.can_edit && !row.needs_reopen) open(row);
          else setSelectedId(row.id);
        }}
        onFocusRow={(rowId) => {
          setSelectedId(rows?.find((row) => String(row.id) === rowId)?.id ?? null);
        }}
        focusRowId={selectedId === null ? null : String(selectedId)}
        isInactive={(row) => row.status === 'cancelled'}
        // A list of two kinds (lent and returned, paid out and received) has no one total: a sum of both would
        // mean nothing, so only a list of one kind shows the backend's total.
        totals={
          page === undefined || others.length > 0
            ? undefined
            : (context) => ({
                labelField: totalsField,
                values: {
                  [totalsField]: context.text('glossary-meezan'),
                  ...(page.totals.total_paisa === null
                    ? {}
                    : { amount: context.money(page.totals.total_paisa) }),
                },
              })
        }
      />
      <ListWindowBar window={listWindow} />
    </Card>
  );
}
