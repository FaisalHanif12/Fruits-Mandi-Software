// What each partner has put into the trip and got back so far (PRD FR-12.27, E2E-34b): money in
// from the shop's cash (it came off his total when it was paid), money in from his pocket
// (recorded only), money back (his parts of the seller credits and of the money of sales in other
// shops), and the net, with the working behind its ⓘ. A net below zero is red and also written
// (NFR-USE-12). Everything is the backend's; nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import { formatBps, isNegativePaisa } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { CalcInfo, Card, FigureCell, Num, Table, type TableColumn } from '../../../shared/ui';
import type { TripPartnerResult } from '../types';

interface TripPartnersCardProps {
  results: readonly TripPartnerResult[];
}

export function TripPartnersCard({ results }: TripPartnersCardProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const columns: readonly TableColumn<TripPartnerResult>[] = [
    {
      id: 'partner',
      header: l10n.getString('glossary-shirakat-dar'),
      cell: (row) => (
        <span className="inline-flex items-baseline gap-2">
          <span lang={language}>{displayName(row.partner, language)}</span>
          <span className="text-ink-muted">
            <Num>
              {l10n.getString('ventures-share-percent', { value: formatBps(row.share_bps) })}
            </Num>
          </span>
        </span>
      ),
    },
    {
      id: 'in-shop',
      header: l10n.getString('trips-partner-in-shop'),
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.money_in_shop_paisa} />,
    },
    {
      id: 'in-pocket',
      header: l10n.getString('trips-partner-in-pocket'),
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.money_in_pocket_paisa} />,
    },
    {
      id: 'back',
      header: l10n.getString('trips-partner-back'),
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.money_back_paisa} />,
    },
    {
      id: 'net',
      header: l10n.getString('trips-partner-net'),
      kind: 'number',
      size: 'money',
      // The figure under the middle of its heading; the loss mark before it, the ⓘ after it (plan 39 R1).
      cell: (row) => (
        <span className={isNegativePaisa(row.net_paisa) ? 'text-due' : ''}>
          <FigureCell
            lead={
              isNegativePaisa(row.net_paisa) ? (
                <>
                  <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
                  <Localized id="trips-partner-loss">
                    <span className="sr-only" />
                  </Localized>
                </>
              ) : undefined
            }
            extra={<CalcInfo working={row.working} inCell />}
          >
            <Num paisa={row.net_paisa} />
          </FigureCell>
        </span>
      ),
    },
  ];
  return (
    <Card compact title={<Localized id="trips-partner-results-title" />}>
      {results.length === 0 ? (
        <Localized id="trips-partner-results-empty">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <Table
          label={l10n.getString('trips-partner-results-title')}
          columns={columns}
          rows={results}
          rowKey={(row) => String(row.partner.id)}
        />
      )}
    </Card>
  );
}
