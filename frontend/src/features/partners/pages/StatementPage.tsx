// The partner statement (route /partners/statement; PRD FR-17.9, FR-17.10, FR-19.11, R31): one dated
// line for every amount that reaches a partner (his share of a sale's income, his share of a shop
// expense, capital, money taken and returned, trips), with where it came from, the reason, the
// signed amount and his running کل رقم. A note line (month close, trip goods written off, a trip
// closed) has no amount and leaves the running total the same. Every line is made by the backend
// from posted records; the words come from Fluent here (statement-text.ts).
//
// - The range is two dates and the partner is a drop-down; all are text in this screen's state, so
//   a language switch loses nothing (F39).
// - The list loads page by page; the totals above it are the backend's over the whole range, never
//   a sum of the rows on screen (FR-19.11, F34).
// - Four states: loading, an error with a retry, nothing in the range (a helpful message), the
//   lines. Without a partner it says to choose one.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { ScrollText } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, formNavigationProps, useEscape } from '../../../shared/keyboard';
import { Remembered, type MemoryProps } from '../../../shared/screen-state';
import {
  Button,
  EmptyState,
  ErrorState,
  Fact,
  Field,
  FilterBar,
  FormSpan,
  ListPage,
  Num,
  PeriodBox,
  Select,
} from '../../../shared/ui';
import { SignedNum } from '../../cashbook';
import { PagedList } from '../../../shared/grid';
import { usePeriod } from '../../../shared/hooks';
import { usePartnerList } from '../hooks';
import { usePartnerStatement } from '../money-hooks';
import { STATEMENT_SCREEN, statementChoicesFrom, statementChoicesText } from '../remember';
import { parseStatementSearch } from '../screen-type';
import { statementColumns, type StatementRow } from '../statement-columns';

/** Text or nothing: an empty date is sent as no limit. */
function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

/** The statement starts as the Owner left it: the partner and the days (PRD FR-7.45). */
export function StatementPage() {
  return (
    <Remembered screen={STATEMENT_SCREEN}>{(memory) => <StatementBody {...memory} />}</Remembered>
  );
}

/** The month of a statement line, `YYYY-MM`: the statement is in date order. */
const LINE_MONTH_OF = (row: StatementRow) => row.line.entry_date.slice(0, 7);
/** The backend names the subtotal of a month like the field of its totals. */
const LINE_MONTH_COLUMNS = { amount_paisa: 'amount', running_paisa: 'running' } as const;

function StatementBody({ saved, remember }: MemoryProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const navigate = useNavigate();
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  const search = parseStatementSearch(rawSearch);
  const partners = usePartnerList();
  // An address with a partner or days in it (the dashboard's partner card) wins over what was kept.
  const fromAddress =
    search.partner !== undefined || search.from !== undefined || search.to !== undefined;
  // What the screen starts with is read once, when it opens.
  const [start] = useState(() =>
    fromAddress
      ? {
          partnerId: search.partner === undefined ? '' : String(search.partner),
          from: search.from ?? '',
          to: search.to ?? '',
        }
      : statementChoicesFrom(saved),
  );
  const [partnerId, setPartnerId] = useState(start.partnerId);
  const day = useDayStatus();
  // The statement opens on this year, and "from the first date" is one choice of the period box. Days
  // in the address (the dashboard's partner card) win over what was kept, once, when it opens.
  const period = usePeriod('partners.statement', 'year', day.data?.today);
  const { change: changePeriod } = period;
  const applied = useRef(false);
  useEffect(() => {
    if (applied.current) return;
    applied.current = true;
    if (start.from !== '' || start.to !== '') {
      changePeriod({ kind: 'custom', from: start.from, to: start.to });
    }
  }, [changePeriod, start.from, start.to]);
  useEffect(() => {
    remember(statementChoicesText({ partnerId, from: '', to: '' }));
  }, [partnerId, remember]);
  useEscape('leave', () => {
    void navigate({ to: '/partners' });
  });

  const id = partnerId === '' ? null : Number(partnerId);
  const range = useMemo(
    () => ({ from: textOrNull(period.range.from), to: textOrNull(period.range.to) }),
    [period.range.from, period.range.to],
  );
  const statement = usePartnerStatement(id, range, true, period.ready);
  const first = statement.data?.pages[0];
  const lines = useMemo(
    () =>
      statement.data?.pages
        .flatMap((page) => page.lines)
        .map((line, index): StatementRow => ({ id: String(index), line })),
    [statement.data],
  );
  const columns = useMemo(
    () => statementColumns(grouping, (messageId) => l10n.getBundle(messageId) !== null),
    [grouping, l10n],
  );

  return (
    <ListPage
      titleId="partners-statement-title"
      help={
        <Localized id="partners-statement-help">
          <p />
        </Localized>
      }
      actions={
        <Button
          shortcut={SHORTCUTS.back}
          onClick={() => {
            void navigate({ to: '/partners' });
          }}
        >
          <Localized id="common-back" />
        </Button>
      }
      filters={
        <div {...formNavigationProps}>
          {/* The three filters are one row: the partner, and the period with "from the first date" among
              its choices (plan 38 R4). */}
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
                    onChange={setPartnerId}
                    options={(partners.data ?? []).map((partner) => ({
                      value: String(partner.id),
                      label: displayName(partner, language),
                    }))}
                  />
                )}
              </Field>
            )}
            {period.ready ? <PeriodBox period={period} /> : null}
          </FilterBar>
        </div>
      }
    >
      {first === undefined ? null : (
        <dl className="grid shrink-0 grid-cols-2 gap-4 normal:grid-cols-4">
          <Fact label={<Localized id="partners-statement-opening" />}>
            <SignedNum paisa={first.opening_total_paisa} />
          </Fact>
          <Fact label={<Localized id="partners-statement-plus" />}>
            <Num paisa={first.totals.plus_paisa} />
          </Fact>
          <Fact label={<Localized id="partners-statement-minus" />}>
            <Num paisa={first.totals.minus_paisa} />
          </Fact>
          <Fact label={<Localized id="partners-statement-closing" />}>
            <SignedNum paisa={first.closing_total_paisa} />
          </Fact>
        </dl>
      )}
      {id === null ? (
        <EmptyState
          icon={ScrollText}
          title={<Localized id="partners-statement-pick" />}
          help={<Localized id="partners-statement-pick-help" />}
        />
      ) : (
        <PagedList
          columns={columns}
          rows={lines}
          getRowId={(row) => row.id}
          labelId="partners-statement-label"
          failedId="partners-statement-failed"
          error={statement.isError ? uiErrorOf(statement.error) : null}
          onRetry={() => {
            void statement.refetch();
          }}
          pager={statement.pager}
          sort={{ value: statement.sort, onChange: statement.setSort }}
          months={
            first === undefined
              ? undefined
              : { months: first.months, monthOf: LINE_MONTH_OF, columnOf: LINE_MONTH_COLUMNS }
          }
          empty={
            <EmptyState
              icon={ScrollText}
              title={<Localized id="partners-statement-empty" />}
              help={<Localized id="partners-statement-empty-help" />}
            />
          }
        />
      )}
    </ListPage>
  );
}
