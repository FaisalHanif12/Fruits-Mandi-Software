// The opening balances grid (route /index/openings; plan 07, PRD FR-10.9): one row per account,
// with the opening date, debit and credit typed in the cells, so many openings can be entered at
// once. Only the rows the user changed go to the backend, as the text typed. The backend checks the
// lines and works out the totals and what ابتدائی فرق would be (the same function as the save,
// Code Standards B5); the screen only shows them. The save is all or nothing.
//
// - What is typed lives in the entry store, never only in the grid, so a language switch rebuilds
//   the grid and loses nothing (Code Standards F39).
// - Enter stops on the date, the debit and the credit of a row and goes on to the next row; the
//   rows are fixed, no key adds one. Keys: F10 or Ctrl+S saves, Esc goes back (asking first).
// - Four states: loading, error with a retry, empty with a button, and the grid.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Landmark, Plus } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { LeaveGuard } from '../../../app/LeaveGuard';
import {
  FALLBACK_ERROR_MESSAGE_ID,
  errorMessageId,
  messageVars,
  uiErrorOf,
} from '../../../shared/api';
import type { UiError } from '../../../bindings/UiError';
import type { UiErrorItem } from '../../../bindings/UiErrorItem';
import { isoToDisplayDate, useGrouping } from '../../../shared/format';
import { EntryGrid, createEntryStore, type EntryColumn, type EntryRow } from '../../../shared/grid';
import { useDebouncedValue, useSingleFlight } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape, useShortcut } from '../../../shared/keyboard';
import {
  Balance,
  Button,
  DateInput,
  EmptyState,
  ErrorNotice,
  ErrorState,
  Field,
  LoadingState,
  Num,
  showToast,
} from '../../../shared/ui';
import {
  useAccountList,
  useOpeningsPreview,
  useSaveOpenings,
  type AccountFilter,
  type AccountRow,
  type OpeningsSaved,
} from '../../../shared/accounts';
import {
  CR_FIELD,
  DATE_FIELD,
  DR_FIELD,
  baselineOf,
  changedLines,
  openingTyped,
  problemsOfError,
  problemsOfPreview,
  type ChangedLine,
  type LineProblem,
  type OpeningBaseline,
} from '../openings';

/** The preview waits this long after the last key press, and the one before it is dropped (F35). */
const PREVIEW_DELAY_MS = 50;
/** A grid of this many rows or fewer is as tall as its rows; a longer one fills the screen. */
const SHORT_GRID_ROWS = 30;
/** Every active account, the same list the account screen opens with. */
const ALL_ACTIVE: AccountFilter = { account_type: null, text: '', include_inactive: false };

const BASE_COLUMNS: readonly EntryColumn[] = [
  { field: 'code', type: 'calc', headerId: 'accounts-col-code' },
  { field: 'name', type: 'label', headerId: 'accounts-col-name' },
  { field: DATE_FIELD, type: 'date', headerId: 'accounts-col-opening-date' },
  { field: DR_FIELD, type: 'money', headerId: 'accounts-col-opening-dr' },
  { field: CR_FIELD, type: 'money', headerId: 'accounts-col-opening-cr' },
];

/** The cells a user types in. A row of an account that cannot take an opening locks all three. */
const TYPED_FIELDS: readonly string[] = [DATE_FIELD, DR_FIELD, CR_FIELD];

const FIELD_LABEL: Record<string, string> = {
  [DATE_FIELD]: 'accounts-col-opening-date',
  [DR_FIELD]: 'accounts-col-opening-dr',
  [CR_FIELD]: 'accounts-col-opening-cr',
};

interface EditorProps {
  accounts: readonly AccountRow[];
  /** Today as `YYYY-MM-DD`. */
  today: string;
}

/** What the grid starts from: the store, what each account looked like, and which row is which. */
function startOf(
  accounts: readonly AccountRow[],
  grouping: ReturnType<typeof useGrouping>,
  today: string,
) {
  const common = isoToDisplayDate(today);
  const typedRows = accounts.map((account) => openingTyped(account, grouping, common));
  const store = createEntryStore(typedRows);
  const baselines = new Map<string, OpeningBaseline>();
  accounts.forEach((account, index) => {
    baselines.set(String(account.id), baselineOf(account, typedRows[index] ?? {}));
  });
  const accountOfRow = new Map<string, AccountRow>();
  const rowOfAccount = new Map<string, string>();
  store.getState().rows.forEach((row, index) => {
    const account = accounts[index];
    if (account === undefined) return;
    accountOfRow.set(row.rowId, account);
    rowOfAccount.set(String(account.id), row.rowId);
  });
  return { common, store, baselines, accountOfRow, rowOfAccount };
}

