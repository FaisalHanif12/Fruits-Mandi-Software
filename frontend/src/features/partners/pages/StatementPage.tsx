// The partner statement (route /partners/statement; PRD FR-17.12, FR-17.10, FR-19.11, R31, plan 76 point 11): what
// came into and went out of one partner's کل رقم, one month at a time. It opens on the current month and can
// go back to the first month of data. The top row is his opening کل رقم of the month (carried from before; the
// first month starts with his opening capital); then one row for each day with a movement: commission, expense
// share, gardens and trips, fruit taken, money taken and returned, capital in and out, the day's net and his کل
// رقم at the end of the day; under them the month's totals. The closing of the current month is the کل رقم of
// the overview and of R31. A click or Enter on a day opens every movement of that day in place. Every figure
// is the backend's, read from posted records; the screen adds nothing up (CLAUDE.md rule 1).
//
// - The partner is a drop-down; the month is in the address (`?month=2026-10`), so a language switch, a reload
//   and the way back from an entry land on the same month. Previous and next month, go to month, Page Up and
//   Page Down, as the party statement and پچھلی بکری.
// - The page is behind the partner code (PRD FR-17.11).
// - Four states: loading, an error with a retry, no partner chosen, and the days (a month with no movement
//   says so, and the buttons still work).

import { Localized } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { Printer, ScrollText } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { addMonths, isIsoMonth, monthOf, windowMonth } from '../../../shared/format';
import { useWindowParam } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, formNavigationProps, useEscape, useShortcut } from '../../../shared/keyboard';
import { Remembered, type MemoryProps } from '../../../shared/screen-state';
import {
  Button,
  EmptyState,
  ErrorState,
  Field,
  FilterBar,
  FilterCard,
  FormSpan,
  GoToMonth,
  ListPage,
  LoadingState,
  MonthNav,
  Select,
} from '../../../shared/ui';
import { useOpenDays } from '../../dashboard/components/CommissionDays';
import { ExcelExportButton, ExcelExportNotices } from '../../../shared/export';
import { TablePrintPreview, useTableExcel } from '../../reports';
import { partnersStatementMonthPrint, partnersStatementMonthPrintData } from '../api';
import { PartnerGate } from '../components/PartnerGate';
import { StatementMonthDays } from '../components/StatementMonthDays';
import { usePartnerList } from '../hooks';
import { usePartnerStatementMonth } from '../money-hooks';
import { STATEMENT_SCREEN, statementChoicesFrom, statementChoicesText } from '../remember';
import { parseStatementSearch } from '../screen-type';

/** The statement starts as the Owner left it: the partner (PRD FR-7.45). */
function StatementPageScreen() {
  return (
    <Remembered screen={STATEMENT_SCREEN}>{(memory) => <StatementBody {...memory} />}</Remembered>
  );
}

