// The opening balances grid (route /index/openings; plan 07, PRD FR-10.9): one row per account,
// with the opening date, debit and credit typed in the cells, so many openings can be entered at
// once. The accounts come one page at a time (plan 38 R1) with a search box, so a shop with 20,000
// accounts never reads them all at once. Only the rows the user changed go to the backend, as the
// text typed. The backend checks the lines and works out the totals and what ابتدائی فرق would be
// (the same function as the save, Code Standards B5); the screen only shows them. The save is all
// or nothing.
//
// - What is typed lives in the screen, never only in the grid: the edits of every account the
//   user touched are kept as text across pages and searches, and the grid is drawn again from them
//   when a page turns or the language changes (Code Standards F39).
// - Enter stops on the date, the debit and the credit of a row and goes on to the next row; the
//   rows are fixed, no key adds one. Keys: F10 or Ctrl+S saves, Esc goes back (asking first).
// - Four states: loading, error with a retry, empty with a button, and the grid.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Landmark, Plus, SearchX } from 'lucide-react';
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
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
import {
  EntryGrid,
  ListRegion,
  Pager,
  createEntryStore,
  listHeightMode,
  type EntryColumn,
  type EntryRow,
} from '../../../shared/grid';
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
  FilterBar,
  InfoHint,
  LoadingState,
  Num,
  Select,
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
/** A pause so a burst of key presses in the search box asks once. */
const SEARCH_DELAY_MS = 150;

