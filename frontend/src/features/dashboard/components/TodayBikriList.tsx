// Today's بکری on the dashboard (PRD 9.3): number, truck, seller, خام بکری and صافی بکری, with a
// word for a cancelled one. The number opens the بکری; the button under the list opens the whole
// day's list. The backend sends at most 100 rows and the count of all of them; the list says how
// many it shows (PRD FR-19.11). The figures are the backend's; a net below zero shows its minus sign
// and the due colour. With none today it says so and offers a new بکری (PRD FR-7.46).

import { Localized, useLocalization } from '@fluent/react';
import { Truck } from 'lucide-react';
import type { DashboardBikri } from '../../../bindings/DashboardBikri';
import { isNegativePaisa } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useRovingKeys } from '../../../shared/keyboard';
import {
  Button,
  Code,
  EmptyState,
  Num,
  ScriptText,
  Table,
  type TableColumn,
} from '../../../shared/ui';
import { useOpenDestination } from '../destinations';

interface TodayBikriListProps {
  rows: readonly DashboardBikri[];
  /** How many بکری there are today in all. */
  count: number;
}

export function TodayBikriList({ rows, count }: TodayBikriListProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const open = useOpenDestination();
  const keys = useRovingKeys();

  if (rows.length === 0) {
    return (
      <EmptyState
        icon={Truck}
        title={<Localized id="dashboard-bikri-empty-title" />}
        help={
          <Localized
            id="dashboard-bikri-empty-help"
            vars={{ key: SHORTCUTS.newBikri.label }}
            elems={{ key: <Code /> }}
          >
            <span />
          </Localized>
        }
        action={
          <Button
            variant="primary"
            shortcut={SHORTCUTS.newBikri}
            onClick={() => {
              open({ kind: 'bikri-book' });
            }}
          >
            <Localized id="dashboard-new-bikri" />
          </Button>
        }
      />
    );
  }

  const columns: TableColumn<DashboardBikri>[] = [
    {
      id: 'number',
      header: <Localized id="dashboard-bikri-col-number" />,
      cell: (row) => (
        <button
          type="button"
          data-nav-item
          onClick={() => {
            open({ kind: 'bikri-book', id: row.bikri_id });
          }}
          className="min-h-target rounded-1 text-primary underline"
        >
          <Localized
            id="dashboard-bikri-number"
            vars={{ number: row.number }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        </button>
      ),
    },
    {
      id: 'truck',
      header: <Localized id="dashboard-bikri-col-truck" />,
      cell: (row) => (row.truck_no === null ? null : <Code>{row.truck_no}</Code>),
    },
    {
      id: 'seller',
      header: <Localized id="dashboard-bikri-col-seller" />,
      cell: (row) => {
        if (row.seller === null) return null;
        const name = displayName({ name_ur: row.seller.ur, name_en: row.seller.en }, language);
        return row.seller_count > 1 ? (
          <Localized
            id="dashboard-bikri-sellers"
            vars={{ name, more: row.seller_count - 1 }}
            elems={{ name: <ScriptText />, num: <Num /> }}
          >
            <span />
          </Localized>
        ) : (
          <ScriptText>{name}</ScriptText>
        );
      },
    },
    {
      id: 'gross',
      header: <Localized id="dashboard-bikri-col-gross" />,
      kind: 'number',
      cell: (row) => <Num paisa={row.gross_paisa} />,
    },
    {
      id: 'net',
      header: <Localized id="dashboard-bikri-col-net" />,
      kind: 'number',
      cell: (row) => (
        <span className={isNegativePaisa(row.net_paisa) ? 'text-due' : undefined}>
          <Num paisa={row.net_paisa} />
        </span>
      ),
    },
    {
      id: 'state',
      header: <Localized id="dashboard-bikri-col-state" />,
      cell: (row) => (row.cancelled ? <Localized id="dashboard-bikri-cancelled" /> : null),
    },
  ];

  return (
    <div className="flex flex-col gap-3" onKeyDown={keys}>
      <Table
        label={l10n.getString('dashboard-bikri-title')}
        columns={columns}
        rows={rows}
        rowKey={(row) => String(row.bikri_id)}
      />
      <div className="flex flex-wrap items-center justify-between gap-3">
        <Localized
          id="dashboard-bikri-showing"
          vars={{ shown: rows.length, count }}
          elems={{ num: <Num /> }}
        >
          <p className="type-body text-ink-muted" />
        </Localized>
        <Button
          onClick={() => {
            open({ kind: 'bikri-today' });
          }}
        >
          <Localized id="dashboard-bikri-all" />
        </Button>
      </div>
    </div>
  );
}
