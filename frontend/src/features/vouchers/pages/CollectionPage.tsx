// اگراہی فہرست (route /bikri/collection; PRD FR-15.6, FR-7.61, FR-15.1): the buyers planned for a day,
// by collector and city, with their dues from the backend. The round is one list already filled:
// the munshi types only the amounts received and saves them together. Each row is its own receipt
// and its own posting event, so a row the backend refuses keeps its message at its cell while the
// others are saved; a row left empty (or at 0) is skipped.
//
// - What is typed lives in the entry store, never only in the grid, so a language switch rebuilds
//   the grid and loses nothing (Code Standards F39).
// - Enter goes down the amount cells, row by row; the rows are fixed. Keys: F10 or Ctrl+S saves the
//   round, Esc goes back (asking first).
// - The frontend works out no money: the amounts go as typed; the dues, the days and the total
//   saved are the backend's.
// - Four states: loading, an error with a retry, nothing planned (with words), and the list.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { ClipboardList } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import { useDayStatus } from '../../../app/use-day-status';
import type { CollectionListInput } from '../../../bindings/CollectionListInput';
import type { UiError } from '../../../bindings/UiError';
import type { UiErrorItem } from '../../../bindings/UiErrorItem';
import {
  uiErrorOf,
  errorMessageId,
  messageVars,
  FALLBACK_ERROR_MESSAGE_ID,
} from '../../../shared/api';
import { isoToDisplayDate, useGrouping } from '../../../shared/format';
import { EntryGrid, FIT_MIN, createEntryStore, type EntryColumn } from '../../../shared/grid';
import { useSingleFlight } from '../../../shared/hooks';
import { useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Card,
  DateInput,
  EmptyState,
  ErrorNotice,
  ErrorState,
  Field,
  LoadingState,
  Num,
  Picker,
  showToast,
  type PickerValue,
} from '../../../shared/ui';
import {
  RECEIVED_FIELD,
  calcOfRow,
  collectionRows,
  receiptsOf,
  rowPathOf,
  type CollectionRow,
} from '../collection';
import { useCollectionList, useSaveReceiptsBatch } from '../hooks';
import type { CollectionList, ReceiptsSaved } from '../types';

/** A grid of this many rows or fewer is as tall as its rows; a longer one fills the screen. */
const SHORT_GRID_ROWS = 30;

const COLUMNS: readonly EntryColumn[] = [
  { field: 'code', type: 'calc', headerId: 'accounts-col-code' },
  { field: 'name', type: 'label', headerId: 'accounts-col-name', grow: true },
  { field: 'group', type: 'label', headerId: 'vouchers-collection-col-group' },
  { field: 'dues', type: 'calc', headerId: 'vouchers-collection-col-dues' },
  { field: 'days', type: 'calc', headerId: 'vouchers-col-days' },
  {
    field: RECEIVED_FIELD,
    type: 'money',
    headerId: 'vouchers-collection-col-received',
    minWidth: FIT_MIN.longHeading,
  },
];

/** A row the backend refused: the amount typed and what it said, kept for the rebuilt round. */
interface Refused {
  amount: string;
  problems: { field: string; item: UiErrorItem }[];
}

interface RoundProps {
  list: CollectionList;
  /** The day of the round as typed, `DD-MM-YYYY`. */
  date: string;
  /** Rows refused in the round before, by buyer, for the rebuilt round. */
  carried: ReadonlyMap<number, Refused>;
  /** The round was saved: the screen reads the list again and builds a new round. */
  onSaved: (saved: ReceiptsSaved, refused: ReadonlyMap<number, Refused>) => void;
}

