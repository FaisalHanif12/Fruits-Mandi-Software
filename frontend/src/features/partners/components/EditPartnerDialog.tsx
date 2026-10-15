// The details of one partner (PRD 17.1, FR-17.2): his names, whether he is active, and the
// history of his shares. A new share is not set here: shares are always set for all partners
// together, so they total 100% (the setup list). Making a partner inactive needs his share to
// be 0 first; the backend says so with ERR_PARTNER_HAS_SHARE.
//
// The partner is read again when the dialog opens, so the version sent back is the current one.
// Keys: F10 saves, Esc closes.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight, useSpellingFlow } from '../../../shared/hooks';
import { formatBps, isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Checkbox,
  ErrorNotice,
  ErrorState,
  FormDialog,
  LoadingState,
  NamePair,
  Num,
  fieldError,
  showToast,
  type NamePairValue,
} from '../../../shared/ui';
import { usePartner, useUpdatePartner } from '../hooks';
import type { PartnerDetail } from '../types';

const FIELD_PATHS = ['name_ur', 'name_en'];

interface EditPartnerFormProps {
  detail: PartnerDetail;
  onClose: () => void;
}

function EditPartnerForm({ detail, onClose }: EditPartnerFormProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const { row } = detail;
  const [names, setNames] = useState<NamePairValue>({ urdu: row.name_ur, english: row.name_en });
  const [active, setActive] = useState(row.is_active);
  const update = useUpdatePartner();
  const spelling = useSpellingFlow();
  const error = update.error === null ? null : uiErrorOf(update.error);

  const flight = useSingleFlight();
  const submit = async () => {
    try {
      const saved = await spelling.guard(names, setNames, (confirmedWords) =>
        update.mutateAsync({
          id: row.id,
          version: row.version,
          name_ur: names.urdu,
          name_en: names.english,
          is_active: active,
          confirmed_rules_words: confirmedWords,
        }),
      );
      if (saved === undefined) return;
      showToast(
        'success',
        l10n.getString('partners-updated', { name: displayName(saved, language) }),
      );
      onClose();
    } catch {
      // The refusal is shown in the form.
    }
  };
  const run = () => {
    void flight.run(submit, undefined);
  };
  const navigation = useFormNavigation(run);
  useShortcut(SHORTCUTS.save, run);
  const history = [...detail.shares].reverse();

  return (
    <form
      {...navigation}
      onSubmit={(event) => {
        event.preventDefault();
      }}
      className="flex flex-col gap-4"
    >
      {error === null ? null : <ErrorNotice error={error} fieldsShown={FIELD_PATHS} />}
      <NamePair
        value={names}
        onChange={setNames}
        urduLabel={<Localized id="partners-field-name-ur" />}
        englishLabel={<Localized id="partners-field-name-en" />}
        urduError={fieldError(error, 'name_ur')}
        englishError={fieldError(error, 'name_en')}
      />
      <div className="flex flex-col gap-1">
        <Checkbox
          checked={active}
          onChange={setActive}
          label={<Localized id="partners-field-active" />}
        />
        <Localized id="partners-field-active-hint">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      <section className="flex flex-col gap-2">
        <Localized id="partners-history-title">
          <h3 className="type-label" />
        </Localized>
        {history.length === 0 ? (
          <Localized id="partners-history-empty">
            <p className="type-body text-ink-muted" />
          </Localized>
        ) : (
          <ul className="flex flex-col gap-1">
            {history.map((item) => (
              <li key={item.effective_from} className="type-body">
                <Localized
                  id="partners-history-row"
                  vars={{
                    date: isoToDisplayDate(item.effective_from),
                    share: formatBps(item.share_bps),
                  }}
                  elems={{ num: <Num /> }}
                >
                  <span />
                </Localized>
              </li>
            ))}
          </ul>
        )}
      </section>
      <div className="flex flex-wrap justify-end gap-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.save}
          busy={flight.busy || update.isPending}
          onClick={run}
        >
          <Localized id="layout-save-save" />
        </Button>
      </div>
      {spelling.dialog}
    </form>
  );
}

function EditPartnerBody({ id, onClose }: { id: number; onClose: () => void }) {
  const detail = usePartner(id);
  if (detail.isPending) return <LoadingState />;
  if (detail.isError) {
    return (
      <ErrorState
        title={<Localized id="partners-setup-failed" />}
        error={uiErrorOf(detail.error)}
        onRetry={() => {
          void detail.refetch();
        }}
      />
    );
  }
  return <EditPartnerForm key={detail.data.row.version} detail={detail.data} onClose={onClose} />;
}

interface EditPartnerDialogProps {
  /** The partner being edited, or null while the dialog is closed. */
  partnerId: number | null;
  onClose: () => void;
}

export function EditPartnerDialog({ partnerId, onClose }: EditPartnerDialogProps) {
  return (
    <FormDialog
      open={partnerId !== null}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
      title={<Localized id="partners-edit-title" />}
    >
      {partnerId === null ? null : <EditPartnerBody id={partnerId} onClose={onClose} />}
    </FormDialog>
  );
}
