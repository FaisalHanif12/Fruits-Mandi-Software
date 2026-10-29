// The open gardens on the dashboard (PRD 9.1, FR-12.37): each one with its running balance and the rent
// still to pay, from the same call as the tiles. A garden's name opens its screen. The balance
// starts below zero (what is paid for the rent and the costs) and rises as the fruit is sold; its sign
// is written in the figure. The figures are the backend's (CLAUDE.md rule 1). With no open garden the
// section is not drawn: the gardens tile says so (PRD FR-7.46).

import { Localized, useLocalization } from '@fluent/react';
import type { DashboardGarden } from '../../../bindings/DashboardGarden';
import { displayName, useLanguage } from '../../../shared/i18n';
import { useRovingKeys } from '../../../shared/keyboard';
import { Button, Code, Num, ScriptText, Table, type TableColumn } from '../../../shared/ui';
import { useOpenDestination } from '../destinations';

export function OpenGardens({ rows }: { rows: readonly DashboardGarden[] }) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const open = useOpenDestination();
  const keys = useRovingKeys();

  const columns: TableColumn<DashboardGarden>[] = [
    {
      id: 'garden',
      header: <Localized id="dashboard-gardens-col-garden" />,
      cell: (row) => (
        <button
          type="button"
          data-nav-item
          onClick={() => {
            open({ kind: 'garden', id: row.id });
          }}
          className="flex min-h-target items-center gap-2 rounded-1 text-start text-primary underline"
        >
          <Code>{row.code}</Code>
          <ScriptText>{displayName(row, language)}</ScriptText>
        </button>
      ),
    },
    {
      id: 'balance',
      header: <Localized id="dashboard-gardens-col-balance" />,
      kind: 'number',
      cell: (row) => <Num paisa={row.balance_paisa} />,
    },
    {
      id: 'rent_due',
      header: <Localized id="dashboard-gardens-col-rent-due" />,
      kind: 'number',
      cell: (row) => <Num paisa={row.rent_due_paisa} />,
    },
  ];

  return (
    <div className="flex flex-col gap-3" onKeyDown={keys}>
      <Table
        label={l10n.getString('dashboard-gardens-title')}
        columns={columns}
        rows={rows}
        rowKey={(row) => String(row.id)}
      />
      <div>
        <Button
          onClick={() => {
            open({ kind: 'gardens' });
          }}
        >
          <Localized id="dashboard-gardens-all" />
        </Button>
      </div>
    </div>
  );
}
