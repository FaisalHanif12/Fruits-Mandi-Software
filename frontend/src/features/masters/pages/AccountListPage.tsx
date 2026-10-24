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
import { ListGrid, type ListContext } from '../../../shared/grid';
import { useFocusFirstField, useLayoutStep, useRememberedFlag } from '../../../shared/hooks';
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
import type { AccountFilter, AccountRow, AccountType } from '../../../shared/accounts';
import { ACCOUNTS_SCREEN, accountChoicesFrom, accountChoicesText } from '../remember';
import { useMasterFocus } from '../store';

/** A list of this many rows or fewer is as tall as its rows; a longer one fills the screen. */
const SHORT_LIST_ROWS = 30;
const ALL = 'all';
const TYPES: readonly AccountType[] = ['seller', 'buyer', 'money_lent', 'other'];

/** The account list starts as the Owner left it: the kind and the inactive tick (PRD FR-7.45). */
export function AccountListPage() {
  return (
    <Remembered screen={ACCOUNTS_SCREEN}>{(memory) => <AccountListBody {...memory} />}</Remembered>
  );
}

function AccountListBody({ saved, remember: keepChoices }: MemoryProps) {
  const navigate = useNavigate();
  const start = useMemo(() => accountChoicesFrom(saved), [saved]);
  const [type, setType] = useState<AccountType | typeof ALL>(start.type);
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
      account_type: type === ALL ? null : type,
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
  const filtering = filter.text !== '' || filter.account_type !== null || filter.include_inactive;
  const data = list.data;
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
        ...(data === undefined
          ? {}
          : {
              opening_dr: context.money(data.total_dr_paisa),
              opening_cr: context.money(data.total_cr_paisa),
            }),
      },
    }),
    [data],
  );

  let body;
  if (data === undefined && list.isPending) {
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
  } else if (data?.rows.length === 0) {
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
  } else if (data !== undefined) {
    body = (
      <div className="flex min-h-0 flex-1 flex-col gap-3">
        <div className={data.rows.length <= SHORT_LIST_ROWS ? '' : 'min-h-0 flex-1'}>
          <ListGrid<AccountRow>
            columns={columns}
            rows={data.rows}
            getRowId={(row) => String(row.id)}
            labelId="accounts-title"
            onOpen={open}
            heightMode={data.rows.length <= SHORT_LIST_ROWS ? 'auto' : 'fill'}
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
              <Num paisa={data.total_dr_paisa} />
            </span>
            <span className="flex items-baseline gap-2">
              <Localized id="accounts-total-cr" />
              <Num paisa={data.total_cr_paisa} />
            </span>
          </div>
        )}
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
        {data === undefined || data.rows.length === 0 ? null : (
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
                  ...TYPES.map((option) => ({
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
