// The books check as a dashboard tile (PRD 9.1, NFR-OBS-5). It asks the backend for itself, so when it cannot
// be read it says so in words on its own tile and the rest of the dashboard is unaffected. It opens the
// screen where the matter is handled. It is the result of the last run, with its time. Colour always has a
// word with it (NFR-USE-12). The last backup is no longer a tile (plan 76 point 10): its state is in the
// status bar and the attention list (NFR-BAK-10).

import { Localized } from '@fluent/react';
import { ShieldCheck } from 'lucide-react';
import { utcToLocalDisplay } from '../../../shared/format';
import { Num, Tile, type TileTone } from '../../../shared/ui';
import { useLastHealth } from '../../settings';
import { useOpenDestination } from '../destinations';

export function HealthTile() {
  const open = useOpenDestination();
  const health = useLastHealth();
  const result = health.data ?? null;
  const tone: TileTone = result === null ? 'neutral' : result.ok ? 'good' : 'bad';
  return (
    <Tile
      words
      tone={tone}
      label={<Localized id="dashboard-tile-health" />}
      icon={ShieldCheck}
      note={
        result === null ? undefined : (
          <Localized
            id="dashboard-health-when"
            vars={{ when: utcToLocalDisplay(result.checked_at) }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        )
      }
      onOpen={() => {
        open({ kind: 'settings-day' });
      }}
    >
      {health.isPending ? (
        <Localized id="dashboard-tile-checking" />
      ) : health.isError ? (
        <Localized id="dashboard-tile-unknown" />
      ) : result === null ? (
        <Localized id="dashboard-health-never" />
      ) : (
        <Localized id={result.ok ? 'dashboard-health-ok' : 'dashboard-health-bad'} />
      )}
    </Tile>
  );
}
