// The full Index form of a new record, opened above any screen by the link "Full form" of the short
// form of "add it here" (plan 41, PRD FR-7.69; before, plan 32 R8, it was the picker's own button).
// It is the real Index form (the account form, or the form of a master), with the type chosen and
// what was typed in the picker already in the name box. It has the fields the short form does not
// ask: the opening balance, the credit limit and the others.
//
// It is a dialog, not a new address: the screen underneath stays as it is, so a sale in progress, a
// voucher, the lines of a grid and the selected row are all still there when the form closes. Save
// or Esc closes it; the picker takes the focus back. If a record was saved, the picker puts it in
// the field.
//
// Pickers inside this form can add a record of their own with the short form (a city for the
// account), because the host's dialogs are still in reach; they get no second full form.

import { Localized } from '@fluent/react';
import { uiErrorOf } from '../../../shared/api';
import { useEscape } from '../../../shared/keyboard';
import {
  ErrorState,
  FormDialog,
  LoadingState,
  useWorkAreaStore,
  type AddedRecord,
  type AddRecordKind,
  type AddRecordRequest,
} from '../../../shared/ui';
import { useDayStatus } from '../../../app/use-day-status';
import { configOfKind } from '../configs';
import { useNextCode } from '../hooks';
import type { MasterKind } from '../types';
import { AccountForm } from './AccountForm';
import { MasterForm } from './MasterForm';

interface IndexAddDialogProps {
  /** What the picker asked for, or null while the dialog is closed. */
  request: AddRecordRequest | null;
  /** The form closed: with the record that was saved, or null if it was left. */
  onClose: (added: AddedRecord | null) => void;
}

function NewAccount({
  type,
  text,
  onDone,
}: {
  type: 'seller' | 'buyer';
  text: string;
  onDone: (added: AddedRecord | null) => void;
}) {
  const day = useDayStatus();
  if (day.isPending) return <LoadingState />;
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="accounts-load-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  return <AccountForm account={null} today={day.data.today} embedded={{ type, text, onDone }} />;
}

function NewMaster({
  kind,
  text,
  onDone,
}: {
  kind: MasterKind;
  text: string;
  onDone: (added: AddedRecord | null) => void;
}) {
  const config = configOfKind(kind);
  const nextCode = useNextCode(config.kind, config.hasCode);
  if (config.hasCode && nextCode.isPending) return <LoadingState />;
  if (config.hasCode && nextCode.isError) {
    return (
      <ErrorState
        title={<Localized id="masters-load-failed" />}
        error={uiErrorOf(nextCode.error)}
        onRetry={() => {
          void nextCode.refetch();
        }}
      />
    );
  }
  return (
    <MasterForm
      config={config}
      record={null}
      suggestedCode={config.hasCode ? (nextCode.data ?? '') : ''}
      embedded={{ text, onDone }}
    />
  );
}

/** Which Index form a request opens: a new account of a type, or a new record of a master. */
type Target = { account: 'seller' | 'buyer' } | { master: MasterKind };

function targetOf(kind: AddRecordKind): Target {
  switch (kind) {
    case 'seller':
      return { account: 'seller' };
    // A field that takes any account starts as a buyer; the form's type list can change it.
    case 'buyer':
    case 'any':
      return { account: 'buyer' };
    default:
      return { master: kind };
  }
}

/**
 * The way out while the form is still loading, or has an error: the form has its own Esc once it
 * is open, and its handler comes later, so it answers first (shared/keyboard/escape.ts). It sits
 * inside the dialog, so it belongs to the dialog's layer.
 */
function EscapeToClose({ onClose }: { onClose: (added: AddedRecord | null) => void }) {
  useEscape('leave', () => {
    onClose(null);
  });
  return null;
}

export function IndexAddDialog({ request, onClose }: IndexAddDialogProps) {
  const target = request === null ? null : targetOf(request.kind);
  // The form fills the work area beside the main menu, like the Index page it is (decision D1).
  const slot = useWorkAreaStore((state) => state.slot);
  return (
    <FormDialog
      open={request !== null}
      onOpenChange={() => undefined}
      layout="screen"
      container={slot}
      escapeToApp
      title={
        target === null ? null : 'account' in target ? (
          <Localized id="accounts-new-title" />
        ) : (
          <Localized id={configOfKind(target.master).newId} />
        )
      }
    >
      {request === null || target === null ? null : (
        <>
          <EscapeToClose onClose={onClose} />
          {'account' in target ? (
            <NewAccount type={target.account} text={request.text} onDone={onClose} />
          ) : (
            <NewMaster kind={target.master} text={request.text} onDone={onClose} />
          )}
        </>
      )}
    </FormDialog>
  );
}
