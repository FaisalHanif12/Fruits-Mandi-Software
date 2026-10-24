// The oldest buyer dues on the dashboard (PRD 9.3): the ten buyers whose oldest unpaid bill is the
// oldest, with his city, what he owes and how many days old the bill is. The buyer's name opens his
// ledger (a one-time person inside a پختہ کھاتہ account, his own lines). The figures and the days
// are the backend's. With nothing owed it says so (PRD FR-7.46).

import { Localized, useLocalization } from '@fluent/react';
import { WalletCards } from 'lucide-react';
import type { DashboardDue } from '../../../bindings/DashboardDue';
import { displayName, useLanguage } from '../../../shared/i18n';
import { useRovingKeys } from '../../../shared/keyboard';
import {
  Button,
  Code,
  EmptyState,
  Num,
  ScriptText,
  Table,
  type TableColumn,
} from '../../../shared/ui';
import { nameWithTag } from '../../reports';
import { ledgerOf, useOpenDestination } from '../destinations';

export function OldestDues({ rows }: { rows: readonly DashboardDue[] }) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const open = useOpenDestination();
  const keys = useRovingKeys();

  if (rows.length === 0) {
    return (
      <EmptyState
        icon={WalletCards}
        title={<Localized id="dashboard-dues-empty-title" />}
        help={<Localized id="dashboard-dues-empty-help" />}
      />
    );
  }

  const columns: TableColumn<DashboardDue>[] = [
    {
      id: 'party',
      header: <Localized id="dashboard-dues-col-party" />,
      cell: (row) => (
        <button
          type="button"
          data-nav-item
          onClick={() => {
            open(
              ledgerOf(row.party.account_id, row.party.person_key, {
                name_ur: row.party.name_ur,
                name_en: row.party.name_en,
              }),
            );
          }}
          className="flex min-h-target items-center gap-2 rounded-1 text-start text-primary underline"
        >
          <Code>{row.party.short_code ?? row.party.code}</Code>
          <ScriptText>{nameWithTag(row.party, row.party.tag, language)}</ScriptText>
        </button>
      ),
    },
    {
      id: 'city',
      header: <Localized id="dashboard-dues-col-city" />,
      cell: (row) =>
        row.city === null ? null : (
          <ScriptText>
            {displayName({ name_ur: row.city.ur, name_en: row.city.en }, language)}
          </ScriptText>
        ),
    },
    {
      id: 'balance',
      header: <Localized id="dashboard-dues-col-balance" />,
      kind: 'number',
      cell: (row) => <Num paisa={row.balance_paisa} />,
    },
    {
      id: 'days',
      header: <Localized id="dashboard-dues-col-days" />,
      kind: 'number',
      cell: (row) => (row.days === null ? null : <Num>{row.days}</Num>),
    },
  ];

  return (
    <div className="flex flex-col gap-3" onKeyDown={keys}>
      <Table
        label={l10n.getString('dashboard-dues-title')}
        columns={columns}
        rows={rows}
        rowKey={(row) => `${String(row.party.account_id)}|${row.party.person_key ?? ''}`}
      />
      <div>
        <Button
          onClick={() => {
            open({ kind: 'report', id: 'R20' });
          }}
        >
          <Localized id="dashboard-dues-all" />
        </Button>
      </div>
    </div>
  );
}
