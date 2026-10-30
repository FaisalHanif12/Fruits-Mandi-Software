// The partners on the overview (PRD 17.3, plan 38): ONE table, one row per partner. A row has his
// name, his کل رقم first and in bold (C22, FR-17.9: everything he has in the firm), his share, his
// profit share and خالص of the month, and the buttons that open his statement and his capital. The
// other figures (today, month and year income, expense, money taken and returned, capital, capital put
// in, and the باغ، سفر اور پھل column) are under his row when "More columns" is ticked, so the table
// never scrolls sideways (NFR-USE-15). Every number comes from the backend (`partners_overview`) and
// is only shown here (CLAUDE.md rule 1). A negative amount shows its minus sign and the due colour, so
// colour is never the only signal (NFR-USE-12).

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Landmark, ScrollText } from 'lucide-react';
import { Fragment, type ReactNode } from 'react';
import { formatBps } from '../../../shared/format';
import { displayName, otherScriptLang, otherScriptName, useLanguage } from '../../../shared/i18n';
import { Button, Card, Num } from '../../../shared/ui';
import { SignedNum } from '../../cashbook';
import type { PartnerOverviewRow } from '../types';
import { OpenVentures } from './OpenVentures';

interface OverviewTableProps {
  rows: readonly PartnerOverviewRow[];
  /** The other figures of every partner are shown under his row. */
  more: boolean;
}

/** A heading of a number column: centred over its figures (NFR-USE-19, plan 39 R1). */
function NumberHead({ labelId }: { labelId: string }) {
  return (
    <th
      scope="col"
      dir="ltr"
      data-align="num"
      className="px-3 py-2 text-center align-middle font-normal"
    >
      <Localized id={labelId}>
        <span dir="auto" className="inline-block" />
      </Localized>
    </th>
  );
}

/** A label and its figure on one line: the label at the start edge, the figure at the right edge. */
function Detail({ labelId, children }: { labelId: string; children: ReactNode }) {
  return (
    <div className="flex min-w-0 items-baseline justify-between gap-3">
      <Localized id={labelId}>
        <dt className="type-body text-ink-muted" />
      </Localized>
      <dd className="value-col type-body">{children}</dd>
    </div>
  );
}

/** The figures of the second group: what the table leaves out until "More columns". */
function MoreFigures({ row }: { row: PartnerOverviewRow }) {
  return (
    <dl className="grid grid-cols-3 gap-x-4 gap-y-3 normal:grid-cols-5 wide:grid-cols-9">
      <Detail labelId="partners-metric-today">
        <Num paisa={row.today_income_paisa} />
      </Detail>
      <Detail labelId="partners-metric-month">
        <Num paisa={row.month_income_paisa} />
      </Detail>
      <Detail labelId="partners-metric-year">
        <Num paisa={row.year_income_paisa} />
      </Detail>
      <Detail labelId="partners-metric-expense">
        <Num paisa={row.month_expense_paisa} />
      </Detail>
      <Detail labelId="partners-metric-taken">
        <Num paisa={row.month_taken_paisa} />
      </Detail>
      <Detail labelId="partners-metric-returned">
        <Num paisa={row.month_returned_paisa} />
      </Detail>
      <Detail labelId="partners-metric-capital">
        <Num paisa={row.capital_balance_paisa} />
      </Detail>
      <Detail labelId="partners-metric-capital-in">
        <SignedNum paisa={row.month_capital_in_paisa} />
      </Detail>
      <Detail labelId="partners-metric-ventures">
        <SignedNum paisa={row.month_ventures_paisa} />
      </Detail>
    </dl>
  );
}

export function OverviewTable({ rows, more }: OverviewTableProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const language = useLanguage();
  return (
    <div className="flex flex-col gap-4">
      <table
        aria-label={l10n.getString('partners-overview-label')}
        className="w-full border-collapse type-body"
      >
        <thead className="bg-surface-sunken text-ink-muted">
          <tr>
            <th scope="col" data-align="text" className="px-3 py-2 text-start font-normal">
              <Localized id="partners-column-partner" />
            </th>
            {/* The کل رقم comes first among the figures: it is the one number he asks for. */}
            <NumberHead labelId="partners-metric-total" />
            <NumberHead labelId="partners-share-now" />
            <NumberHead labelId="partners-metric-profit" />
            <NumberHead labelId="partners-metric-net" />
            <th scope="col" data-align="centre" className="px-3 py-2 text-center font-normal">
              <span className="sr-only">
                <Localized id="partners-open-statement" />
              </span>
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => {
            const name = displayName(row, language);
            return (
              <Fragment key={row.partner_id}>
                <tr
                  data-partner={row.partner_id}
                  className={`${more ? '' : 'border-t border-border'} align-middle`}
                >
                  <th scope="row" className="px-3 py-2 text-start font-normal">
                    <span className="flex flex-wrap items-baseline gap-x-3">
                      <span className="type-section" lang={language}>
                        {name}
                      </span>
                      <span className="text-ink-muted" lang={otherScriptLang(language)}>
                        {otherScriptName(row, language)}
                      </span>
                    </span>
                  </th>
                  <td dir="ltr" data-align="num" className="px-3 py-2 text-center">
                    <SignedNum paisa={row.total_paisa} />
                  </td>
                  <td dir="ltr" data-align="num" className="px-3 py-2 text-center">
                    <Num>{`${formatBps(row.share_bps)}%`}</Num>
                  </td>
                  <td dir="ltr" data-align="num" className="px-3 py-2 text-center">
                    <Num paisa={row.month_profit_paisa} />
                  </td>
                  <td dir="ltr" data-align="num" className="px-3 py-2 text-center">
                    <SignedNum paisa={row.month_net_paisa} />
                  </td>
                  <td data-align="centre" className="px-3 py-2">
                    <div className="flex justify-center gap-2">
                      <Button
                        accessibleName={l10n.getString('partners-open-statement-of', { name })}
                        onClick={() => {
                          void navigate({
                            to: '/partners/statement',
                            search: { partner: row.partner_id },
                          });
                        }}
                      >
                        <ScrollText aria-hidden="true" className="size-icon-md" />
                        <span className="hidden normal:inline">
                          <Localized id="partners-open-statement" />
                        </span>
                      </Button>
                      <Button
                        accessibleName={l10n.getString('partners-open-capital-of', { name })}
                        onClick={() => {
                          void navigate({
                            to: '/partners/capital',
                            search: { partner: row.partner_id },
                          });
                        }}
                      >
                        <Landmark aria-hidden="true" className="size-icon-md" />
                        <span className="hidden normal:inline">
                          <Localized id="partners-open-capital" />
                        </span>
                      </Button>
                    </div>
                  </td>
                </tr>
                {more ? (
                  <tr data-partner-more={row.partner_id} className="border-b border-border">
                    <td colSpan={6} className="px-3 pb-3">
                      <MoreFigures row={row} />
                    </td>
                  </tr>
                ) : null}
              </Fragment>
            );
          })}
        </tbody>
      </table>
      {/* The open gardens and trips of the partners that are in any (PRD 17.3, FR-12.29). */}
      {rows
        .filter((row) => row.open_ventures.length > 0)
        .map((row) => (
          <Card
            compact
            key={row.partner_id}
            title={<span lang={language}>{displayName(row, language)}</span>}
          >
            <OpenVentures ventures={row.open_ventures} partnerName={displayName(row, language)} />
          </Card>
        ))}
    </div>
  );
}
