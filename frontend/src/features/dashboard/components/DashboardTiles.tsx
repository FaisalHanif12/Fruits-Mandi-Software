// The tiles of the dashboard (PRD 9.1, FR-9.1): today's sales and commission, cash in hand, the
// shop's dues, payables and advances, stock, market trips, gardens, money lent and today's expenses. Every
// tile is one button that opens the screen or report behind it. The figures are the backend's, shown
// by `Num` and `formatPaisa`; nothing is added up here (CLAUDE.md rule 1). A figure that is below
// zero or past its date also says so in words, never by colour alone (NFR-USE-12). ↑ ↓ ← → move
// between tiles the way the user sees them (PRD FR-7.13).
//
// The gardens tile (PRD 9.1) asks the gardens module for itself (`GardenTile`), so a garden list that
// cannot be read never hides the others. The backup and the books check tiles are drawn by
// `StatusTiles` and come in as `children` for the same reason.

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import type { DashboardTiles as DashboardTilesData } from '../../../bindings/DashboardTiles';
import { isNegativePaisa, isPositivePaisa } from '../../../shared/format';
import { formNavigationProps } from '../../../shared/keyboard';
import { Num, Tile } from '../../../shared/ui';
import { useOpenDestination } from '../destinations';
import { GardenTile } from './GardenTile';

interface DashboardTilesProps {
  tiles: DashboardTilesData;
  /** More tiles at the end of the grid: the backup and the books check. */
  children?: ReactNode;
}

export function DashboardTiles({ tiles, children }: DashboardTilesProps) {
  const open = useOpenDestination();
  const cashBelowZero = isNegativePaisa(tiles.cash_in_hand_paisa);
  const lentOverdue = isPositivePaisa(tiles.lent_overdue_paisa);
  return (
    <div
      {...formNavigationProps}
      role="group"
      aria-labelledby="dashboard-tiles-title"
      className="grid grid-cols-3 gap-4 normal:grid-cols-4 wide:grid-cols-5"
    >
      <Tile
        label={<Localized id="dashboard-tile-sales" />}
        note={
          <Localized
            id="dashboard-tile-sales-count"
            vars={{ count: tiles.sales_count }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        }
        onOpen={() => {
          open({ kind: 'bikri-today' });
        }}
      >
        <Num inherit paisa={tiles.sales_gross_paisa} />
      </Tile>
      <Tile
        label={<Localized id="dashboard-tile-commission" />}
        onOpen={() => {
          open({ kind: 'analytics' });
        }}
      >
        <Num inherit paisa={tiles.commission_paisa} />
      </Tile>
      <Tile
        label={<Localized id="dashboard-tile-cash" />}
        tone={cashBelowZero ? 'bad' : 'neutral'}
        note={cashBelowZero ? <Localized id="dashboard-tile-cash-below" /> : undefined}
        onOpen={() => {
          open({ kind: 'cashbook' });
        }}
      >
        <Num inherit paisa={tiles.cash_in_hand_paisa} />
      </Tile>
      <Tile
        label={<Localized id="dashboard-tile-dues" />}
        onOpen={() => {
          open({ kind: 'report', id: 'R20' });
        }}
      >
        <Num inherit paisa={tiles.buyers_dues_paisa} />
      </Tile>
      <Tile
        label={<Localized id="dashboard-tile-payable" />}
        onOpen={() => {
          open({ kind: 'report', id: 'R24' });
        }}
      >
        <Num inherit paisa={tiles.sellers_payable_paisa} />
      </Tile>
      <Tile
        label={<Localized id="dashboard-tile-advances" />}
        onOpen={() => {
          open({ kind: 'report', id: 'R26' });
        }}
      >
        <Num inherit paisa={tiles.advances_open_paisa} />
      </Tile>
      <Tile
        label={<Localized id="dashboard-tile-stock" />}
        note={
          <Localized
            id="dashboard-tile-stock-note"
            vars={{ lots: tiles.stock_lots }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        }
        onOpen={() => {
          open({ kind: 'stock' });
        }}
      >
        <Localized
          id="dashboard-figure-nag"
          vars={{ qty: tiles.stock_qty }}
          elems={{ num: <Num /> }}
        >
          <span />
        </Localized>
      </Tile>
      <Tile
        label={<Localized id="dashboard-tile-trips" />}
        note={
          <Localized
            id="dashboard-tile-trips-note"
            vars={{ open: tiles.trips_open }}
            elems={{ num: <Num />, money: <Num paisa={tiles.trips_cost_left_paisa} /> }}
          >
            <span />
          </Localized>
        }
        onOpen={() => {
          open({ kind: 'trips' });
        }}
      >
        <Localized
          id="dashboard-figure-nag"
          vars={{ qty: tiles.trips_qty_left }}
          elems={{ num: <Num inherit />, unit: <span className="type-body text-ink-muted" /> }}
        >
          <span />
        </Localized>
      </Tile>
      <GardenTile />
      <Tile
        label={<Localized id="dashboard-tile-lent" />}
        tone={lentOverdue ? 'warn' : 'neutral'}
        note={
          lentOverdue ? (
            <Localized
              id="dashboard-tile-lent-overdue"
              elems={{ money: <Num paisa={tiles.lent_overdue_paisa} /> }}
            >
              <span />
            </Localized>
          ) : undefined
        }
        onOpen={() => {
          open({ kind: 'report', id: 'R32' });
        }}
      >
        <Num inherit paisa={tiles.lent_open_paisa} />
      </Tile>
      <Tile
        label={<Localized id="dashboard-tile-expenses" />}
        onOpen={() => {
          open({ kind: 'report', id: 'R29' });
        }}
      >
        <Num inherit paisa={tiles.expenses_today_paisa} />
      </Tile>
      {children}
    </div>
  );
}
