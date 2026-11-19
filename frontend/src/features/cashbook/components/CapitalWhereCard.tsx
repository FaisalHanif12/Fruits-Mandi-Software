// "Where the capital is" (سرمایہ کہاں ہے), a card on the روکڑ page (PRD FR-16.14, plan 78 E, Umar 17-11-2026). It
// answers two questions with the books as they are now: where the partners' money is (cash in hand, what the
// buyers owe, advances with sellers, money lent ... and minus what the shop owes), and whose it is (each
// partner's کل رقم, the opening capital, the advances still open). The two totals are always equal, because the
// trial balance balances; both are the backend's. The card is a folded section, shut at first, so the list of
// cash rows stays on top; it reads nothing until it is opened. It is behind the partner code (FR-17.11): while
// the section is locked the answer is `ERR_PARTNER_LOCKED` and the box asks for the code in its place.
//
// Nothing is worked out here: every line and both totals come from the backend. The four main lines of the left side
// (cash, what the buyers owe, advances with sellers, what the shop owes sellers) always show, also at 0; every other
// line shows only when it is not 0 (team-lead, 17-11-2026). On the right the partners' totals and the opening capital
// always show, and any other line only when it is not 0.

import { Localized, useLocalization } from '@fluent/react';
import { uiErrorOf } from '../../../shared/api';
import { isZeroPaisa } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { ErrorState, FoldSection, LoadingState, Table, type TableColumn } from '../../../shared/ui';
import { useCapitalWhere } from '../hooks';
import { kindId } from '../list-columns';
import type {
  CapitalWhereKind,
  CapitalWhereLine,
  CapitalWhoseKind,
  CapitalWhoseLine,
} from '../types';
import { SignedNum } from './SignedNum';

/** The lines of the left side that always show, also at 0 (PRD FR-16.14). */
const MAIN_WHERE: ReadonlySet<CapitalWhereKind> = new Set([
  'cash',
  'buyers_owe',
  'seller_advances',
  'sellers_payable',
]);
/** The lines of the right side that always show. */
const MAIN_WHOSE: ReadonlySet<CapitalWhoseKind> = new Set(['partner_total', 'opening_capital']);

/** The total row's first cell is the same word as every other total of the app. */
const TOTAL_LABEL = (
  <Localized id="glossary-meezan">
    <span />
  </Localized>
);

function CapitalWhereBody() {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const read = useCapitalWhere(true);
  if (read.isError) {
    return (
      <ErrorState
        title={<Localized id="cashbook-capital-failed" />}
        error={uiErrorOf(read.error)}
        onRetry={() => {
          void read.refetch();
        }}
      />
    );
  }
  const data = read.data;
  if (data === undefined) return <LoadingState rows={4} />;

  const whereColumns: readonly TableColumn<CapitalWhereLine>[] = [
    {
      id: 'what',
      header: <Localized id="cashbook-capital-col-what" />,
      cell: (line) => <Localized id={`cashbook-capital-where-${kindId(line.kind)}`} />,
      total: TOTAL_LABEL,
    },
    {
      id: 'amount',
      header: <Localized id="cashbook-capital-col-amount" />,
      kind: 'number',
      size: 'money',
      cell: (line) => <SignedNum paisa={line.amount_paisa} />,
      total: <SignedNum paisa={data.where_total_paisa} />,
    },
  ];
  const whoseColumns: readonly TableColumn<CapitalWhoseLine>[] = [
    {
      id: 'what',
      header: <Localized id="cashbook-capital-col-what" />,
      cell: (line) =>
        line.partner === null ? (
          <Localized id={`cashbook-capital-whose-${kindId(line.kind)}`} />
        ) : (
          <Localized
            id="cashbook-capital-whose-partner-total"
            vars={{ partner: displayName(line.partner, language) }}
          />
        ),
      total: TOTAL_LABEL,
    },
    {
      id: 'amount',
      header: <Localized id="cashbook-capital-col-amount" />,
      kind: 'number',
      size: 'money',
      cell: (line) => <SignedNum paisa={line.amount_paisa} />,
      total: <SignedNum paisa={data.whose_total_paisa} />,
    },
  ];

  return (
    <div className="flex flex-col gap-3">
      <Localized id="cashbook-capital-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
        <section className="flex min-w-0 flex-col gap-2">
          <Localized id="cashbook-capital-where-title">
            <h3 className="type-label text-ink" />
          </Localized>
          <Table
            label={l10n.getString('cashbook-capital-where-title')}
            columns={whereColumns}
            rows={data.where_lines.filter(
              (line) => MAIN_WHERE.has(line.kind) || !isZeroPaisa(line.amount_paisa),
            )}
            rowKey={(line) => line.kind}
          />
        </section>
        <section className="flex min-w-0 flex-col gap-2">
          <Localized id="cashbook-capital-whose-title">
            <h3 className="type-label text-ink" />
          </Localized>
          <Table
            label={l10n.getString('cashbook-capital-whose-title')}
            columns={whoseColumns}
            rows={data.whose_lines.filter(
              (line) => MAIN_WHOSE.has(line.kind) || !isZeroPaisa(line.amount_paisa),
            )}
            rowKey={(line) => `${line.kind}-${String(line.partner?.id ?? 0)}`}
          />
        </section>
      </div>
    </div>
  );
}

/** The card on the روکڑ page: shut at first, and nothing is read until it is opened. */
export function CapitalWhereCard() {
  return (
    <FoldSection title={<Localized id="cashbook-capital-title" />}>
      <CapitalWhereBody />
    </FoldSection>
  );
}
