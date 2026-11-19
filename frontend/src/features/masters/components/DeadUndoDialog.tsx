// "دوبارہ فعال کریں" (PRD FR-10.10, E73, plan 40): the move to the dead amount ends and the person
// is active again. His balance goes back into the normal tiles and lists. A reason is required and
// goes to the audit log. Built on the shared reason box; a refusal stays inside it.

import { Localized, useLocalization } from '@fluent/react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { uiErrorOf } from '../../../shared/api';
import { displayName, useLanguage } from '../../../shared/i18n';
import { ReasonDialog, ScriptText, showToast, type DeadUndoRequest } from '../../../shared/ui';
import { deadUndo } from '../api';

interface DeadUndoDialogProps {
  request: DeadUndoRequest;
  onClose: (undone: boolean) => void;
}

export function DeadUndoDialog({ request, onClose }: DeadUndoDialogProps) {
  const language = useLanguage();
  const { l10n } = useLocalization();
  const client = useQueryClient();
  const name = displayName(request.names, language);
  const undo = useMutation({
    mutationFn: (reason: string) => deadUndo({ mark_id: request.markId, reason }),
    onSuccess: () => {
      void client.invalidateQueries();
      showToast('success', l10n.getString('dead-undo-saved', { name }));
      onClose(true);
    },
  });
  return (
    <ReasonDialog
      open
      onOpenChange={(next) => {
        if (!next) onClose(false);
      }}
      title={
        <Localized id="dead-undo-title" vars={{ name }} elems={{ name: <ScriptText /> }}>
          <span />
        </Localized>
      }
      description={<Localized id="dead-undo-description" />}
      reasonLabel={<Localized id="dead-undo-reason" />}
      confirmLabel={<Localized id="dead-undo-confirm" />}
      confirmVariant="primary"
      error={undo.isError ? uiErrorOf(undo.error) : null}
      busy={undo.isPending}
      onConfirm={(reason) => {
        undo.mutate(reason);
      }}
    />
  );
}
