// The last backup and the books check as dashboard tiles (PRD 9.1, NFR-BAK-10, NFR-OBS-5). Each asks
// the backend for itself, so one that cannot be read says so in words on its own tile and the rest of
// the dashboard is unaffected. Both open the screen where the matter is handled. The words and the
// state of the backup are the backup module's (`statusBarWords`); the books check is the result of
// the last run, with its time. Colour always has a word with it (NFR-USE-12).

import { Localized } from '@fluent/react';
import { statusBarWords, useBackupStatus } from '../../backup';
import { utcToLocalDisplay } from '../../../shared/format';
import { Code, Num, Tile, type TileTone } from '../../../shared/ui';
import { useLastHealth } from '../../settings';
import { useOpenDestination } from '../destinations';

export function BackupTile() {
  const open = useOpenDestination();
  const status = useBackupStatus();
  const words = status.data === undefined ? null : statusBarWords(status.data);
  const tone: TileTone =
    words === null
      ? 'neutral'
      : words.tone === 'good'
        ? 'good'
        : words.tone === 'bad'
          ? 'bad'
          : 'neutral';
  return (
    <Tile
      words
      tone={tone}
      label={<Localized id="dashboard-tile-backup" />}
      onOpen={() => {
        open({ kind: 'backup' });
      }}
    >
      {status.isPending ? (
        <Localized id="dashboard-tile-checking" />
      ) : words === null ? (
        <Localized id="dashboard-tile-unknown" />
      ) : (
        <Localized id={words.messageId} vars={words.vars} elems={{ num: <Code /> }}>
          <span />
        </Localized>
      )}
    </Tile>
  );
}

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
