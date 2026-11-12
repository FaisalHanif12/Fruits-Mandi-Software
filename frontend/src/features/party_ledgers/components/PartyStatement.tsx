// Level 2 of the party ledgers screen (PRD FR-12.43, FR-12.47, plan 74): the statement of one party, a
// row for each date on which he has an entry, newest first, 30 days at a time. The first window ends
// today; "پچھلے 30 دن" and "اگلے 30 دن" under the table move it, with the days of the window between
// them, and the later button stops at today. Page Up and Page Down do the same; the arrow keys follow the
// reading direction when the buttons have focus. The window's totals are the backend's: the balance at
// its start, what it charged, what was paid, and the balance at its end, which for the newest window is
// his balance now, the figure of his party card and of R13. The screen adds up nothing (CLAUDE.md
// rule 1).
//
// - Enter or a click on a date opens that day under its own row (the R35 slip, with its own print
//   button); a second click shuts it. Several can be open; moving the window shuts them.
// - "تاریخ پر جائیں": the date with an entry opens, highlighted, inside its window; a date with none
//   says so and shows the window around it; a date after today is refused.
// - The first and the last date are two boxes, as before: with one of them typed the statement is a
//   range, shown page by page with its own opening and closing balance, and the window buttons go away.
// - Esc goes back to the list (or shuts a print preview first).
// - Four states: loading, an error with a retry (also for a party that has no ledger), nothing in the
//   window or range (with a way out), and the rows.
// - A place at the top is left for the statement print (plan 74 T3, `actions`).

import { Localized, useLocalization } from '@fluent/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { ArrowLeft, CalendarX, Printer, SearchX } from 'lucide-react';
import { useCallback, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { LedgerSide } from '../../../bindings/LedgerSide';
import type { PartyLedgerStatement } from '../../../bindings/PartyLedgerStatement';
import { uiErrorOf } from '../../../shared/api';
import { Pager } from '../../../shared/grid';
import { useFocusFirstField } from '../../../shared/hooks';
import { isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, formNavigationProps, useEscape, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Code,
  DateInput,
  EmptyState,
  ErrorNotice,
  ErrorState,
  Field,
  FilterBar,
  GoToDate,
  ListPage,
  LoadingState,
  Notice,
  RUNNING_NUMBER,
  WindowNav,
  fieldError,
} from '../../../shared/ui';
import { PartyDayPreview, PartyDaySlip } from '../../reports';
import { partyLedgersGoToDate } from '../api';
import { partyWindowKey, usePartyStatement, usePartyWindow, useSettledDates } from '../hooks';
import {
  STATEMENT_DAYS,
  canComeNearer,
  statementEnd,
  statementWindowOf,
  stepStatementEnd,
} from '../statement-window';
import { StatementPrintAction } from '../print/StatementPrintAction';
import { PartyMarks } from './PartyMarks';
import { StatementDays } from './StatementDays';

interface PartyStatementProps {
  accountId: number;
  /** The last day of the 30 days asked for in the address, `YYYY-MM-DD`; nothing: today. */
  end: string | undefined;
  /** Back to the list of this kind of party; told the kind the backend says the party is. */
  onBack: (kind: LedgerSide | null) => void;
  /** Moves the window: the address keeps the day it ends on (nothing for today). */
  onWindow: (end: string | null) => void;
}

const NO_DAYS: ReadonlySet<string> = new Set();

/** One open day: its title, its own print button, and the R35 slip of that day for this party. */
function OpenDay({
  accountId,
  date,
  onPrint,
}: {
  accountId: number;
  date: string;
  onPrint: (date: string) => void;
}) {
  const { l10n } = useLocalization();
  const shown = isoToDisplayDate(date);
  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Localized id="party-ledgers-day-heading" vars={{ date: shown }} elems={RUNNING_NUMBER}>
          <h3 className="type-section" />
        </Localized>
        <Button
          accessibleName={l10n.getString('party-ledgers-day-print-name', { date: shown })}
          onClick={() => {
            onPrint(date);
          }}
        >
          <Printer aria-hidden="true" className="size-icon-md" />
          <Localized id="report-preview" />
        </Button>
      </div>
      <PartyDaySlip accountId={accountId} date={date} inPlace />
    </div>
  );
}