const BASE_COLUMNS: readonly EntryColumn[] = [
  { field: 'code', type: 'calc', headerId: 'accounts-col-code' },
  { field: 'name', type: 'label', headerId: 'accounts-col-name', grow: true },
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

/** The three cells of one account as typed. */
interface Triple {
  date: string;
  dr: string;
  cr: string;
}

/**
 * Which accounts the list shows (plan 38, tech lead): the ones that still need an opening balance
 * (the start), the ones that have one, or every account. The backend filters (`has_opening`).
 */
type OpeningsShow = 'without' | 'with' | 'all';
const OPENINGS_SHOWN: readonly OpeningsShow[] = ['without', 'with', 'all'];

/** Every active account of the choice: the search box narrows it. */
function filterOf(text: string, show: OpeningsShow): AccountFilter {
  return {
    kind: 'every',
    text,
    include_inactive: false,
    has_opening: show === 'without' ? false : show === 'with' ? true : null,
  };
}

function tripleOf(typed: Readonly<Record<string, string>>): Triple {
  return {
    date: typed[DATE_FIELD] ?? '',
    dr: typed[DR_FIELD] ?? '',
    cr: typed[CR_FIELD] ?? '',
  };
}

function sameTriple(a: Triple, b: Triple): boolean {
  return a.date === b.date && a.dr === b.dr && a.cr === b.cr;
}

function OpeningsScreen({ today }: { today: string }) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const navigate = useNavigate();
  const [search, setSearch] = useState('');
  const [show, setShow] = useState<OpeningsShow>('without');
  const searchText = useDebouncedValue(search.trim(), SEARCH_DELAY_MS);
  const list = useAccountList(filterOf(searchText, show), 'openings.list');
  const first = list.data?.pages[0];
  const accounts = list.isError ? undefined : first?.rows;
  const [store] = useState(() => createEntryStore());
  const [commonDate, setCommonDate] = useState(() => isoToDisplayDate(today));
  // What each account looked like the first time the screen saw it, to tell what the user changed.
  const [baselines, setBaselines] = useState<ReadonlyMap<string, OpeningBaseline>>(new Map());
  // What the user typed, per account, for every account he touched: it stays across pages (F39).
  const [edits, setEdits] = useState<Readonly<Record<string, Triple>>>({});
  // Every account the screen has seen, for the names in the list of problems.
  const [seen, setSeen] = useState<ReadonlyMap<string, AccountRow>>(new Map());
  // The latest of these, for the places that must not run again when they change.
  const baselinesRef = useRef(baselines);
  const editsRef = useRef(edits);
  const commonDateRef = useRef(commonDate);
  const groupingRef = useRef(grouping);
  useEffect(() => {
    baselinesRef.current = baselines;
    editsRef.current = edits;
    commonDateRef.current = commonDate;
    groupingRef.current = grouping;
  });
  const save = useSaveOpenings();
  // The problems a refused save pointed at, kept only while the lines are the ones that were sent.
  const [refused, setRefused] = useState<{ error: UiError; sent: readonly ChangedLine[] } | null>(
    null,
  );

  // The page on screen is drawn into the grid's store from the accounts and what was typed before.
  // The row of an account is its id, so a problem finds its cell.
  useLayoutEffect(() => {
    if (accounts === undefined) return;
    const nextBaselines = new Map(baselinesRef.current);
    const drawn: EntryRow[] = accounts.map((account) => {
      const id = String(account.id);
      const typed = openingTyped(account, groupingRef.current, commonDateRef.current);
      if (!nextBaselines.has(id)) nextBaselines.set(id, baselineOf(account, typed));
      const before = editsRef.current[id];
      return {
        rowId: id,
        typed:
          before === undefined
            ? typed
            : { ...typed, [DATE_FIELD]: before.date, [DR_FIELD]: before.dr, [CR_FIELD]: before.cr },
      };
    });
    baselinesRef.current = nextBaselines;
    setBaselines(nextBaselines);
    setSeen((previous) => {
      const next = new Map(previous);
      for (const account of accounts) next.set(String(account.id), account);
      return next;
    });
    store.setState({ rows: drawn });
  }, [accounts, store]);

  // What is typed goes into `edits`, an account at a time, and leaves it again when it is back to
  // what the account had.
  useEffect(
    () =>
      store.subscribe((state) => {
        let next: Record<string, Triple> | null = null;
        for (const row of state.rows) {
          const id = row.typed['account_id'];
          if (id === undefined) continue;
          const base = baselinesRef.current.get(id);
          if (base === undefined) continue;
          const typed = tripleOf(row.typed);
          const had = editsRef.current[id];
          const differs = !sameTriple(typed, tripleOf(base.typed));
          if (differs && (had === undefined || !sameTriple(had, typed))) {
            next ??= { ...editsRef.current };
            next[id] = typed;
          } else if (!differs && had !== undefined) {
            next ??= { ...editsRef.current };
            Reflect.deleteProperty(next, id);
          }
        }
        if (next !== null) {
          editsRef.current = next;
          setEdits(next);
        }
      }),
    [store],
  );

  const editRows = useMemo<EntryRow[]>(
    () =>
      Object.entries(edits).map(([id, triple]) => ({
        rowId: id,
        typed: {
          account_id: id,
          [DATE_FIELD]: triple.date,
          [DR_FIELD]: triple.dr,
          [CR_FIELD]: triple.cr,
        },
      })),
    [edits],
  );
  const changed = useMemo(() => changedLines(editRows, baselines), [editRows, baselines]);
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
    for (const account of accounts ?? []) {
      if (!account.can_take_opening) locked.add(String(account.id));
    }
    return locked;
  }, [accounts]);
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
        (accounts ?? []).map((account) => [
          String(account.id),
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
    [accounts, language, l10n],
  );

  const setCommon = (next: string) => {
    const previous = commonDate;
    setCommonDate(next);
    // An account with no opening follows the common date until its own date is typed.
    const follows = (id: string, date: string | undefined) => {
      const base = baselinesRef.current.get(id);
      return base?.canTake === true && !base.hasOpening && date === previous;
    };
    store
      .getState()
      .updateTypedWhere(
        DATE_FIELD,
        (row) => follows(row.typed['account_id'] ?? '', row.typed[DATE_FIELD]),
        next,
      );
    // The same for what was typed on the pages that are not on screen.
    setEdits((now) =>
      Object.fromEntries(
        Object.entries(now).map(([id, triple]) => [
          id,
          follows(id, triple.date) ? { ...triple, date: next } : triple,
        ]),
      ),
    );
  };

  const applySaved = (saved: OpeningsSaved) => {
    const next = new Map(baselines);
    const gone = new Set<string>();
    for (const account of saved.rows) {
      const id = String(account.id);
      const typed = openingTyped(account, grouping, commonDate);
      next.set(id, baselineOf(account, typed));
      gone.add(id);
      for (const field of [DATE_FIELD, DR_FIELD, CR_FIELD]) {
        store.getState().updateTyped(id, field, typed[field] ?? '');
      }
    }
    baselinesRef.current = next;
    setBaselines(next);
    setEdits((now) => Object.fromEntries(Object.entries(now).filter(([id]) => !gone.has(id))));
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
  const searching = searchText !== '';

  let body;
  if (accounts === undefined && list.isPending) {
    body = <LoadingState />;
  } else if (list.isError) {
    body = (
      <ErrorState
        title={<Localized id="openings-load-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (accounts?.length === 0) {
    body = searching ? (
      <EmptyState
        icon={SearchX}
        title={<Localized id="accounts-no-match-title" />}
        help={<Localized id="accounts-no-match-help" />}
        action={
          <Button
            onClick={() => {
              setSearch('');
            }}
          >
            <Localized id="accounts-no-match-clear" />
          </Button>
        }
      />
    ) : show !== 'all' ? (
      <EmptyState
        icon={Landmark}
        title={<Localized id={`openings-none-${show}`} />}
        help={<Localized id={`openings-none-${show}-help`} />}
        action={
          <Button
            onClick={() => {
              setShow('all');
            }}
          >
            <Localized id="openings-show-all-button" />
          </Button>
        }
      />
    ) : (
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
  } else if (accounts !== undefined) {
    body = (
      <div className="flex min-h-0 flex-1 flex-col gap-2">
        <ListRegion rowCount={accounts.length}>
          <EntryGrid
            columns={columns}
            store={store}
            labelId="openings-grid-label"
            calc={calc}
            cellErrors={cellErrors}
            fixedRows
            heightMode={listHeightMode(accounts.length)}
          />
        </ListRegion>
        <div className="shrink-0">
          <Pager pager={list.pager} />
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-3 short:gap-2">
      <LeaveGuard dirty={changed.length > 0} onSave={submit} />
      <div className="flex shrink-0 flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <div className="flex items-center gap-1">
          <Localized id="openings-title">
            <h1 className="type-title" />
          </Localized>
          <InfoHint>
            <Localized id="openings-help">
              <p />
            </Localized>
            {lockedRows.size === 0 ? null : (
              <Localized id="openings-locked-help">
                <p />
              </Localized>
            )}
          </InfoHint>
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
      <div className="shrink-0">
        <FilterBar screen="openings" search={{ value: search, onChange: setSearch }}>
          <Field label={<Localized id="openings-show-label" />}>
            {(props) => (
              <Select<OpeningsShow>
                {...props}
                value={show}
                onChange={setShow}
                options={OPENINGS_SHOWN.map((option) => ({
                  value: option,
                  label: <Localized id={`openings-show-${option}`} />,
                }))}
              />
            )}
          </Field>
          <Field
            label={<Localized id="openings-date-all" />}
            info={
              <Localized id="openings-date-all-hint">
                <p />
              </Localized>
            }
          >
            {(props) => <DateInput {...props} value={commonDate} onChange={setCommon} />}
          </Field>
        </FilterBar>
      </div>
      {/* What will be saved and what it does to the books, in one line (the backend's figures). */}
      <section className="flex shrink-0 flex-wrap items-baseline gap-x-6 gap-y-1 type-body text-ink">
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
          <>
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
              <InfoHint>
                <Localized id="openings-difference-help">
                  <p />
                </Localized>
              </InfoHint>
            </span>
          </>
        )}
      </section>
      {notice === null ? null : <ErrorNotice error={notice} />}
      {problems.length === 0 ? null : (
        <section role="alert" className="flex shrink-0 flex-col gap-1 text-due">
          <Localized id="openings-errors-title">
            <h2 className="type-body font-semibold" />
          </Localized>
          <ul className="flex flex-col gap-1 type-body">
            {problems.map((problem) => {
              const account = seen.get(problem.rowId);
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
      {body}
    </div>
  );
}

export function OpeningsPage() {
  const day = useDayStatus();
  if (day.isPending) return <LoadingState />;
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="openings-load-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  return <OpeningsScreen today={day.data.today} />;
}
