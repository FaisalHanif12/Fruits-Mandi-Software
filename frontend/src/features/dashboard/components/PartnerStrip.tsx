// The partner strip (PRD 9.2): one card for each partner with his profit share today, this month and
// this year, and his کل رقم (C22). A card is one button that opens his statement (R31). The figures
// are the backend's (`dashboard_get`); a share or a total below zero says so with its minus sign and
// shows in the due colour, so colour is never the only signal (NFR-USE-12). Names go through
// `displayName` (Code Standards F38). With no partner yet it says so and leads to the partner setup.

import { Localized, useLocalization } from '@fluent/react';
import { Users } from 'lucide-react';
import { useNavigate } from '@tanstack/react-router';
import type { DashboardPartner } from '../../../bindings/DashboardPartner';
import type { Paisa } from '../../../shared/format';
import { isNegativePaisa } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { formNavigationProps } from '../../../shared/keyboard';
import { Button, EmptyState, Num, ScriptText } from '../../../shared/ui';
import { useOpenDestination } from '../destinations';

function Figure({ label, paisa }: { label: string; paisa: Paisa }) {
  return (
    <div className="flex items-baseline justify-between gap-3">
      <dt className="type-body text-ink-muted">{label}</dt>
      <dd className={`value-col type-number ${isNegativePaisa(paisa) ? 'text-due' : 'text-ink'}`}>
        <Num paisa={paisa} />
      </dd>
    </div>
  );
}

function PartnerCard({ partner }: { partner: DashboardPartner }) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const open = useOpenDestination();
  const name = displayName(partner, language);
  return (
    <button
      type="button"
      data-nav-field
      data-partner-id={partner.partner_id}
      aria-label={l10n.getString('dashboard-partner-open', { name })}
      onClick={() => {
        open({ kind: 'report', id: 'R31', partner: partner.partner_id });
      }}
      className="flex min-w-0 flex-col gap-2 rounded-3 border border-border bg-surface p-4 text-start transition-calm hover:border-border-control"
    >
      <span className="type-section text-ink">
        <ScriptText>{name}</ScriptText>
      </span>
      <dl className="flex flex-col gap-1">
        <Figure label={l10n.getString('dashboard-partner-today')} paisa={partner.today_paisa} />
        <Figure label={l10n.getString('dashboard-partner-month')} paisa={partner.month_paisa} />
        <Figure label={l10n.getString('dashboard-partner-year')} paisa={partner.year_paisa} />
        <div className="border-t border-border pt-1">
          <Figure label={l10n.getString('dashboard-partner-total')} paisa={partner.total_paisa} />
        </div>
      </dl>
    </button>
  );
}

export function PartnerStrip({ partners }: { partners: readonly DashboardPartner[] }) {
  const navigate = useNavigate();
  if (partners.length === 0) {
    return (
      <EmptyState
        icon={Users}
        title={<Localized id="dashboard-partners-empty-title" />}
        help={<Localized id="dashboard-partners-empty-help" />}
        action={
          <Button
            onClick={() => {
              void navigate({ to: '/partners/setup' });
            }}
          >
            <Localized id="dashboard-partners-setup" />
          </Button>
        }
      />
    );
  }
  return (
    <div
      {...formNavigationProps}
      role="group"
      aria-labelledby="dashboard-partners-title"
      className="grid grid-cols-3 gap-4 normal:grid-cols-4"
    >
      {partners.map((partner) => (
        <PartnerCard key={partner.partner_id} partner={partner} />
      ))}
    </div>
  );
}
