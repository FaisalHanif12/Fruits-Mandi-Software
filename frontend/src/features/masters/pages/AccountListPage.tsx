// The account list (route /index/accounts; PRD FR-10.5, FR-7.17): code, names, city, opening
// date, opening debit and credit, credit allowed and the current balance, with the total debit and
// credit under the list. The filter searches both scripts (the backend search); a tick shows the
// inactive ones. Keys: F2 new, ↑ ↓ and Page Up / Page Down between rows, Enter opens, Esc goes back.
// Four states: loading, error with a retry, empty with a button, rows.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Landmark, Plus, SearchX } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, PagedFooter, type ListContext } from '../../../shared/grid';
import {
  useFocusFirstField,
  useLayoutStep,
  useLoadRest,
  useRememberedFlag,
} from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps, useEscape, useShortcut } from '../../../shared/keyboard';
import { Remembered, type MemoryProps } from '../../../shared/screen-state';
import {
  Button,
  Checkbox,
  EmptyState,
  ErrorState,
  Field,
  LoadingState,
  Num,
  Select,
  TextInput,
} from '../../../shared/ui';
import { accountColumns, hasMoreColumns, showsOpeningColumns } from '../account-columns';
import { useAccountList } from '../../../shared/accounts';
import type { AccountFilter, AccountListKind, AccountRow } from '../../../shared/accounts';
import { ACCOUNTS_SCREEN, accountChoicesFrom, accountChoicesText } from '../remember';
import { useMasterFocus } from '../store';

/** A list of this many rows or fewer is as tall as its rows; a longer one fills the screen. */
const SHORT_LIST_ROWS = 30;
/**
 * How many pages (100 accounts each) are read to find the row last opened. It may be on no page of
 * this list (inactive, or not in this filter), and the list can have 20,000 accounts, so the
 * search stops here and the focus starts on the first row.
 */
const REMEMBERED_ROW_PAGES = 10;
const ALL = 'all';
/** The kinds the list can show besides "all": sellers, buyers and the پختہ کھاتہ accounts (plan 32, R6). */
const KINDS: readonly AccountListKind[] = ['seller', 'buyer', 'pukhta'];

/** The account list starts as the Owner left it: the kind and the inactive tick (PRD FR-7.45). */
export function AccountListPage() {
  return (
    <Remembered screen={ACCOUNTS_SCREEN}>{(memory) => <AccountListBody {...memory} />}</Remembered>
  );
}

