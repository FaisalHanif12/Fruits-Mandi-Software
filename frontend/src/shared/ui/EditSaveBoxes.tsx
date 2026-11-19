// The two boxes of the save of an edit (plan 44, PRD FR-7.70, FR-6.7): why it is changed, when the
// entry is not of today, and "is this entry correct?" when it takes cash in hand below zero. One
// place, so every edit asks in the same words. It reads everything from `useEditSave`.

import type { EditSave } from '../edit/useEditSave';
import { CashConfirmDialog } from './CashConfirmDialog';
import { EditReasonDialog } from './EditReasonDialog';

interface EditSaveBoxesProps {
  flow: EditSave;
  /** The day of the entry, `YYYY-MM-DD`, for the words of the reason box. */
  date: string;
  /** What the reason box starts with: the reason typed when the days were reopened. */
  initialReason?: string;
  /** After a "no" to the cash box the screen puts the cursor where it wants. */
  onCashNo?: () => void;
}

export function EditSaveBoxes({ flow, date, initialReason, onCashNo }: EditSaveBoxesProps) {
  return (
    <>
      <EditReasonDialog
        open={flow.askReason}
        onOpenChange={flow.setAskReason}
        date={date}
        error={null}
        busy={flow.busy}
        {...(initialReason === undefined ? {} : { initialReason })}
        onConfirm={flow.confirmReason}
      />
      {flow.cashAfter === null ? null : (
        <CashConfirmDialog
          open={flow.askCash}
          onOpenChange={(open) => {
            if (flow.onCashOpenChange(open)) onCashNo?.();
          }}
          cashAfter={flow.cashAfter}
          onConfirm={flow.confirmCash}
        />
      )}
    </>
  );
}
