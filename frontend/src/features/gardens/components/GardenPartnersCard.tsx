// What each partner has put into the garden and got back so far (PRD FR-12.38, FR-17.10, E2E-40):
// paid from the shop's cash (it came off his total when it was paid), paid from his pocket
// (recorded only), money back (his part of the closing), and the net, with the working behind its
// ⓘ. Before the closing the net is minus what he paid. A net below zero is red and also written
// (NFR-USE-12). Everything is the backend's; nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import { formatBps, isNegativePaisa } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { CalcInfo, Card, Num, Table, type TableColumn } from '../../../shared/ui';
import type { GardenPartnerResult } from '../types';

interface GardenPartnersCardProps {
  results: readonly GardenPartnerResult[];
  /** A closed garden shows the result of the closing; an open one shows what is paid so far. */
  closed: boolean;
}

export function GardenPartnersCard({ results, closed }: GardenPartnersCardProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const columns: readonly TableColumn<GardenPartnerResult>[] = [
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
      id: 'paid-shop',
      header: l10n.getString('gardens-partner-paid-shop'),
      kind: 'number',
      cell: (row) => <Num paisa={row.paid_shop_paisa} />,
    },
    {
      id: 'paid-pocket',
      header: l10n.getString('gardens-partner-paid-pocket'),
      kind: 'number',
      cell: (row) => <Num paisa={row.paid_pocket_paisa} />,
    },
    {
      id: 'back',
      header: l10n.getString('gardens-partner-back'),
      kind: 'number',
      cell: (row) => <Num paisa={row.money_back_paisa} />,
    },
    {
      id: 'net',
      header: l10n.getString(closed ? 'gardens-partner-net-closed' : 'gardens-partner-net-open'),
      kind: 'number',
      cell: (row) => (
        <span
          className={`inline-flex items-center gap-1 ${isNegativePaisa(row.net_paisa) ? 'text-due' : ''}`}
        >
          {closed && isNegativePaisa(row.net_paisa) ? (
            <>
              <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
              <Localized id="gardens-partner-loss">
                <span className="sr-only" />
              </Localized>
            </>
          ) : null}
          <Num paisa={row.net_paisa} />
          <CalcInfo working={row.working} inCell />
        </span>
      ),
    },
  ];
  return (
    <Card compact title={<Localized id="gardens-partner-results-title" />}>
      {results.length === 0 ? (
        <Localized id="gardens-partner-results-empty">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <Table
          label={l10n.getString('gardens-partner-results-title')}
          columns={columns}
          rows={results}
          rowKey={(row) => String(row.partner.id)}
        />
      )}
    </Card>
  );
}
