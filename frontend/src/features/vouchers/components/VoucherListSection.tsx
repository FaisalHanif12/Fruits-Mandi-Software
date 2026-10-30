// The vouchers of the type on the form, in a list under it (PRD 15.1, FR-15.3): by default today's,
// with the day range and the account as filters. ↑ ↓ and Page Up and Page Down move between rows,
// Enter (or a double click) opens the row to edit it while its day is open, and the buttons above
// the list edit, cancel or print the row that has focus. A cancelled voucher stays in the list,
// marked in words. Four states: loading, an error with a retry, nothing to list, and the rows.
// The amounts are the stored sums and the total under them is the backend's figure.

import { Localized } from '@fluent/react';
import { FileText, Pencil, Printer, SearchX, Ban } from 'lucide-react';
import { useMemo, useState } from 'react';
import type { VouchersListInput } from '../../../bindings/VouchersListInput';
import { pickedOf } from '../../../shared/accounts';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { useDebouncedValue, usePeriod } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps } from '../../../shared/keyboard';
import {
  AccountPicker,
  Button,
  Card,
  EmptyState,
  Field,
  FilterBar,
  FormSpan,
  PeriodBox,
  ShowClosedTick,
  type PickerValue,
} from '../../../shared/ui';
import { VOUCHER_COLUMNS } from '../list-columns';
import { useVouchersList } from '../hooks';
import type { ScreenType } from '../screen-type';
import type { VoucherRow } from '../types';

/** A pause so a burst of key presses in the search box asks once. */
const SEARCH_DELAY_MS = 150;

interface VoucherListSectionProps {
  type: ScreenType;
  /** Today, `YYYY-MM-DD`: the list starts with today's vouchers. */
  today: string;
  /** The voucher that has focus in the list, or null. */
  selectedId: number | null;
  onSelect: (row: VoucherRow | null) => void;
  onEdit: (row: VoucherRow) => void;
  onCancel: (row: VoucherRow) => void;
  onPrint: (row: VoucherRow) => void;
}

export function VoucherListSection({
  type,
  today,
  selectedId,
  onSelect,
  onEdit,
  onCancel,
  onPrint,
}: VoucherListSectionProps) {
  // The list opens on today, and the Owner's last choice of period is kept for this kind of voucher
  // (plan 38 R4). Rows of finished (cancelled) vouchers are left out until the tick is on.
  const period = usePeriod(`vouchers.${type}`, 'today', today);
  const [account, setAccount] = useState<PickerValue | null>(null);
  const [search, setSearch] = useState('');
  const [showFinished, setShowFinished] = useState(false);
  const searchText = useDebouncedValue(search.trim(), SEARCH_DELAY_MS);
  const input: Omit<VouchersListInput, 'offset' | 'limit' | 'sort'> = {
    voucher_type: type,
    from: period.range.from,
    to: period.range.to,
    account_id: account?.id ?? null,
    person_key: null,
    show_finished: showFinished,
    search: searchText === '' ? null : searchText,
    // A month heading is worth its row only when the list covers more than one day.
    group_by_month: period.state.kind !== 'today',
  };
  const list = useVouchersList(input, period.ready);
  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const columns = useMemo(() => VOUCHER_COLUMNS, []);
  const selected = rows?.find((row) => row.id === selectedId) ?? null;
  const onlyToday =
    period.state.kind === 'today' && account === null && searchText === '' && !showFinished;
  const months = first === undefined || first.months.length === 0 ? undefined : first.months;

  const empty = onlyToday ? (
    <EmptyState
      icon={FileText}
      title={<Localized id={`vouchers-list-empty-${type}`} />}
      help={<Localized id="vouchers-list-empty-help" vars={{ key: SHORTCUTS.save.label }} />}
    />
  ) : (
    <EmptyState
      icon={SearchX}
      title={<Localized id="vouchers-list-no-match" />}
      help={<Localized id="vouchers-list-no-match-help" />}
      action={
        <Button
          onClick={() => {
            period.change({ kind: 'today', from: '', to: '' });
            setAccount(null);
            setSearch('');
            setShowFinished(false);
          }}
        >
          <Localized id="vouchers-list-clear" />
        </Button>
      }
    />
  );
  // The total under the amount column is the backend's figure for the saved vouchers of the whole
  // filter; a cancelled voucher is in the rows but not in the total. The backend sends no total
  // (null) for a list of all types, which this screen never asks for; then only the word shows.
  const body = (
    <PagedList<VoucherRow>
      columns={columns}
      rows={rows}
      getRowId={(row) => String(row.id)}
      labelId="vouchers-list-label"
      failedId="vouchers-list-failed"
      error={list.isError ? uiErrorOf(list.error) : null}
      onRetry={() => {
        void list.refetch();
      }}
      pager={list.pager}
      sort={{ value: list.sort, onChange: list.setSort }}
      months={
        months === undefined ? undefined : { months, monthOf: (row) => row.entry_date.slice(0, 7) }
      }
      empty={empty}
      onOpen={(row) => {
        if (row.editable) onEdit(row);
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
              labelField: 'account',
              values: {
                account: context.text('glossary-meezan'),
                ...(first.totals.total_paisa === null
                  ? {}
                  : { amount: context.money(first.totals.total_paisa) }),
              },
            })
      }
    />
  );

  return (
    <Card compact title={<Localized id={`vouchers-list-title-${type}`} />}>
      <div {...formNavigationProps}>
        <FilterBar
          screen={`vouchers.list.${type}`}
          search={{ value: search, onChange: setSearch }}
          moreSet={account === null ? 0 : 1}
          more={
            <FormSpan span="two">
              <Field label={<Localized id="vouchers-list-account" />}>
                {(control) => (
                  <AccountPicker
                    {...control}
                    canAdd={false}
                    scope="any"
                    showCard={false}
                    value={account}
                    onChange={(hit) => {
                      setAccount(hit === null ? null : pickedOf(hit));
                    }}
                  />
                )}
              </Field>
            </FormSpan>
          }
          extra={<ShowClosedTick checked={showFinished} onChange={setShowFinished} />}
        >
          {period.ready ? <PeriodBox period={period} /> : null}
        </FilterBar>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Button
          disabled={!selected?.editable}
          onClick={() => {
            if (selected !== null) onEdit(selected);
          }}
        >
          <Pencil aria-hidden="true" className="size-icon-md" />
          <Localized id="vouchers-edit" />
        </Button>
        <Button
          variant="danger"
          disabled={!selected?.editable}
          onClick={() => {
            if (selected !== null) onCancel(selected);
          }}
        >
          <Ban aria-hidden="true" className="size-icon-md" />
          <Localized id="vouchers-cancel" />
        </Button>
        <Button
          shortcut={SHORTCUTS.print}
          disabled={selected === null}
          onClick={() => {
            if (selected !== null) onPrint(selected);
          }}
        >
          <Printer aria-hidden="true" className="size-icon-md" />
          <Localized id="vouchers-print" />
        </Button>
        <Localized id="vouchers-list-select-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      {body}
    </Card>
  );
}