function AccountListBody({ saved, remember: keepChoices }: MemoryProps) {
  const navigate = useNavigate();
  const start = useMemo(() => accountChoicesFrom(saved), [saved]);
  const [type, setType] = useState<AccountListKind>(start.type);
  const [text, setText] = useState('');
  const [showInactive, setShowInactive] = useState(start.showInactive);
  useEffect(() => {
    keepChoices(accountChoicesText({ type, showInactive }));
  }, [type, showInactive, keepChoices]);
  // The tick that adds the opening debit and credit below the wide step. It is kept for next time.
  const [moreColumns, setMoreColumns] = useRememberedFlag('accounts.more_columns', false);
  const step = useLayoutStep();
  const columns = useMemo(() => accountColumns(step, moreColumns), [step, moreColumns]);
  const filter = useMemo<AccountFilter>(
    () => ({
      kind: type,
      text: text.trim(),
      include_inactive: showInactive,
    }),
    [type, text, showInactive],
  );
  const list = useAccountList(filter);
  const containerRef = useFocusFirstField<HTMLDivElement>();
  const remember = useMasterFocus((state) => state.remember);
  const focusRowId = useMasterFocus((state) => state.rows.account);

  const openNew = () => {
    void navigate({ to: '/index/accounts/new' });
  };
  const open = (row: AccountRow) => {
    remember('account', row.id);
    void navigate({ to: '/index/accounts/$id', params: { id: String(row.id) } });
  };
  useShortcut(SHORTCUTS.newEntry, openNew);
  useEscape('leave', () => {
    void navigate({ to: '/index' });
  });

  const newButton = (
    <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={openNew}>
      <Plus aria-hidden="true" className="size-icon-md" />
      <Localized id="accounts-new" />
    </Button>
  );
  const filtering = filter.text !== '' || filter.kind !== ALL || filter.include_inactive;
  // Every page loaded so far. The first page carries the count and the opening totals of the whole
  // filter, which are the ones shown (PRD FR-19.11).
  const first = list.data?.pages[0];
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  // The grid puts the focus back on the account last opened or saved when it is drawn, so that row
  // must be loaded first: while it is on a later page, the pages are read until it is (no filter).
  const findingRow =
    focusRowId !== undefined &&
    !filtering &&
    list.hasNextPage &&
    (list.data?.pages.length ?? 0) < REMEMBERED_ROW_PAGES &&
    !list.data?.pages.some((page) => page.rows.some((row) => row.id === focusRowId));
  useLoadRest(list, findingRow);
  // The opening debit and credit columns come with the wide layout, or with "More columns"; the
  // totals stand under them there (Code Standards F19), and as a line under the list where the
  // columns are not shown.
  const totalsUnderColumns = showsOpeningColumns(step, moreColumns);
  // The callback depends on the loaded list, not on the two amounts: a Paisa value must not go
  // into a dependency list (it is typed unknown there, Code Standards F3).
  const totals = useCallback(
    (context: ListContext) => ({
      labelField: 'name',
      values: {
        name: context.text('glossary-meezan'),
        ...(first === undefined
          ? {}
          : {
              opening_dr: context.money(first.total_dr_paisa),
              opening_cr: context.money(first.total_cr_paisa),
            }),
      },
    }),
    [first],
  );

  let body;
  if ((rows === undefined && list.isPending) || findingRow) {
    body = <LoadingState />;
  } else if (list.isError) {
    body = (
      <ErrorState
        title={<Localized id="accounts-load-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (rows?.length === 0) {
    body = filtering ? (
      <EmptyState
        icon={SearchX}
        title={<Localized id="accounts-no-match-title" />}
        help={<Localized id="accounts-no-match-help" />}
        action={
          <Button
            onClick={() => {
              setText('');
              setType(ALL);
              setShowInactive(true);
            }}
          >
            <Localized id="accounts-no-match-clear" />
          </Button>
        }
      />
    ) : (
      <EmptyState
        icon={Landmark}
        title={<Localized id="accounts-empty" />}
        help={<Localized id="accounts-empty-help" vars={{ key: SHORTCUTS.newEntry.label }} />}
        action={newButton}
      />
    );
  } else if (rows !== undefined && first !== undefined) {
    body = (
      <div className="flex min-h-0 flex-1 flex-col gap-3">
        <div className={rows.length <= SHORT_LIST_ROWS ? '' : 'min-h-0 flex-1'}>
          <ListGrid<AccountRow>
            columns={columns}
            rows={rows}
            getRowId={(row) => String(row.id)}
            labelId="accounts-title"
            onOpen={open}
            heightMode={rows.length <= SHORT_LIST_ROWS ? 'auto' : 'fill'}
            isInactive={(row) => !row.is_active}
            totals={totalsUnderColumns ? totals : undefined}
            focusRowId={focusRowId === undefined ? null : String(focusRowId)}
            onFocusRow={(rowId) => {
              remember('account', Number(rowId));
            }}
          />
        </div>
        {totalsUnderColumns ? null : (
          <div className="flex flex-wrap gap-x-6 gap-y-1 type-body text-ink">
            <span className="flex items-baseline gap-2">
              <Localized id="accounts-total-dr" />
              <Num paisa={first.total_dr_paisa} />
            </span>
            <span className="flex items-baseline gap-2">
              <Localized id="accounts-total-cr" />
              <Num paisa={first.total_cr_paisa} />
            </span>
          </div>
        )}
        <PagedFooter
          shown={rows.length}
          total={first.count}
          hasMore={list.hasNextPage}
          loadingMore={list.isFetchingNextPage}
          onLoadMore={() => {
            void list.fetchNextPage();
          }}
        />
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="accounts-title">
            <h1 className="type-title" />
          </Localized>
          <Localized id="accounts-help">
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        {rows === undefined || rows.length === 0 ? null : (
          <div className="flex flex-wrap gap-2">
            <Button
              onClick={() => {
                void navigate({ to: '/index/openings' });
              }}
            >
              <Localized id="openings-open" />
            </Button>
            {newButton}
          </div>
        )}
      </div>
      <div ref={containerRef} {...formNavigationProps} className="flex flex-wrap items-end gap-4">
        <div className="w-field">
          <Field label={<Localized id="accounts-filter-type" />}>
            {(props) => (
              <Select
                {...props}
                value={type}
                onChange={setType}
                options={[
                  { value: ALL, label: <Localized id="accounts-type-all" /> },
                  ...KINDS.map((option) => ({
                    value: option,
                    label: <Localized id={`accounts-type-${option}`} />,
                  })),
                ]}
              />
            )}
          </Field>
        </div>
        <div className="min-w-0 flex-1 basis-field">
          <Field label={<Localized id="accounts-filter-label" />}>
            {(props) => <TextInput {...props} value={text} onChange={setText} />}
          </Field>
        </div>
        <Checkbox
          checked={showInactive}
          onChange={setShowInactive}
          label={<Localized id="accounts-show-inactive" />}
        />
        {hasMoreColumns(step) ? (
          <Checkbox
            checked={moreColumns}
            onChange={setMoreColumns}
            label={<Localized id="accounts-more-columns" />}
          />
        ) : null}
      </div>
      <div className="flex min-h-0 flex-1 flex-col">{body}</div>
    </div>
  );
}
