// What the party card and the ledger heading say about a person in the dead amount (PRD FR-7.38,
// FR-10.10): "ڈوبی رقم" with the date of the move and the reason, and the note if there is one. The
// words carry the meaning, the amber look only helps (NFR-USE-12). The "make active again" button is
// the caller's, because only some places can open the dialog.

import { Localized, useLocalization } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import type { DeadInfo } from '../../bindings/DeadInfo';
import { isoToDisplayDate } from '../format';
import { Num } from './Num';
import { ScriptText } from './ScriptText';

export function DeadInfoLine({ info }: { info: DeadInfo }) {
  const { l10n } = useLocalization();
  return (
    <div
      data-dead-info
      className="flex flex-wrap items-baseline gap-x-3 gap-y-1 rounded-1 border border-warn-border bg-warn-soft px-3 type-body text-warn"
    >
      <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0 self-center" />
      <Localized
        id="dead-info"
        vars={{
          date: isoToDisplayDate(info.entry_date),
          reason: l10n.getString(`dead-reason-${info.reason}`),
        }}
        elems={{ num: <Num /> }}
      >
        <span />
      </Localized>
      {info.note === '' ? null : (
        <Localized id="dead-info-note" vars={{ note: info.note }} elems={{ note: <ScriptText /> }}>
          <span />
        </Localized>
      )}
    </div>
  );
}
