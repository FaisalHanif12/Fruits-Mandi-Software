// The small "Edited" mark on a list row of an edited entry (plan 44, PRD FR-7.71). The word is
// always written, so colour is never the only signal (NFR-USE-12). Pointing at it shows the time
// and the name of the last edit.

import { Localized, useLocalization } from '@fluent/react';
import { Pencil } from 'lucide-react';
import type { EditStamp } from '../../bindings/EditStamp';
import { stampTipText } from './EditStampLine';

export function EditedMark({ stamp }: { stamp: EditStamp }) {
  const { l10n } = useLocalization();
  return (
    <span
      className="inline-flex items-center gap-1 rounded-1 bg-warn-soft px-2 type-body text-warn"
      title={stampTipText(stamp, l10n)}
      data-edited-mark=""
    >
      <Pencil aria-hidden="true" className="size-icon-sm" />
      <Localized id="edit-mark" />
    </span>
  );
}