/** One round: the grid of the list, the problems of a refused save, and the buttons. */
function CollectionRound({ list, date, carried, onSaved }: RoundProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const navigate = useNavigate();
  const rows = useMemo(() => collectionRows(list), [list]);
  const [start] = useState(() => {
    const store = createEntryStore(
      rows.map((row) => ({ [RECEIVED_FIELD]: carried.get(row.accountId)?.amount ?? '' })),
    );
    const rowOf = new Map<string, CollectionRow>();
    const refused: Record<string, Refused['problems']> = {};
    store.getState().rows.forEach((entry, index) => {
      const row = rows[index];
      if (row === undefined) return;
      rowOf.set(entry.rowId, row);
      const before = carried.get(row.accountId);
      if (before !== undefined) refused[entry.rowId] = before.problems;
    });
    return { store, rowOf, refused };
  });
  const { store, rowOf } = start;
  const entries = store((state) => state.rows);
  const save = useSaveReceiptsBatch();
  // What a refused row said, by row and field, until the next save.
  const [problems, setProblems] = useState<Record<string, Refused['problems']>>(start.refused);
  const [notice, setNotice] = useState<UiError | null>(null);

  const typed = useMemo(() => {
    const map = new Map<number, string>();
    for (const entry of entries) {
      const row = rowOf.get(entry.rowId);
      if (row !== undefined) map.set(row.accountId, entry.typed[RECEIVED_FIELD] ?? '');
    }
    return map;
  }, [entries, rowOf]);
  const filled = useMemo(
    () => [...typed.values()].filter((text) => text.trim() !== '').length,
    [typed],
  );

  const separator = l10n.getString('vouchers-collection-group-separator');
  const calc = useMemo(
    () =>
      Object.fromEntries(
        [...rowOf].map(([rowId, row]) => [rowId, calcOfRow(row, language, grouping, separator)]),
      ),
    [rowOf, language, grouping, separator],
  );
  // The words follow the language and the grouping, so a switch rewrites them.
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
    for (const [rowId, items] of Object.entries(problems)) {
      for (const one of items) (byRow[rowId] ??= {})[one.field] ??= messageOf(one.item);
    }
    return byRow;
  }, [problems, messageOf]);

  const flight = useSingleFlight();
  const submitOnce = async (): Promise<boolean> => {
    const { sent, receipts } = receiptsOf(rows, typed, date);
    if (sent.length === 0) return true;
    try {
      const saved = await save.mutateAsync(receipts);
      setNotice(null);
      const next: Record<string, Refused['problems']> = {};
      const refused = new Map<number, Refused>();
      saved.rows.forEach((result, index) => {
        const row = sent[index];
        if (row === undefined) return;
        const rowId = [...rowOf].find(([, candidate]) => candidate === row)?.[0];
        if (rowId === undefined) return;
        if (result.saved === null && !result.skipped) {
          const problems = result.errors.map((item) => ({
            field: rowPathOf(item.field)?.field === 'amount' ? RECEIVED_FIELD : 'code',
            item,
          }));
          refused.set(row.accountId, { amount: typed.get(row.accountId) ?? '', problems });
          next[rowId] = problems;
        }
      });
      setProblems(next);
      showToast(
        refused.size === 0 ? 'success' : 'error',
        l10n.getString(
          refused.size === 0 ? 'vouchers-collection-saved' : 'vouchers-collection-some-refused',
          {
            count: String(saved.rows.filter((result) => result.saved !== null).length),
            refused: String(refused.size),
          },
        ),
      );
      onSaved(saved, refused);
      return refused.size === 0;
    } catch (failure: unknown) {
      setNotice(uiErrorOf(failure));
      return false;
    }
  };
  const submit = (): Promise<boolean> => flight.run(submitOnce, false);
  const submitNow = () => {
    void submit();
  };
  useShortcut(SHORTCUTS.save, submitNow);
  useEscape('leave', () => {
    void navigate({ to: '/bikri/voucher', search: { type: 'receipt' } });
  });

  const lines = Object.entries(problems).flatMap(([rowId, items]) => {
    const row = rowOf.get(rowId);
    return items.map((one) => ({ rowId, row, text: messageOf(one.item) }));
  });

  return (
    <div className="flex flex-col gap-4">
      <LeaveGuard dirty={filled > 0} onSave={submit} />
      <div className="flex flex-wrap items-center justify-between gap-3">
        <Localized id="vouchers-collection-count" vars={{ count: String(rows.length) }}>
          <p className="type-body text-ink-muted" />
        </Localized>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy || save.isPending}
          disabled={filled === 0}
          onClick={submitNow}
        >
          <Localized id="vouchers-collection-save" />
        </Button>
      </div>
      {notice === null ? null : <ErrorNotice error={notice} />}
      {lines.length === 0 ? null : (
        <section role="alert" className="flex flex-col gap-1 text-due">
          <Localized id="vouchers-collection-problems">
            <h2 className="type-body font-semibold" />
          </Localized>
          <ul className="flex flex-col gap-1 type-body">
            {lines.map((line, index) => (
              <li key={`${line.rowId}-${String(index)}`}>
                {line.row === undefined ? '' : `${calc[line.rowId]?.['name'] ?? ''}: `}
                {line.text}
              </li>
            ))}
          </ul>
        </section>
      )}
      <div className={rows.length <= SHORT_GRID_ROWS ? '' : 'min-h-0 flex-1'}>
        <EntryGrid
          columns={COLUMNS}
          store={store}
          labelId="vouchers-collection-grid-label"
          calc={calc}
          cellErrors={cellErrors}
          fixedRows
          fit
          heightMode={rows.length <= SHORT_GRID_ROWS ? 'auto' : 'fill'}
        />
      </div>
    </div>
  );
}