// The page gives each party his own copy of this screen (`key`), so a new party starts with no dates.
export function PartyStatement({
  accountId,
  end: addressEnd,
  onBack,
  onWindow,
}: PartyStatementProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const day = useDayStatus();
  const today = day.data?.today ?? null;
  const [from, setFrom] = useState('');
  const [to, setTo] = useState('');
  // The window stays on screen while the dates are being typed: a range is asked for once he pauses.
  const settled = useSettledDates(from, to);
  const dated = settled.from.trim() !== '' || settled.to.trim() !== '';
  const end = today === null ? null : statementEnd(addressEnd ?? null, today);
  const win = end === null ? null : statementWindowOf(end);
  const windowRead = usePartyWindow(accountId, dated ? null : win);
  const rangeRead = usePartyStatement(accountId, settled.from, settled.to, dated);
  const containerRef = useFocusFirstField<HTMLDivElement>();

  // The page last shown stays on screen while a half-typed date is refused (its words are under its own
  // box). It is state set while rendering, which React allows when the value really changed.
  const current = dated ? rangeRead.data?.pages[0] : windowRead.data;
  const placeholder = dated ? rangeRead.isPlaceholderData : windowRead.isPlaceholderData;
  const [kept, setKept] = useState<PartyLedgerStatement | null>(null);
  if (current !== undefined && !placeholder && current !== kept) setKept(current);
  const isError = dated ? rangeRead.isError : windowRead.isError;
  const refusal = isError ? uiErrorOf(dated ? rangeRead.error : windowRead.error) : null;
  const dateRefused =
    dated &&
    refusal !== null &&
    refusal.errors.length > 0 &&
    refusal.errors.every((item) => item.field === 'from' || item.field === 'to');
  // While the next page is on its way (typed dates make a range), the page last shown stays, dimmed, so
  // the party's name and the table do not vanish for a moment; an error is never hidden behind it.
  const waiting = current === undefined && !isError && kept !== null;
  const shown = dateRefused || waiting ? kept : (current ?? null);
  const head = shown?.head ?? null;
  const kind: LedgerSide | null = head === null ? null : head.kind;

  // What is open and where the keys are belongs to the window or the page on screen: a new one starts
  // with every day shut (PRD FR-12.47), whichever way it was reached.
  const viewKey = dated
    ? `range:${settled.from}|${settled.to}|${String(rangeRead.pager.page)}`
    : `window:${end ?? ''}`;
  const [openState, setOpenState] = useState<{ key: string; days: ReadonlySet<string> }>({
    key: viewKey,
    days: NO_DAYS,
  });
  const open = openState.key === viewKey ? openState.days : NO_DAYS;
  const [activeState, setActiveState] = useState<{ key: string; date: string | null }>({
    key: viewKey,
    date: null,
  });
  const active = activeState.key === viewKey ? activeState.date : null;
  const [noteState, setNoteState] = useState<{ key: string; date: string } | null>(null);
  const note = noteState !== null && noteState.key === viewKey ? noteState.date : null;
  const [printDay, setPrintDay] = useState<string | null>(null);

  const toggle = (date: string, wantOpen: boolean) => {
    const days = new Set(open);
    if (wantOpen) days.add(date);
    else days.delete(date);
    setOpenState({ key: viewKey, days });
  };

  const queryClient = useQueryClient();
  const goToDate = useMutation({
    mutationFn: (date: string) =>
      partyLedgersGoToDate({ account_id: accountId, date: isoToDisplayDate(date) }),
    onSuccess: (answer, date) => {
      // The typed dates of a range go: the date is looked for in 30 days, not in a range.
      setFrom('');
      setTo('');
      // The answer carries the window's own page and totals: it is the window read, so it is shown with
      // no second call.
      queryClient.setQueryData(
        partyWindowKey(accountId, { from: answer.from, to: answer.to }),
        answer.statement,
      );
      const key = `window:${answer.to}`;
      if (answer.has_entry) {
        setOpenState({ key, days: new Set([date]) });
        setActiveState({ key, date });
        setNoteState(null);
      } else {
        setNoteState({ key, date });
      }
      onWindow(answer.to === today ? null : answer.to);
    },
  });

  const back = useCallback(() => {
    onBack(kind);
  }, [onBack, kind]);
  useEscape('leave', () => {
    if (printDay !== null) setPrintDay(null);
    else back();
  });

  // A move of the window shuts every day and forgets the date the keys were on, also when the window
  // comes back to the one it left (PRD FR-12.47).
  const moveTo = (next: string | null) => {
    setOpenState({ key: '', days: NO_DAYS });
    setActiveState({ key: '', date: null });
    setNoteState(null);
    onWindow(next);
  };
  const move = (direction: -1 | 1) => {
    if (end === null || today === null || dated) return;
    const next = stepStatementEnd(end, direction, today);
    if (next !== end) moveTo(next === today ? null : next);
  };
  // Page Up and Page Down move the window, but not while the keys are inside an open day (its goods
  // list turns its own pages there).
  const windowKeys = !dated && today !== null && printDay === null;
  const outsideDay = () => {
    const focused = document.activeElement;
    return !(focused instanceof Element) || focused.closest('[data-day-detail]') === null;
  };
  useShortcut(
    SHORTCUTS.pastEarlier,
    () => {
      if (outsideDay()) move(-1);
    },
    { enabled: windowKeys },
  );
  useShortcut(
    SHORTCUTS.pastLater,
    () => {
      if (outsideDay()) move(1);
    },
    { enabled: windowKeys },
  );

  const titleHelp = (
    <Localized id="party-ledgers-help">
      <p />
    </Localized>
  );

  if (printDay !== null) {
    return (
      <ListPage titleId="party-ledgers-title" help={titleHelp}>
        <div className="min-h-0 flex-1 overflow-auto">
          <PartyDayPreview
            accountId={accountId}
            date={printDay}
            onBack={() => {
              setPrintDay(null);
            }}
          />
        </div>
      </ListPage>
    );
  }

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
      title={<Localized id="party-ledgers-window-empty" />}
      help={<Localized id="party-ledgers-window-empty-help" />}
    />
  );

  let body;
  if (day.isError && today === null) {
    // Without today there is no window to ask for: say so, with a retry (Code Standards F8).
    body = (
      <ErrorState
        title={<Localized id="party-ledgers-statement-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  } else if (shown === null) {
    body =
      refusal !== null && !dateRefused ? (
        <ErrorState
          title={<Localized id="party-ledgers-statement-failed" />}
          error={refusal}
          onRetry={() => {
            void (dated ? rangeRead.refetch() : windowRead.refetch());
          }}
        />
      ) : (
        <LoadingState />
      );
  } else {
    const latest = end === null || today === null || !canComeNearer(end, today);
    body = (
      <div className="flex flex-col gap-3">
        {note === null ? null : (
          <Notice tone="info">
            <Localized
              id="party-ledgers-goto-none"
              vars={{ date: isoToDisplayDate(note) }}
              elems={RUNNING_NUMBER}
            >
              <span />
            </Localized>
          </Notice>
        )}
        <StatementDays
          kind={shown.head.kind}
          rows={shown.rows}
          totals={shown}
          open={open}
          onToggle={toggle}
          active={active}
          onActive={(date) => {
            setActiveState({ key: viewKey, date });
          }}
          renderDay={(date) => <OpenDay accountId={accountId} date={date} onPrint={setPrintDay} />}
          empty={empty}
          busy={placeholder || waiting}
          labelId="party-ledgers-statement-label"
        />
        {dated ? (
          <div className="shrink-0">
            <Pager pager={rangeRead.pager} />
          </div>
        ) : win === null ? null : (
          <WindowNav
            label={l10n.getString('party-ledgers-nav-label')}
            earlier={
              <Localized
                id="party-ledgers-back-days"
                vars={{ days: String(STATEMENT_DAYS) }}
                elems={RUNNING_NUMBER}
              >
                <span />
              </Localized>
            }
            earlierShortcut={SHORTCUTS.pastEarlier}
            onEarlier={() => {
              move(-1);
            }}
            later={
              <Localized
                id="party-ledgers-forward-days"
                vars={{ days: String(STATEMENT_DAYS) }}
                elems={RUNNING_NUMBER}
              >
                <span />
              </Localized>
            }
            laterShortcut={SHORTCUTS.pastLater}
            onLater={() => {
              move(1);
            }}
            {...(latest ? { laterOffNote: <Localized id="party-ledgers-nav-latest" /> } : {})}
            middle={
              <>
                <Localized
                  id="party-ledgers-range"
                  vars={{ from: isoToDisplayDate(win.from), to: isoToDisplayDate(win.to) }}
                  elems={RUNNING_NUMBER}
                >
                  <span className="type-body text-ink" />
                </Localized>
                {latest ? null : (
                  <Button
                    variant="ghost"
                    onClick={() => {
                      moveTo(null);
                    }}
                  >
                    <Localized id="party-ledgers-nav-latest-button" />
                  </Button>
                )}
              </>
            }
          />
        )}
      </div>
    );
  }

  return (
    <ListPage
      titleId="party-ledgers-title"
      help={titleHelp}
      actions={
        <>
          <StatementPrintAction accountId={accountId} />
          <Button onClick={back}>
            <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
            <Localized id="party-ledgers-back-list" />
          </Button>
        </>
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
          <div
            ref={containerRef}
            {...formNavigationProps}
            className="flex flex-wrap items-end gap-3"
          >
            <GoToDate
              today={today}
              onGo={(date) => {
                goToDate.mutate(date);
              }}
              label={<Localized id="party-ledgers-goto" />}
              info={<Localized id="party-ledgers-goto-hint" />}
              placeholder={l10n.getString('party-ledgers-goto-placeholder')}
            />
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
          {goToDate.isError ? <ErrorNotice error={uiErrorOf(goToDate.error)} /> : null}
        </div>
      }
    >
      {body}
    </ListPage>
  );
}
