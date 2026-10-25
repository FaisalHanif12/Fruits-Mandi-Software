// The gardens tile of the dashboard (PRD 9.1, FR-12.37): the running balance of all the open gardens
// and how many are open. It asks the gardens module for itself (`gardens_summary`), so a garden list
// that cannot be read says so in words on its own tile and the rest of the dashboard is unaffected,
// like the backup tile. The balance is the backend's figure: it starts below zero (what is paid for
// the rent and the costs) and rises as the fruit is sold; the sign is written in the figure, never
// colour alone (NFR-USE-12). The tile opens the list of gardens, where a new one is made (PRD 7.46).

import { Localized } from '@fluent/react';
import { Num, Tile } from '../../../shared/ui';
import { useGardensSummary } from '../../gardens';
import { useOpenDestination } from '../destinations';

export function GardenTile() {
  const open = useOpenDestination();
  const summary = useGardensSummary();
  const data = summary.data;
  const words = data === undefined;
  return (
    <Tile
      words={words}
      label={<Localized id="dashboard-tile-gardens" />}
      note={
        data === undefined ? undefined : data.open_gardens === 0 ? (
          <Localized id="dashboard-tile-gardens-none" />
        ) : (
          <Localized
            id="dashboard-tile-gardens-note"
            vars={{ open: data.open_gardens }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        )
      }
      onOpen={() => {
        open({ kind: 'gardens' });
      }}
    >
      {summary.isPending ? (
        <Localized id="dashboard-tile-checking" />
      ) : data === undefined ? (
        <Localized id="dashboard-tile-unknown" />
      ) : (
        <Num inherit paisa={data.balance_total_paisa} />
      )}
    </Tile>
  );
}
