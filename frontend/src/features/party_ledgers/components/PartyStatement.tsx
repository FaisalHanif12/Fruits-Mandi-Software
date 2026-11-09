// Level 2 of the party ledgers screen (PRD FR-12.43): the statement of one party, a row for each
// date on which he has an entry, newest first: the balance before the day, what the day charged,
// what was paid, and the balance after it, each balance with its side word. The footer is the
// backend's: the balance before the first date asked for, the totals of the dates, and the balance
// after the last one, which with no dates typed is his balance now, the figure of his party card and
// of R13 (PRD FR-19.11). The screen adds up nothing (CLAUDE.md rule 1).
//
// - Enter (or a double click) on a row opens that day's slip, level 3; Esc and the button go back to
//   the list.
// - The first and the last date are two boxes; a date the backend refuses (a half-typed one) shows
//   its words under its own box and the rows on screen stay (Code Standards F35).
// - Four states: loading, an error with a retry (also for a party that has no ledger), nothing yet or
//   nothing in the period (with a way out), and the rows.

import { Localized } from '@fluent/react';
import { ArrowLeft, CalendarX, SearchX } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import type { LedgerSide } from '../../../bindings/LedgerSide';
import type { PartyLedgerDay } from '../../../bindings/PartyLedgerDay';
import type { PartyLedgerStatement } from '../../../bindings/PartyLedgerStatement';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListContext } from '../../../shared/grid';
import { useFocusFirstField } from '../../../shared/hooks';
import { formNavigationProps, useEscape } from '../../../shared/keyboard';
import {
  Button,
  Code,
  DateInput,
  EmptyState,
  Field,
  FilterBar,
  ListPage,
  fieldError,
} from '../../../shared/ui';
import { useLanguage, displayName } from '../../../shared/i18n';
import { statementColumns, statementTotals } from '../columns';
import { usePartyStatement } from '../hooks';
import { PartyMarks } from './PartyMarks';

interface PartyStatementProps {
  accountId: number;
  /** Back to the list of this kind of party; told the kind the backend says the party is. */
  onBack: (kind: LedgerSide | null) => void;
  /** Opens the slip of a date (`YYYY-MM-DD`). */
  onOpenDay: (date: string) => void;
}

// The page gives each party his own copy of this screen (`key`), so a new party starts with no dates.
export function PartyStatement({ accountId, onBack, onOpenDay }: PartyStatementProps) {
  const language = useLanguage();
  const [from, setFrom] = useState('');
  const [to, setTo] = useState('');
  const query = usePartyStatement(accountId, from, to);
  const containerRef = useFocusFirstField<HTMLDivElement>();
  const [focusId, setFocusId] = useState<string | null>(null);

  // The page last shown stays on screen while a half-typed date is refused (its words are under its
  // own box). It is state set while rendering, which React allows when the value really changed.
  const current = query.data?.pages[0];
  const [kept, setKept] = useState<PartyLedgerStatement | null>(null);
  if (current !== undefined && !query.isPlaceholderData && current !== kept) setKept(current);
  const refusal = query.isError ? uiErrorOf(query.error) : null;
  const dateRefused =
    refusal !== null &&
    refusal.errors.length > 0 &&
    refusal.errors.every((item) => item.field === 'from' || item.field === 'to');
  const shown = dateRefused ? kept : (current ?? null);
  const head = shown?.head ?? null;
  const kind: LedgerSide | null = head === null ? null : head.kind;

  const back = useCallback(() => {
    onBack(kind);
  }, [onBack, kind]);
  useEscape('leave', back);

  const rows = shown === null ? undefined : shown.rows;
  const columns = useMemo(() => statementColumns(kind ?? 'buyer'), [kind]);
  const totals = useCallback(
    (context: ListContext) => statementTotals(shown ?? undefined, context),
    [shown],
  );
  const dated = from.trim() !== '' || to.trim() !== '';

  const empty = dated ? (
    <EmptyState
      icon={SearchX}
      title={<Localized id="party-ledgers-statement-no-match" />}
      help={<Localized id="party-ledgers-statement-no-match-help" />}
      action={
        <Button
          onClick={() => {
            setFrom('');
            setTo('');
          }}
        >
          <Localized id="party-ledgers-clear-dates" />
        </Button>
      }
    />
  ) : (
    <EmptyState
      icon={CalendarX}
      title={<Localized id="party-ledgers-statement-empty" />}
      help={<Localized id="party-ledgers-statement-empty-help" />}
    />
  );

  return (
    <ListPage
      titleId="party-ledgers-title"
      help={
        <Localized id="party-ledgers-help">
          <p />
        </Localized>
      }
      actions={
        <Button onClick={back}>
          <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
          <Localized id="party-ledgers-back-list" />
        </Button>
      }
      filters={
        <div className="flex flex-col gap-3">
          {head === null ? null : (
            <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
              <h2 className="type-section">{displayName(head, language)}</h2>
              <Code>{head.code}</Code>
              <PartyMarks isDead={head.is_dead} isActive={head.is_active} />
            </div>
          )}
          <div ref={containerRef} {...formNavigationProps}>
            <FilterBar screen="party_ledgers.statement">
              <Field
                label={<Localized id="party-ledgers-from" />}
                error={dateRefused ? fieldError(refusal, 'from') : undefined}
              >
                {(control) => <DateInput {...control} value={from} onChange={setFrom} />}
              </Field>
              <Field
                label={<Localized id="party-ledgers-to" />}
                error={dateRefused ? fieldError(refusal, 'to') : undefined}
              >
                {(control) => <DateInput {...control} value={to} onChange={setTo} />}
              </Field>
            </FilterBar>
          </div>
        </div>
      }
    >
      <PagedList<PartyLedgerDay>
        columns={columns}
        rows={rows}
        getRowId={(row) => row.date}
        labelId="party-ledgers-statement-label"
        failedId="party-ledgers-statement-failed"
        error={refusal !== null && !dateRefused ? refusal : null}
        onRetry={() => {
          void query.refetch();
        }}
        pager={query.pager}
        empty={empty}
        onOpen={(row) => {
          onOpenDay(row.date);
        }}
        onFocusRow={setFocusId}
        focusRowId={focusId}
        totals={totals}
      />
    </ListPage>
  );
}
