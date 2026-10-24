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
import { useEffect, useMemo, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, formNavigationProps, useEscape } from '../../../shared/keyboard';
import { Remembered, type MemoryProps } from '../../../shared/screen-state';
import {
  Button,
  Card,
  DateInput,
  EmptyState,
  ErrorState,
  Fact,
  Field,
  Num,
  Select,
} from '../../../shared/ui';
import { SignedNum } from '../../cashbook';
import { PagedList } from '../../../shared/grid';
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
  const [from, setFrom] = useState(start.from);
  const [to, setTo] = useState(start.to);
  useEffect(() => {
    remember(statementChoicesText({ partnerId, from, to }));
  }, [partnerId, from, to, remember]);
  useEscape('leave', () => {
    void navigate({ to: '/partners' });
  });

  const id = partnerId === '' ? null : Number(partnerId);
  const range = useMemo(() => ({ from: textOrNull(from), to: textOrNull(to) }), [from, to]);
  const statement = usePartnerStatement(id, range);
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
    <div className="flex max-w-page flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="partners-statement-title">
            <h1 className="sr-only normal:not-sr-only normal:type-title" />
          </Localized>
          <Localized id="partners-statement-help">
            <p className="hidden type-body text-ink-muted normal:block" />
          </Localized>
        </div>
        <Button
          shortcut={SHORTCUTS.back}
          onClick={() => {
            void navigate({ to: '/partners' });
          }}
        >
          <Localized id="common-back" />
        </Button>
      </div>
      <Card compact>
        <div
          {...formNavigationProps}
          className="grid grid-cols-1 items-end gap-4 normal:grid-cols-3"
        >
          {partners.isError ? (
            <ErrorState
              title={<Localized id="cashbook-partners-failed" />}
              error={uiErrorOf(partners.error)}
              onRetry={() => {
                void partners.refetch();
              }}
            />
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
          <Field label={<Localized id="cashbook-filter-from" />}>
            {(control) => <DateInput {...control} value={from} onChange={setFrom} />}
          </Field>
          <Field label={<Localized id="cashbook-filter-to" />}>
            {(control) => <DateInput {...control} value={to} onChange={setTo} />}
          </Field>
        </div>
        {first === undefined ? null : (
          <dl className="grid grid-cols-2 gap-4 normal:grid-cols-4">
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
            totalCount={first?.total_count}
            getRowId={(row) => row.id}
            labelId="partners-statement-label"
            failedId="partners-statement-failed"
            error={statement.isError ? uiErrorOf(statement.error) : null}
            onRetry={() => {
              void statement.refetch();
            }}
            hasMore={statement.hasNextPage}
            loadingMore={statement.isFetchingNextPage}
            onLoadMore={() => {
              void statement.fetchNextPage();
            }}
            empty={
              <EmptyState
                icon={ScrollText}
                title={<Localized id="partners-statement-empty" />}
                help={<Localized id="partners-statement-empty-help" />}
              />
            }
          />
        )}
      </Card>
    </div>
  );
}
