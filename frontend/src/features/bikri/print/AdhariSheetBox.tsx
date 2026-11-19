// The second print box of the sale book, ادھاری تفصیل (PRD R5, FR-12.12, plan 78 A1, A5), opened with Shift+F9
// or its button. It prints only ادھاری تفصیل, how much the shop has to take from its buyers: one row for each of
// the buyers of the day, never a page for a buyer, and at the end the total of everyone who owes. Nothing else is
// in the box but the language, the paper, Print and Save as PDF (they are the preview frame's). It is always the
// whole day of the saved بکری: its کل ادھار is the shop's, not one بکری's. The backend works out every figure
// and writes every date; nothing here adds up.

import { Localized } from '@fluent/react';
import { uiErrorOf } from '../../../shared/api';
import { ErrorState, FormDialog, LoadingState } from '../../../shared/ui';
import {
  TablePrintPreview,
  reportsAdhariSheetPrint,
  reportsAdhariSheetPrintData,
} from '../../reports';
import { useBikriDetail } from '../hooks/useBikriReads';
import type { PrintTarget } from './target';

interface AdhariSheetBoxProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  target: PrintTarget;
}

function SheetBody({ target, onClose }: { target: PrintTarget; onClose: () => void }) {
  const detail = useBikriDetail(target.id);
  if (detail.isPending) return <LoadingState />;
  if (detail.isError) {
    return (
      <ErrorState
        title={<Localized id="print-preview-failed" />}
        error={uiErrorOf(detail.error)}
        onRetry={() => {
          void detail.refetch();
        }}
      />
    );
  }
  // The posting day of the بکری, as the backend stored it (`YYYY-MM-DD`).
  const date = detail.data.entry_date;
  return (
    <div className="flex flex-col gap-3">
      <Localized id="print-adhari-box-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      <TablePrintPreview
        scope={['adhari-sheet', date]}
        read={(language, paper) => reportsAdhariSheetPrintData({ date, language, paper })}
        send={(to, language, paper) =>
          reportsAdhariSheetPrint({ sheet: { date, language, paper }, to, copies: null })
        }
        report="R5"
        onBack={onClose}
      />
    </div>
  );
}

export function AdhariSheetBox({ open, onOpenChange, target }: AdhariSheetBoxProps) {
  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="print-adhari-title" />}
    >
      {/* The body is made when the box opens, so every opening reads the day again. */}
      {open ? (
        <SheetBody
          target={target}
          onClose={() => {
            onOpenChange(false);
          }}
        />
      ) : null}
    </FormDialog>
  );
}
