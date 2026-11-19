// What a separate khata is linked to (PRD FR-16.6, plan 76 point 5), under the title of its list: nobody (the
// firm's own), the shop (every movement is split between the partners by their shares of that day) or one
// partner (the whole of it is his). A button changes it. The link is fixed while the khata's balance is not 0:
// then the button is off and the card says why, and the backend refuses it too (`ERR_LINK_LOCKED`). Four
// states of the dialog: the form, saving, a refusal at its field, saved. The backend splits and posts; the
// balance and the link are what it sent (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { Link2 } from 'lucide-react';
import { useState } from 'react';
import type { KhataAccount } from '../../../bindings/KhataAccount';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { isZeroPaisa } from '../../../shared/format';
import { useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import { Button, ErrorNotice, Fact, FormDialog, showToast } from '../../../shared/ui';
import { useSetKhataLink } from '../hooks';
import {
  KHATA_LINK_KINDS,
  LINK_FIELD_PATHS,
  linkDraftOf,
  linkOf,
  type LinkDraft,
} from '../link-draft';
import { LinkChoice } from './LinkChoice';
import { LinkWords } from './LinkWords';

function LinkForm({ khata, onClose }: { khata: KhataAccount; onClose: () => void }) {
  const { l10n } = useLocalization();
  const [draft, setDraft] = useState<LinkDraft>(() => linkDraftOf(khata.link));
  const [error, setError] = useState<UiError | null>(null);
  const setLink = useSetKhataLink();
  const flight = useSingleFlight();
  const submit = async () => {
    setError(null);
    try {
      await setLink.mutateAsync({ khata_account_id: khata.account.id, link: linkOf(draft) });
      showToast('success', l10n.getString('cashbook-link-saved'));
      onClose();
    } catch (failure: unknown) {
      setError(uiErrorOf(failure));
    }
  };
  const submitNow = () => {
    void flight.run(submit, undefined);
  };
  const navigation = useFormNavigation(submitNow);
  useShortcut(SHORTCUTS.save, submitNow);
  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-col gap-4"
    >
      {error === null ? null : <ErrorNotice error={error} fieldsShown={LINK_FIELD_PATHS} />}
      <LinkChoice
        draft={draft}
        onChange={setDraft}
        error={error}
        kinds={KHATA_LINK_KINDS}
        labelId="cashbook-link-label"
        helpId="cashbook-link-hint-khata"
      />
      <div className="flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy || setLink.isPending}
          onClick={submitNow}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
    </form>
  );
}

export function KhataLinkCard({ khata }: { khata: KhataAccount }) {
  const [open, setOpen] = useState(false);
  // The balance is the backend's; only whether it is 0 matters here.
  const locked = !isZeroPaisa(khata.balance_paisa);
  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <dl>
          <Fact label={<Localized id="cashbook-link-label" />}>
            <LinkWords link={khata.link} />
          </Fact>
        </dl>
        <Button
          disabled={locked}
          onClick={() => {
            setOpen(true);
          }}
        >
          <Link2 aria-hidden="true" className="size-icon-md" />
          <Localized id="cashbook-link-change" />
        </Button>
      </div>
      <Localized id={locked ? 'cashbook-link-locked' : 'cashbook-link-hint-khata'}>
        <p className="type-body text-ink-muted" />
      </Localized>
      <FormDialog
        open={open}
        onOpenChange={setOpen}
        title={<Localized id="cashbook-link-change" />}
      >
        {open ? (
          <LinkForm
            khata={khata}
            onClose={() => {
              setOpen(false);
            }}
          />
        ) : null}
      </FormDialog>
    </div>
  );
}
