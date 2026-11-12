// The "+" under an entry grid (lots, sale lines): a small round button (32 px, the smallest target,
// NFR-USE-23, with a 16 px icon) at the end side of the row below the grid, so it is on the left in Urdu
// and on the right in English with no `left` or `right` written (Code Standards F24). It takes no room
// from the grid's own rows and adds no width: it has a row of its own under the grid, and the screen
// keeps only a small token gap (`--space-1`) between the grid and it, so it sits at the grid's end corner.
//
// What it does is the screen's own `onAdd`, which must go through the grid's one new-row path
// (`addRowAndFocus`, the same as `↓` on the last row), so the store and the keyboard rules do not
// change. At a limit (a lot or line count the app allows, NFR-SCL-3) the button is dimmed and the
// words say why, in the row beside it, so a dimmed button never stands alone. The name and the tip are
// Fluent messages of the screen. The button is reached with Tab after the grid, like any button, and by
// the arrows: `↓` on the trash cell of the grid's last row lands on it, and `↑` on it goes back to that
// cell (addLink.ts). `Enter` and `Space` add a row, as on any button.

import { Localized, useLocalization } from '@fluent/react';
import { Plus } from 'lucide-react';
import { useId } from 'react';
import { RUNNING_NUMBER } from '../ui/Num';
import { Button } from '../ui/Button';
import type { AddRowLink } from './addLink';

export interface AddRowLimit {
  /** The Fluent message that says why no row can be added now. `<num>` marks a number in it. */
  messageId: string;
  vars?: Record<string, string | number>;
}

interface AddRowButtonProps {
  /** The Fluent message that names the button and is its tip, for example "New lot". */
  labelId: string;
  /** Adds the row and puts the cursor on its first cell (through `addRowAndFocus`). */
  onAdd: () => void;
  /** Set when no row can be added: the button is dimmed and these words stand beside it. */
  limit?: AddRowLimit | undefined;
  /** The link to the grid above (`useAddRowLink`): the arrows come and go between its last row and this button. */
  link?: AddRowLink | undefined;
}

export function AddRowButton({ labelId, onAdd, limit, link }: AddRowButtonProps) {
  const { l10n } = useLocalization();
  const reasonId = useId();
  const name = l10n.getString(labelId);
  return (
    <div
      className="flex min-w-0 items-center justify-end gap-3"
      data-add-row=""
      onKeyDown={(event) => {
        // ↑ goes back to the trash cell of the grid's last row (addLink.ts). Any other key is the button's own.
        if (link === undefined || event.key !== 'ArrowUp' || event.defaultPrevented) return;
        if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
        if (link.focusLastRemove()) event.preventDefault();
      }}
    >
      {limit === undefined ? null : (
        <Localized id={limit.messageId} vars={limit.vars ?? {}} elems={RUNNING_NUMBER}>
          <p id={reasonId} className="type-body min-w-0 text-ink-muted" />
        </Localized>
      )}
      <Button
        {...(link === undefined ? {} : { ref: link.buttonRef })}
        iconOnly
        round
        small
        variant="primary"
        accessibleName={name}
        title={name}
        disabled={limit !== undefined}
        {...(limit === undefined ? {} : { describedBy: reasonId })}
        onClick={onAdd}
      >
        <Plus aria-hidden="true" className="size-icon-sm" />
      </Button>
    </div>
  );
}
