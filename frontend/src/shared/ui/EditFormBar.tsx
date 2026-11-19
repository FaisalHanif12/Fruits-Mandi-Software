// The top of the form of an entry open for edit (plan 44, PRD FR-7.70, FR-7.71): the amber band that
// says what is being changed, the stamp of the last edit with its history, and the two buttons, Save
// and Back without changing. The same on every entry of the trip and the garden.

import { Localized } from '@fluent/react';
import { ArrowLeft } from 'lucide-react';
import { useEffect, useRef } from 'react';
import type { EditEntity } from '../../bindings/EditEntity';
import type { EditStamp } from '../../bindings/EditStamp';
import { SHORTCUTS } from '../keyboard';
import { Button } from './Button';
import { EditBand, type EditBandKind } from './EditBand';
import { EditStampLine } from './EditStampLine';

interface EditFormBarProps {
  kind: EditBandKind;
  /** The number as shown ("3", "خ-3"). */
  number: string;
  /** The day of the entry, `YYYY-MM-DD`. */
  date: string;
  reopenedDays?: readonly string[];
  edited: EditStamp | null;
  history: { entity: EditEntity; id: number };
  busy: boolean;
  onSave: () => void;
  onBack: () => void;
}

export function EditFormBar({
  kind,
  number,
  date,
  reopenedDays = [],
  edited,
  history,
  busy,
  onSave,
  onBack,
}: EditFormBarProps) {
  // The page shows the top of the edit when it opens: the form is lower on the page than the band, so
  // the cursor in the first box would otherwise scroll the band out of sight.
  const top = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const element = top.current;
    if (element !== null && typeof element.scrollIntoView === 'function') {
      element.scrollIntoView({ block: 'start' });
    }
  }, []);
  return (
    <div ref={top} className="flex scroll-mt-4 flex-col gap-3">
      <EditBand kind={kind} number={number} date={date} reopenedDays={reopenedDays} />
      {edited === null ? null : <EditStampLine stamp={edited} history={history} />}
      <div className="flex flex-wrap items-center gap-2">
        <Button variant="primary" shortcut={SHORTCUTS.save} busy={busy} onClick={onSave}>
          <Localized id="edit-save" />
        </Button>
        <Button onClick={onBack}>
          <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
          <Localized id="edit-back" />
        </Button>
      </div>
    </div>
  );
}
