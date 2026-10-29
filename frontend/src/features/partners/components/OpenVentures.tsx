// The open gardens and trips a partner is in, under his card on the overview (PRD 17.3, FR-12.29,
// plan 17c): each with his %, what he paid so far from the shop's cash and from his own pocket, the
// money he got back so far, and the running balance of a garden or the result of a trip. A garden
// or a trip opens from its name. Every number is the backend's (`open_ventures`) and only shown
// here (CLAUDE.md rule 1). A partner in none of them shows nothing.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { formatBps } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { LinkButton, Num, Table, type TableColumn } from '../../../shared/ui';
import type { PartnerVentureCard } from '../../../bindings/PartnerVentureCard';

interface OpenVenturesProps {
  ventures: readonly PartnerVentureCard[];
  /** His name, for the name of the list read by a screen reader. */
  partnerName: string;
}

export function OpenVentures({ ventures, partnerName }: OpenVenturesProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const language = useLanguage();
  if (ventures.length === 0) return null;

  const open = (venture: PartnerVentureCard) => {
    void navigate(
      venture.kind === 'garden'
        ? { to: '/gardens/$id', params: { id: String(venture.venture_id) } }
        : { to: '/trips/$id', params: { id: String(venture.venture_id) } },
    );
  };
  const columns: readonly TableColumn<PartnerVentureCard>[] = [
    {
      id: 'venture',
      header: l10n.getString('partners-ventures-col-venture'),
      cell: (venture) => {
        const label =
          venture.kind === 'garden'
            ? `${venture.code} ${displayName(venture, language)}`
            : displayName(venture, language);
        return (
          <LinkButton
            onClick={() => {
              open(venture);
            }}
          >
            <span lang={language}>{label}</span>
          </LinkButton>
        );
      },
    },
    {
      id: 'kind',
      header: l10n.getString('partners-ventures-col-kind'),
      cell: (venture) => (
        <Localized id={venture.kind === 'garden' ? 'glossary-bagh' : 'glossary-safar'}>
          <span />
        </Localized>
      ),
    },
    {
      id: 'share',
      header: l10n.getString('glossary-hissa'),
      kind: 'number',
      cell: (venture) => <Num>{`${formatBps(venture.share_bps)}%`}</Num>,
    },
    {
      id: 'paid-shop',
      header: l10n.getString('partners-ventures-col-shop'),
      kind: 'number',
      cell: (venture) => <Num paisa={venture.paid_shop_paisa} />,
    },
    {
      id: 'paid-pocket',
      header: l10n.getString('partners-ventures-col-pocket'),
      kind: 'number',
      cell: (venture) => <Num paisa={venture.paid_pocket_paisa} />,
    },
    {
      id: 'back',
      header: l10n.getString('partners-ventures-col-back'),
      kind: 'number',
      cell: (venture) => <Num paisa={venture.money_back_paisa} />,
    },
    {
      id: 'balance',
      header: l10n.getString('partners-ventures-col-balance'),
      kind: 'number',
      cell: (venture) => <Num paisa={venture.balance_paisa} />,
    },
  ];
  return (
    <div className="flex flex-col gap-2 border-t border-border pt-3">
      <Localized id="partners-ventures-title">
        <h3 className="type-label text-ink-muted" />
      </Localized>
      <Table
        label={l10n.getString('partners-ventures-label', { name: partnerName })}
        columns={columns}
        rows={ventures}
        rowKey={(venture) => `${venture.kind}-${String(venture.venture_id)}`}
      />
    </div>
  );
}