export function CollectionPage() {
  const navigate = useNavigate();
  const day = useDayStatus();
  // The day field is empty until today is known, then starts as today; the user may change it.
  const [dayText, setDayText] = useState<string | null>(null);
  const [collector, setCollector] = useState<PickerValue | null>(null);
  const [city, setCity] = useState<PickerValue | null>(null);
  // A new round is built when the list is read again after a save; the refused amounts carry over.
  const [round, setRound] = useState<{ n: number; carried: ReadonlyMap<number, Refused> }>({
    n: 0,
    carried: new Map(),
  });
  const today = day.data === undefined ? '' : isoToDisplayDate(day.data.today);
  const date = dayText ?? today;
  const input: CollectionListInput | null =
    date.trim() === ''
      ? null
      : { date: date.trim(), collector_staff_id: collector?.id ?? null, city_id: city?.id ?? null };
  const list = useCollectionList(input);
  const total = list.data;

  let body;
  if (day.isPending || (list.isPending && input !== null)) {
    body = <LoadingState />;
  } else if (day.isError || list.isError) {
    body = (
      <ErrorState
        title={<Localized id="vouchers-collection-failed" />}
        error={uiErrorOf(day.isError ? day.error : list.error)}
        onRetry={() => {
          void (day.isError ? day.refetch() : list.refetch());
        }}
      />
    );
  } else if (total?.groups.every((group) => group.buyers.length === 0) === true) {
    body = (
      <EmptyState
        icon={ClipboardList}
        title={<Localized id="vouchers-collection-empty" />}
        help={<Localized id="vouchers-collection-empty-help" />}
        action={
          <Button
            onClick={() => {
              void navigate({ to: '/bikri/voucher', search: { type: 'receipt' } });
            }}
          >
            <Localized id="vouchers-collection-back" />
          </Button>
        }
      />
    );
  } else if (total !== undefined) {
    body = (
      <CollectionRound
        key={`${String(round.n)}-${date}-${String(collector?.id)}-${String(city?.id)}`}
        list={total}
        date={date}
        carried={round.carried}
        onSaved={(_saved, refused) => {
          setRound((current) => ({ n: current.n + 1, carried: refused }));
        }}
      />
    );
  }

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="vouchers-collection-title">
            <h1 className="type-title" />
          </Localized>
          <Localized id="vouchers-collection-help">
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        <Button
          shortcut={SHORTCUTS.back}
          onClick={() => {
            void navigate({ to: '/bikri/voucher', search: { type: 'receipt' } });
          }}
        >
          <Localized id="vouchers-collection-back" />
        </Button>
      </div>
      <Card compact>
        <div className="grid grid-cols-2 items-end gap-4 normal:grid-cols-4">
          <Field label={<Localized id="vouchers-field-date" />}>
            {(control) => <DateInput {...control} value={date} onChange={setDayText} />}
          </Field>
          <Field label={<Localized id="vouchers-field-collector" />}>
            {(control) => (
              <Picker
                {...control}
                scope="staff"
                value={collector}
                onChange={(hit) => {
                  setCollector(
                    hit === null
                      ? null
                      : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
                  );
                }}
              />
            )}
          </Field>
          <Field label={<Localized id="accounts-field-city" />}>
            {(control) => (
              <Picker
                {...control}
                scope="city"
                value={city}
                onChange={(hit) => {
                  setCity(
                    hit === null
                      ? null
                      : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
                  );
                }}
              />
            )}
          </Field>
          {total === undefined ? null : (
            <p className="flex items-baseline gap-2 type-body">
              <Localized id="vouchers-collection-total" />
              <Num paisa={total.total_paisa} />
            </p>
          )}
        </div>
      </Card>
      {body}
    </div>
  );
}
