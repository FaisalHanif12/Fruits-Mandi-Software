// The numbers of the selected lot of a posting open for edit, before and after (PRD FR-12.18, plan
// 16): gross sale, deductions, net sale and the buyers' total. All of them are the backend's: the
// stored ones and the preview's, never worked out here. The list of what was changed and its
// effect on balances, cash and partners is the shared review card (plan 44, PRD FR-7.70); this card
// is only the lot's figures. The same changes are marked in the grids (corner mark and tooltip) so
// colour is never the only signal (NFR-USE-12).

import { Localized, useLocalization } from '@fluent/react';
import type { Paisa } from '../../../shared/format';
import { Card, Num, Table, type TableColumn } from '../../../shared/ui';

/** The four numbers compared. Both the backend's: the loaded entry and the changed one. */
export interface Figures {
  gross: Paisa;
  deductions: Paisa;
  net: Paisa;
  buyer_total: Paisa;
}

interface NumberRow {
  labelId: string;
  before: Paisa | null;
  after: Paisa | null;
}

function numberRow(labelId: string, was: Paisa | undefined, now: Paisa | undefined): NumberRow {
  return { labelId, before: was ?? null, after: now ?? null };
}

/** The four numbers compared, lot by lot. */
function numberRows(before: Figures | null, after: Figures | null): NumberRow[] {
  return [
    numberRow('glossary-kham-bikri', before?.gross, after?.gross),
    numberRow('bikri-summary-deductions', before?.deductions, after?.deductions),
    numberRow('glossary-safi-bikri', before?.net, after?.net),
    numberRow('bikri-summary-buyer-total', before?.buyer_total, after?.buyer_total),
  ];
}

interface EditChangesCardProps {
  /** The numbers of the selected lot as loaded (before) and as changed (after). */
  before: Figures | null;
  after: Figures | null;
}

export function EditChangesCard({ before, after }: EditChangesCardProps) {
  const { l10n } = useLocalization();
  const columns: readonly TableColumn<NumberRow>[] = [
    {
      id: 'what',
      header: l10n.getString('bikri-edit-compare-what'),
      cell: (row) => l10n.getString(row.labelId),
    },
    {
      id: 'before',
      header: l10n.getString('bikri-edit-before'),
      kind: 'number',
      size: 'money',
      cell: (row) => (row.before === null ? '' : <Num paisa={row.before} />),
    },
    {
      id: 'after',
      header: l10n.getString('bikri-edit-after'),
      kind: 'number',
      size: 'money',
      cell: (row) => (row.after === null ? '' : <Num paisa={row.after} />),
    },
  ];

  return (
    <Card compact title={<Localized id="bikri-edit-figures-title" />}>
      <Table
        label={l10n.getString('bikri-edit-figures-title')}
        columns={columns}
        rows={numberRows(before, after)}
        rowKey={(row) => row.labelId}
      />
    </Card>
  );
}
