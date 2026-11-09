// The saved جنرل entries under the form (plan 44, PRD FR-7.70, FR-19.11): by default today's, with
// the days as a filter, so an older entry can be found and fixed. The row that has focus is the one
// the Edit and Cancel buttons work on; Enter on a row opens it. A cancelled entry stays in the list,
// marked in words. Four states: loading, an error with a retry, nothing to list, and the rows. The
// total under the amount column is the backend's.

import { Localized } from '@fluent/react';
import { BookOpen } from 'lucide-react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { useDebouncedValue, usePeriod } from '../../../shared/hooks';
import { formNavigationProps } from '../../../shared/keyboard';
import { Card, EmptyState, FilterBar, PeriodBox, ShowClosedTick } from '../../../shared/ui';
import { useVouchersList } from '../../vouchers';
import { GENERAL_LIST_COLUMNS } from '../list-columns';
import type { VoucherRow } from '../types';
import { useOpenVoucher } from '../../../shared/edit';
import { VoucherRowActions } from './VoucherRowActions';

/** A pause so a burst of key presses in the search box asks once. */
const SEARCH_DELAY_MS = 150;

export function GeneralListSection({ today }: { today: string }) {
  const open = useOpenVoucher();
  const period = usePeriod('cashbook.general', 'today', today);
  const [search, setSearch] = useState('');
  const [showFinished, setShowFinished] = useState(false);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const searchText = useDebouncedValue(search.trim(), SEARCH_DELAY_MS);
  const list = useVouchersList(
    {
      voucher_type: 'general',
      from: period.range.from,
      to: period.range.to,
      account_id: null,
      show_finished: showFinished,
      search: searchText === '' ? null : searchText,
      group_by_month: period.state.kind !== 'today',
    },
    period.ready,
  );
  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const selected = rows?.find((row) => row.id === selectedId) ?? null;
  const months = first === undefined || first.months.length === 0 ? undefined : first.months;
  return (
    <Card compact title={<Localized id="cashbook-general-list-title" />}>
      <div {...formNavigationProps}>
        <FilterBar
          screen="cashbook.general.list"
          search={{ value: search, onChange: setSearch }}
          extra={<ShowClosedTick checked={showFinished} onChange={setShowFinished} />}
        >
          {period.ready ? <PeriodBox period={period} /> : null}
        </FilterBar>
      </div>
      <VoucherRowActions row={selected} />
      <PagedList<VoucherRow>
        columns={GENERAL_LIST_COLUMNS}
        rows={rows}
        getRowId={(row) => String(row.id)}
        labelId="cashbook-general-list-label"
        failedId="cashbook-general-list-failed"
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
        empty={
          <EmptyState
            icon={BookOpen}
            title={<Localized id="cashbook-general-list-empty" />}
            help={<Localized id="cashbook-general-list-empty-help" />}
          />
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
        totals={
          first === undefined
            ? undefined
            : (context) => ({
                labelField: 'narration',
                values: {
                  narration: context.text('glossary-meezan'),
                  ...(first.totals.total_paisa === null
                    ? {}
                    : { amount: context.money(first.totals.total_paisa) }),
                },
              })
        }
      />
    </Card>
  );
}