function StatementBody({ saved, remember }: MemoryProps) {
  const language = useLanguage();
  const navigate = useNavigate();
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  const search = parseStatementSearch(rawSearch);
  const partners = usePartnerList();
  // An address with a partner in it (the overview's button) wins over what was kept, once, when it opens.
  const [partnerId, setPartnerId] = useState(() =>
    search.partner === undefined ? statementChoicesFrom(saved).partnerId : String(search.partner),
  );
  useEffect(() => {
    remember(statementChoicesText({ partnerId, from: '', to: '' }));
  }, [partnerId, remember]);

  const [view, setView] = useState<'table' | 'preview'>('table');
  useEscape('leave', () => {
    if (view === 'preview') setView('table');
    else void navigate({ to: '/partners' });
  });
  useShortcut(SHORTCUTS.print, () => {
    if (view === 'table') setView('preview');
  });
  const day = useDayStatus();
  const [asked, setAsked] = useWindowParam('month', isIsoMonth);
  const id = partnerId === '' ? null : Number(partnerId);
  const month = usePartnerStatementMonth(id, asked);
  const days = useOpenDays();
  const [active, setActive] = useState<string | null>(null);

  const data = month.data;
  // Excel: every day of the month on screen, for the partner on screen (behind the partner code).
  const exporter = useTableExcel('partner-statement', (sheet) => ({
    kind: 'partner_statement_month',
    partner_id: id ?? 0,
    month: data?.month ?? asked,
    language: sheet,
    paper: null,
  }));
  const first = data?.first_month ?? null;
  const today = day.data?.today ?? null;
  const shown = data?.month ?? null;
  const move = (next: string) => {
    days.closeAll();
    setActive(null);
    // The current month is the address with no month, so a reload on it is always the latest one.
    setAsked(today !== null && next === monthOf(today) ? null : next);
  };
  const stepKeys = shown !== null && today !== null;
  useShortcut(
    SHORTCUTS.pastEarlier,
    () => {
      if (shown !== null) move(windowMonth(addMonths(shown, -1), today ?? '', first));
    },
    { enabled: stepKeys },
  );
  useShortcut(
    SHORTCUTS.pastLater,
    () => {
      if (shown !== null) move(windowMonth(addMonths(shown, 1), today ?? '', first));
    },
    { enabled: stepKeys },
  );
  // A month the address asked for that has nothing before it opens the first month with data instead.
  useEffect(() => {
    if (asked !== null && first !== null && asked < first) setAsked(first);
  }, [asked, first, setAsked]);

  let body;
  if (id === null) {
    body = (
      <EmptyState
        icon={ScrollText}
        title={<Localized id="partners-statement-pick" />}
        help={<Localized id="partners-statement-pick-help" />}
      />
    );
  } else if (month.isError && data === undefined) {
    body = (
      <ErrorState
        title={<Localized id="partners-statement-failed" />}
        error={uiErrorOf(month.error)}
        onRetry={() => {
          void month.refetch();
        }}
      />
    );
  } else if (data === undefined) {
    body = <LoadingState rows={6} />;
  } else {
    body = (
      <div className="flex flex-col gap-3">
        <StatementMonthDays
          data={data}
          busy={month.isFetching}
          open={days.open}
          onToggle={days.toggle}
          active={active}
          onActive={setActive}
        />
        {shown === null || today === null ? null : (
          <div className="flex flex-col gap-3">
            <GoToMonth today={today} first={first} onGo={move} />
            <MonthNav month={shown} today={today} first={first} onMove={move} />
          </div>
        )}
      </div>
    );
  }

  if (view === 'preview' && id !== null) {
    // The print is the month on screen of the partner on screen, every day of it.
    const printInput = (printLanguage: 'ur' | 'en', paper: 'a4' | 'a5' | 'mm80') => ({
      partner_id: id,
      month: shown ?? asked,
      language: printLanguage,
      paper,
    });
    return (
      <ListPage titleId="partners-statement-title" icon={ScrollText}>
        <TablePrintPreview
          exporter={exporter}
          scope={['partner_month', id, shown ?? asked]}
          read={(printLanguage, paper) =>
            partnersStatementMonthPrintData(printInput(printLanguage, paper))
          }
          send={(to, printLanguage, paper) =>
            partnersStatementMonthPrint({
              statement: printInput(printLanguage, paper),
              to,
              copies: null,
            })
          }
          onBack={() => {
            setView('table');
          }}
        />
      </ListPage>
    );
  }

  return (
    <ListPage
      titleId="partners-statement-title"
      icon={ScrollText}
      help={
        <Localized id="partners-statement-help">
          <p />
        </Localized>
      }
      actions={
        <>
          <Button
            shortcut={SHORTCUTS.back}
            onClick={() => {
              void navigate({ to: '/partners' });
            }}
          >
            <Localized id="common-back" />
          </Button>
          <Button
            shortcut={SHORTCUTS.print}
            disabled={data === undefined}
            onClick={() => {
              setView('preview');
            }}
          >
            <Printer aria-hidden="true" className="size-icon-md" />
            <Localized id="report-preview" />
          </Button>
          <ExcelExportButton
            exporter={exporter}
            language={language}
            disabled={data === undefined || id === null}
          />
        </>
      }
      filters={
        <FilterCard {...formNavigationProps}>
          <FilterBar screen="partners.statement">
            {partners.isError ? (
              <FormSpan>
                <ErrorState
                  title={<Localized id="cashbook-partners-failed" />}
                  error={uiErrorOf(partners.error)}
                  onRetry={() => {
                    void partners.refetch();
                  }}
                />
              </FormSpan>
            ) : (
              <Field label={<Localized id="cashbook-field-partner" />}>
                {(props) => (
                  <Select
                    {...props}
                    readOnly={props.readOnly === true || partners.isPending}
                    value={partnerId === '' ? undefined : partnerId}
                    onChange={(next) => {
                      setPartnerId(next);
                      days.closeAll();
                      setActive(null);
                    }}
                    options={(partners.data ?? []).map((partner) => ({
                      value: String(partner.id),
                      label: displayName(partner, language),
                    }))}
                  />
                )}
              </Field>
            )}
          </FilterBar>
        </FilterCard>
      }
    >
      <ExcelExportNotices exporter={exporter} />
      {body}
    </ListPage>
  );
}

/** The screen behind the partner code (plan 44, PRD FR-17.11). */
export function StatementPage() {
  return (
    <PartnerGate>
      <StatementPageScreen />
    </PartnerGate>
  );
}
