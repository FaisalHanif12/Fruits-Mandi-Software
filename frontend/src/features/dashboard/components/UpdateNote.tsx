// A note on the dashboard that a newer version of the app is ready (plan 19, NFR-INS-3). The app
// asks once when it starts; this is only a note. Installing stays in Settings, with the Owner's
// agreement and a backup first, never during work. Nothing shows when there is no newer version or
// the check could not reach the network.

import { Localized } from '@fluent/react';
import { Download } from 'lucide-react';
import { Button, Notice, Num } from '../../../shared/ui';
import { useStartupUpdateCheck } from '../../settings';
import { useOpenDestination } from '../destinations';

export function UpdateNote() {
  const update = useStartupUpdateCheck(true);
  const open = useOpenDestination();
  if (update.data?.available !== true) return null;
  const version = update.data.version ?? '';
  return (
    <Notice
      tone="info"
      action={
        <Button
          onClick={() => {
            open({ kind: 'settings-app' });
          }}
        >
          <Download aria-hidden="true" className="size-icon-md" />
          <Localized id="dashboard-update-open" />
        </Button>
      }
    >
      <span data-update-note>
        <Localized id="dashboard-update-note" vars={{ version }} elems={{ num: <Num /> }}>
          <span />
        </Localized>
      </span>
    </Notice>
  );
}
