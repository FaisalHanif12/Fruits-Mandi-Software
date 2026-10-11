// The chart of accounts (route /index/chart; PRD FR-10.1): the tree of heads, controls and
// accounts with live balances, and the head list and the control list. Four states. The data is
// the backend's chart with every balance and its side; nothing is worked out here.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { useMemo, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, type ListColumn } from '../../../shared/grid';
import { useEscape } from '../../../shared/keyboard';
import { displayName } from '../../../shared/i18n';
import { ErrorState, LoadingState, Tabs, sideMessageId } from '../../../shared/ui';
import { useCoaTree } from '../../../shared/accounts';
import type { CoaControl, CoaHead } from '../../../shared/accounts';
import { ChartTree } from '../components/ChartTree';

type ChartRow = CoaHead | CoaControl;

const COLUMNS: readonly ListColumn<ChartRow>[] = [
  { field: 'code', type: 'code', headerId: 'accounts-col-code', value: (row) => row.code },
  {
    field: 'name',
    type: 'name',
    headerId: 'accounts-col-name',
    grow: true,
    value: (row, { language }) => displayName(row, language),
  },
  {
    field: 'balance',
    type: 'money',
    headerId: 'accounts-col-balance',
    value: (row, { money }) => money(row.balance.amount_paisa),
  },
  {
    field: 'side',
    type: 'status',
    headerId: 'accounts-col-side',
    value: (row, { text }) => {
      const id = sideMessageId(row.balance.side);
      return id === null ? '' : text(id);
    },
  },
];

export function ChartPage() {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const tree = useCoaTree();
  const [tab, setTab] = useState('tree');
  useEscape('leave', () => {
    void navigate({ to: '/index' });
  });
  const heads = tree.data;
  const controls = useMemo<CoaControl[]>(
    () => (heads === undefined ? [] : heads.flatMap((head) => head.controls)),
    [heads],
  );

  if (tree.isPending) return <LoadingState />;
  if (tree.isError) {
    return (
      <ErrorState
        title={<Localized id="chart-load-failed" />}
        error={uiErrorOf(tree.error)}
        onRetry={() => {
          void tree.refetch();
        }}
      />
    );
  }
  const list = (rows: readonly ChartRow[], labelId: string) => (
    <ListGrid<ChartRow>
      columns={COLUMNS}
      rows={rows}
      getRowId={(row) => String(row.id)}
      labelId={labelId}
      onOpen={() => undefined}
      heightMode="auto"
    />
  );
  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-col gap-1">
        <Localized id="chart-title">
          <h1 className="type-title" />
        </Localized>
        <Localized id="chart-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      <Tabs
        label={l10n.getString('chart-tabs')}
        value={tab}
        onValueChange={setTab}
        items={[
          {
            id: 'tree',
            label: <Localized id="chart-tab-tree" />,
            content: (
              <div className="h-tree-view min-h-tree">
                <ChartTree
                  heads={tree.data}
                  onOpenAccount={(id) => {
                    void navigate({ to: '/index/accounts/$id', params: { id: String(id) } });
                  }}
                />
              </div>
            ),
          },
          {
            id: 'heads',
            label: <Localized id="chart-tab-heads" />,
            content: list(tree.data, 'chart-tab-heads'),
          },
          {
            id: 'controls',
            label: <Localized id="chart-tab-controls" />,
            content: list(controls, 'chart-tab-controls'),
          },
        ]}
      />
    </div>
  );
}
