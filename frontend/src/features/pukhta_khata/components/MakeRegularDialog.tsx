// "Make regular" (PRD FR-13.8): a one-time person who starts coming often gets a proper account,
// and his open balance moves to it with a جنرل entry, in one action. The form is the quick add of
// an account with his names and phone filled in; saving calls `pukhta_make_regular` instead of the
// plain account save. The new account comes back to the screen that opened the dialog.

import { Localized, useLocalization } from '@fluent/react';
import type { MadeRegular } from '../../../bindings/MadeRegular';
import type { MakeRegularInput } from '../../../bindings/MakeRegularInput';
import type { SavedPukhta } from '../../../bindings/SavedPukhta';
import type { PukhtaSide } from '../../../bindings/PukhtaSide';
import { accountInputOf } from '../../../shared/accounts';
import { QuickAddAccount, showToast } from '../../../shared/ui';

interface MakeRegularDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The one-time person and the side of his account. */
  person: SavedPukhta;
  side: PukhtaSide;
  /**
   * Makes the account and moves his balance (the command, through the hook of the screen that
   * opened the dialog, so the lists it changes are read again).
   */
  onMake: (input: MakeRegularInput) => Promise<MadeRegular>;
  /** The account was made and his balance moved. */
  onDone: (made: MadeRegular) => void;
}

export function MakeRegularDialog({
  open,
  onOpenChange,
  person,
  side,
  onMake,
  onDone,
}: MakeRegularDialogProps) {
  const { l10n } = useLocalization();
  return (
    <QuickAddAccount
      open={open}
      onOpenChange={onOpenChange}
      scope={side}
      typed=""
      title={<Localized id="pukhta-make-regular-title" />}
      help={<Localized id="pukhta-make-regular-help" />}
      initial={{
        names: { urdu: person.name_ur, english: person.name_en },
        phone: person.phone,
      }}
      saveWith={async (draft, confirmedWords) => {
        const made = await onMake({
          person_key: person.person_key,
          side,
          account_input: accountInputOf(draft, confirmedWords),
        });
        showToast('success', l10n.getString('pukhta-make-regular-done'));
        onDone(made);
      }}
      onSaved={() => undefined}
    />
  );
}