function OpeningsEditor({ accounts, today }: EditorProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const navigate = useNavigate();
  const [start] = useState(() => startOf(accounts, grouping, today));
  const { store, accountOfRow, rowOfAccount } = start;
  const [baselines, setBaselines] = useState(start.baselines);
  const [commonDate, setCommonDate] = useState(start.common);
  const rows = store((state) => state.rows);
  const save = useSaveOpenings();
  // The problems a refused save pointed at, kept only while the lines are the ones that were sent.
  const [refused, setRefused] = useState<{ error: UiError; sent: readonly ChangedLine[] } | null>(
    null,
  );

  const changed = useMemo(() => changedLines(rows, baselines), [rows, baselines]);
  const settled = useDebouncedValue(changed, PREVIEW_DELAY_MS);
  const settledLines = useMemo(() => settled.map((entry) => entry.line), [settled]);
  const preview = useOpeningsPreview(settledLines);

  const refusal = useMemo(
    () =>
      refused !== null && refused.sent === changed ? problemsOfError(refused.error, changed) : null,
    [refused, changed],
  );
  const problems = useMemo<LineProblem[]>(() => {
    if (refusal !== null) return refusal.problems;
    if (preview.data === undefined || preview.isPlaceholderData) return [];
    return problemsOfPreview(settled, preview.data);
  }, [refusal, preview.data, preview.isPlaceholderData, settled]);

  const messageOf = useCallback(
    (item: UiErrorItem): string => {
      const id = errorMessageId(item.code);
      return l10n.getBundle(id) === null
        ? l10n.getString(FALLBACK_ERROR_MESSAGE_ID)
        : l10n.getString(id, messageVars(item.values, grouping));
    },
    [l10n, grouping],
  );
  const cellErrors = useMemo(() => {
    const byRow: Record<string, Record<string, string>> = {};
    for (const problem of problems) {
      const row = (byRow[problem.rowId] ??= {});
      row[problem.field] ??= messageOf(problem.item);
    }
    return byRow;
  }, [problems, messageOf]);

  // The rows of accounts that cannot take an opening balance (PRD FR-10.9): their cells are locked.
  const lockedRows = useMemo(() => {
    const locked = new Set<string>();
    for (const [rowId, account] of accountOfRow) {
      if (!account.can_take_opening) locked.add(rowId);
    }
    return locked;
  }, [accountOfRow]);
  const columns = useMemo<readonly EntryColumn[]>(
    () =>
      BASE_COLUMNS.map((column) =>
        TYPED_FIELDS.includes(column.field)
          ? { ...column, editable: (row: EntryRow) => !lockedRows.has(row.rowId) }
          : column,
      ),
    [lockedRows],
  );

  // The cells the user never types in: the code and the name.
  const calc = useMemo(
    () =>
      Object.fromEntries(
        Array.from(accountOfRow, ([rowId, account]) => [
          rowId,
          {
            code: account.code,
            // A seller whose opening counts as an advance says so beside his name, so nothing the
            // line carries is hidden (NFR-USE-27).
            name:
              account.account_type === 'seller' && account.opening_is_advance
                ? l10n.getString('openings-name-advance', {
                    name: displayName(account, language),
                  })
                : displayName(account, language),
          },
        ]),
      ),
    [accountOfRow, language, l10n],
  );

  const setCommon = (next: string) => {
    const previous = commonDate;
    setCommonDate(next);
    // An account with no opening follows the common date until its own date is typed.
    store.getState().updateTypedWhere(
      DATE_FIELD,
      (row) => {
        const base = baselines.get(row.typed['account_id'] ?? '');
        return (
          base !== undefined &&
          base.canTake &&
          !base.hasOpening &&
          row.typed[DATE_FIELD] === previous
        );
      },
      next,
    );
  };

  const applySaved = (saved: OpeningsSaved) => {
    const next = new Map(baselines);
    for (const account of saved.rows) {
      const typed = openingTyped(account, grouping, commonDate);
      next.set(String(account.id), baselineOf(account, typed));
      const rowId = rowOfAccount.get(String(account.id));
      if (rowId === undefined) continue;
      for (const field of [DATE_FIELD, DR_FIELD, CR_FIELD]) {
        store.getState().updateTyped(rowId, field, typed[field] ?? '');
      }
    }
    setBaselines(next);
  };

  // One save at a time: a second F10 while one is on its way does nothing (NFR-REL-1).
  const flight = useSingleFlight();
  const submitOnce = (): Promise<boolean> => {
    if (changed.length === 0) return Promise.resolve(true);
    const sent = changed;
    return save
      .mutateAsync(sent.map((entry) => entry.line))
      .then((saved) => {
        setRefused(null);
        applySaved(saved);
        showToast('success', l10n.getString('openings-saved'));
        return true;
      })
      .catch((error: unknown) => {
        setRefused({ error: uiErrorOf(error), sent });
        return false;
      });
  };
  const submit = (): Promise<boolean> => flight.run(submitOnce, false);
  const submitNow = () => {
    void submit();
  };
  useShortcut(SHORTCUTS.save, submitNow);
  useEscape('leave', () => {
    void navigate({ to: '/index/accounts' });
  });

  const notice = refusal?.rest ?? null;
  const total = preview.data;

  return (
    <div className="flex h-full flex-col gap-4">
      <LeaveGuard dirty={changed.length > 0} onSave={submit} />
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="openings-title">
            <h1 className="type-title" />
          </Localized>
          <Localized id="openings-help">
            <p className="type-body text-ink-muted" />
          </Localized>
          {lockedRows.size === 0 ? null : (
            <Localized id="openings-locked-help">
              <p className="type-body text-ink-muted" />
            </Localized>
          )}
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            shortcut={SHORTCUTS.back}
            onClick={() => {
              void navigate({ to: '/index/accounts' });
            }}
          >
            <Localized id="common-back" />
          </Button>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            busy={flight.busy || save.isPending}
            disabled={changed.length === 0}
            onClick={submitNow}
          >
            <Localized id="layout-save-save" />
          </Button>
        </div>
      </div>
      <div className="flex flex-wrap items-start gap-x-6 gap-y-3">
        <div className="w-full max-w-field">
          <Field
            label={<Localized id="openings-date-all" />}
            hint={<Localized id="openings-date-all-hint" />}
          >
            {(props) => <DateInput {...props} value={commonDate} onChange={setCommon} />}
          </Field>
        </div>
        <section className="flex min-w-0 flex-1 flex-col gap-1 type-body text-ink">
          <Localized id="openings-summary-title">
            <h2 className="type-label" />
          </Localized>
          {changed.length === 0 ? (
            <Localized id="openings-none-changed">
              <p className="text-ink-muted" />
            </Localized>
          ) : (
            <Localized id="openings-changed" vars={{ count: changed.length }}>
              <p />
            </Localized>
          )}
          {total === undefined ? null : (
            <div className="flex flex-wrap gap-x-6 gap-y-1">
              <span className="flex items-baseline gap-2">
                <Localized id="accounts-total-dr" />
                <Num paisa={total.total_dr_paisa} />
              </span>
              <span className="flex items-baseline gap-2">
                <Localized id="accounts-total-cr" />
                <Num paisa={total.total_cr_paisa} />
              </span>
              <span className="flex items-baseline gap-2">
                <Localized id="openings-difference" />
                <Balance balance={total.opening_difference_after} />
              </span>
            </div>
          )}
          <Localized id="openings-difference-help">
            <p className="text-ink-muted" />
          </Localized>
        </section>
      </div>
      {notice === null ? null : <ErrorNotice error={notice} />}
      {problems.length === 0 ? null : (
        <section role="alert" className="flex flex-col gap-1 text-due">
          <Localized id="openings-errors-title">
            <h2 className="type-body font-semibold" />
          </Localized>
          <ul className="flex flex-col gap-1 type-body">
            {problems.map((problem) => {
              const account = accountOfRow.get(problem.rowId);
              const fieldId = FIELD_LABEL[problem.field];
              return (
                <li key={`${problem.rowId}-${problem.field}-${problem.item.code}`}>
                  {l10n.getString('openings-problem', {
                    account: account === undefined ? '' : displayName(account, language),
                    field: fieldId === undefined ? '' : l10n.getString(fieldId),
                    message: messageOf(problem.item),
                  })}
                </li>
              );
            })}
          </ul>
        </section>
      )}
      <div className={accounts.length <= SHORT_GRID_ROWS ? '' : 'min-h-0 flex-1'}>
        <EntryGrid
          columns={columns}
          store={store}
          labelId="openings-grid-label"
          calc={calc}
          cellErrors={cellErrors}
          fixedRows
          heightMode={accounts.length <= SHORT_GRID_ROWS ? 'auto' : 'fill'}
        />
      </div>
    </div>
  );
}

export function OpeningsPage() {
  const navigate = useNavigate();
  const list = useAccountList(ALL_ACTIVE);
  const day = useDayStatus();
  if ((list.data === undefined && list.isPending) || day.isPending) return <LoadingState />;
  if (list.isError || day.isError) {
    return (
      <ErrorState
        title={<Localized id="openings-load-failed" />}
        error={uiErrorOf(list.isError ? list.error : day.error)}
        onRetry={() => {
          void (list.isError ? list.refetch() : day.refetch());
        }}
      />
    );
  }
  const accounts = list.data.rows;
  if (accounts.length === 0) {
    return (
      <EmptyState
        icon={Landmark}
        title={<Localized id="openings-empty" />}
        help={<Localized id="openings-empty-help" />}
        action={
          <Button
            variant="primary"
            onClick={() => {
              void navigate({ to: '/index/accounts/new' });
            }}
          >
            <Plus aria-hidden="true" className="size-icon-md" />
            <Localized id="accounts-new" />
          </Button>
        }
      />
    );
  }
  return <OpeningsEditor accounts={accounts} today={day.data.today} />;
}
